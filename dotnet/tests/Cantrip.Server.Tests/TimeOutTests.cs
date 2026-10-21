using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Seats out of time in a game of three (GameSession.TimeOut; CR 104.3a, 800.4a): each leaves through the rules, and when
/// every seat still in runs out at once nobody is left to win - a draw - judged by who is really still in, not by one seat's
/// view, which may show a position from before the run being played.
/// </summary>
public sealed class TimeOutTests
{
    /// <summary>
    /// Seat one has left; then seats two and three run out together. Seat one's own view still shows everyone in (it is not
    /// the seat answering, so it is shown the position from before), but seats two and three are all that is left, both out:
    /// a draw, and nobody else is made to leave - not seat two, leaving seat three the winner.
    /// </summary>
    [Fact]
    public void Two_seats_out_together_after_a_third_left_draw_though_one_view_is_behind()
    {
        var rules = new ThreeSeats();
        using var session = new GameSession(rules);
        session.Start();

        session.TimeOut([1]);
        Assert.Equal([1], rules.Left);
        Assert.False(session.IsOver);
        Assert.Equal([2, 3], session.InGame);

        session.TimeOut([2, 3]);
        Assert.Equal([1], rules.Left);
        Assert.True(session.IsOver);
        GameOverMessage over = Assert.IsType<GameOverMessage>(session.Drain(new PlayerId(3)).Last(message => message is GameOverMessage));
        Assert.Equal(GameOutcomeView.Draw, over.Outcome);
    }

    /// <summary>
    /// A game of three whose seat one is asked first, and whose seat one's view, once it has left, still shows the position
    /// from before (as the host shows a run to its answering seat alone): the others see who has left.
    /// </summary>
    private sealed class ThreeSeats : IRulesHost
    {
        public List<int> Left { get; } = [];

        public IReadOnlyList<int> Seats => [1, 2, 3];

        public IReadOnlyList<string> SeatNames => ["Ann", "Bob", "Cy"];

        public int Team(int seat) => seat;

        public IReadOnlyList<DeckCardView> Decklist(int seat) => [];

        public RulesStep Start() => Step(asked: 1);

        public RulesStep Answer(int requestId, int seat, AnswerPayload answer) => throw new NotSupportedException();

        public bool CanConcede(int seat) => false;

        public RulesStep Concede(int seat) => throw new NotSupportedException();

        public RulesStep Leave(int seat)
        {
            Left.Add(seat);
            IsOver = Left.Count >= 2;
            return Step(asked: Seats.First(each => !Left.Contains(each)));
        }

        public PlayerView View(int seat) => ViewFor(seat);

        public ObjectView? Oracle(string cardId) => null;

        public bool IsOver { get; private set; }

        public GameOutcomeView Outcome => IsOver ? GameOutcomeView.TeamWon : GameOutcomeView.InProgress;

        public int WinningTeam => IsOver ? Seats.First(each => !Left.Contains(each)) : 0;

        public IReadOnlyList<RulesAnswer> History => [];

        public void Dispose()
        {
        }

        private RulesStep Step(int asked) => new(
            IsOver ? [] : [new RulesRequest(Left.Count + 1, asked, new PriorityPromptMessage { Player = asked }, false, null)],
            Seats.ToDictionary(seat => seat, ViewFor),
            Seats.ToDictionary(seat => seat, _ => (IReadOnlyList<(long Sequence, GameEventView View)>)[]),
            0,
            IsOver);

        private PlayerView ViewFor(int viewer) => new()
        {
            Viewer = viewer,
            Players = [.. Seats.Select(seat => new PlayerSummaryView
            {
                Seat = seat,
                Name = SeatNames[seat - 1],
                Team = seat,
                // Seat one's view is behind: everyone still in.
                Status = viewer != 1 && Left.Contains(seat) ? PlayerStatusView.Conceded : PlayerStatusView.Active,
            })],
        };
    }
}
