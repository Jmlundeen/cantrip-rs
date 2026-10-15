using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Tests;

/// <summary>
/// A rules factory that deals nothing: it keeps the seats it was asked to deal and hands back a host that does not
/// play. For a test of what a table or an event would deal, which must not depend on which cards the native engine has
/// ported so far.
/// </summary>
internal sealed class CapturingRulesFactory : IRulesHostFactory
{
    private readonly Func<string, bool> _known;

    /// <param name="known">Which card names the engine has, as the real host refuses a deal naming one it lacks; all of them by default.</param>
    public CapturingRulesFactory(Func<string, bool>? known = null)
    {
        _known = known ?? (_ => true);
    }

    public IReadOnlyList<RulesSeat> Seats { get; private set; } = [];

    public ulong Seed { get; private set; }

    public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options)
    {
        foreach (RulesSeat seat in seats)
        {
            if (seat.Deck.Concat(seat.Sideboard).FirstOrDefault(card => !_known(card)) is { } unknown)
            {
                throw new RulesRejectedException(ErrorCode.InvalidSetup, $"{seat.Name}'s deck names {unknown}, which is not a card the engine has");
            }
        }

        Seats = seats;
        Seed = seed;
        return new InertHost(seats);
    }

    private sealed class InertHost : IRulesHost
    {
        private readonly IReadOnlyList<RulesSeat> _seats;

        public InertHost(IReadOnlyList<RulesSeat> seats)
        {
            _seats = seats;
        }

        public IReadOnlyList<int> Seats => [.. Enumerable.Range(1, _seats.Count)];

        public IReadOnlyList<string> SeatNames => [.. _seats.Select(seat => seat.Name)];

        public int Team(int seat) => seat;

        public IReadOnlyList<DeckCardView> Decklist(int seat) => [];

        public RulesStep Start() => throw new NotSupportedException();

        public RulesStep Answer(int requestId, int seat, AnswerPayload answer) => throw new NotSupportedException();

        public bool CanConcede(int seat) => false;

        public RulesStep Concede(int seat) => throw new NotSupportedException();

        public PlayerView View(int seat) => throw new NotSupportedException();

        public ObjectView? Oracle(string cardId) => null;

        public bool IsOver => false;

        public GameOutcomeView Outcome => GameOutcomeView.InProgress;

        public int WinningTeam => 0;

        public IReadOnlyList<RulesAnswer> History => [];

        public void Dispose()
        {
        }
    }
}
