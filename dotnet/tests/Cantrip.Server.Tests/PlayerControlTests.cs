using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// A question asked of a player another seat controls (CR 723.5: Mindslaver, Emrakul, the Promised End): the rules send it
/// to the controlling seat, naming the player it is for, and the session passes it on to that seat with
/// <see cref="PromptMessage.ForSeat"/> and takes the answer from that seat alone. The controlled player's priority is not the
/// controlling seat's to offer a draw with (CR 723.5b: it would pass their priority).
/// </summary>
public sealed class PlayerControlTests
{
    [Fact]
    public void A_controlled_players_question_goes_to_the_controlling_seat_for_that_player()
    {
        var rules = new Controlled();
        using var session = new GameSession(rules);
        session.Start();

        PromptMessage prompt = Assert.Single(session.Drain(new PlayerId(1)).OfType<PromptMessage>());
        Assert.Equal(2, prompt.ForSeat);
        Assert.IsType<PriorityPromptMessage>(prompt.Prompt);
        Assert.Empty(session.Drain(new PlayerId(2)).OfType<PromptMessage>());
        Assert.Equal([1], session.WaitingOn);

        // The controlled player's priority offers no draw: the offer would pass it.
        Assert.NotNull(session.CannotOfferDraw(new PlayerId(1)));
        Assert.NotNull(session.CannotOfferDraw(new PlayerId(2)));

        // The seat it was sent to answers it; the controlled seat may not.
        session.Submit(new PlayerId(2), new AnswerMessage { RequestId = prompt.RequestId, Answer = new PassAnswer() });
        Assert.Empty(rules.Answered);
        Assert.Contains(session.Drain(new PlayerId(2)).OfType<ErrorMessage>(), error => error.Code == ErrorCode.NotYourTurnToAnswer);
        session.Submit(new PlayerId(1), new AnswerMessage { RequestId = prompt.RequestId, Answer = new PassAnswer() });
        Assert.Equal([1], rules.Answered);
    }

    /// <summary>Two seats; seat 1 controls seat 2, whose priority is asked, and is sent it (the host's <c>ForSeat</c>).</summary>
    private sealed class Controlled : IRulesHost
    {
        public List<int> Answered { get; } = [];

        public IReadOnlyList<int> Seats => [1, 2];

        public IReadOnlyList<string> SeatNames => ["Ann", "Bob"];

        public int Team(int seat) => seat;

        public IReadOnlyList<DeckCardView> Decklist(int seat) => [];

        public RulesStep Start() => Step();

        public RulesStep Answer(int requestId, int seat, AnswerPayload answer)
        {
            Answered.Add(seat);
            return Step();
        }

        public bool CanConcede(int seat) => false;

        public RulesStep Concede(int seat) => throw new NotSupportedException();

        public RulesStep Leave(int seat) => throw new NotSupportedException();

        public PlayerView View(int seat) => ViewFor(seat);

        public ObjectView? Oracle(string cardId) => null;

        public bool IsOver => false;

        public GameOutcomeView Outcome => GameOutcomeView.InProgress;

        public int WinningTeam => 0;

        public IReadOnlyList<RulesAnswer> History => [];

        public void Dispose()
        {
        }

        private RulesStep Step() => new(
            [new RulesRequest(Answered.Count + 1, 1, new PriorityPromptMessage { Player = 2 }, false, null, ForSeat: 2)],
            Seats.ToDictionary(seat => seat, ViewFor),
            Seats.ToDictionary(seat => seat, _ => (IReadOnlyList<(long Sequence, GameEventView View)>)[]),
            0,
            false);

        private PlayerView ViewFor(int viewer) => new()
        {
            Viewer = viewer,
            ActivePlayer = 2,
            PriorityPlayer = 2,
            Players = [.. Seats.Select(seat => new PlayerSummaryView
            {
                Seat = seat,
                Name = SeatNames[seat - 1],
                Team = seat,
                Status = PlayerStatusView.Active,
                ControlledBy = seat == 2 ? 1 : 0,
            })],
        };
    }
}
