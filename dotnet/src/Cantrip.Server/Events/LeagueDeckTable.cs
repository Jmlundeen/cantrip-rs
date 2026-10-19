using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// The table a league run's deck is changed at between its matches: one seat, the run's player, shown the same deck
/// builder a draft ends with - the run's pool, its deck, any basic lands. Every change is kept as it is made
/// (<see cref="League.DeckChanged"/>); saving the deck (the builder's Submit) is done with the table, which then says so
/// and leads back to the league's page. A player who does not come, or leaves, has the table closed after a while
/// (<see cref="LeagueTiming.DeckAway"/>), the deck kept as they left it.
/// <para>All of its state is the league's, kept under the league's lock.</para>
/// </summary>
public sealed class LeagueDeckTable : ITableEvent
{
    private readonly League _league;
    private readonly LeagueRun _run;
    private readonly List<string> _tickets = [];
    private readonly DateTimeOffset _made;
    private DateTimeOffset? _awaySince;
    private bool _seen;
    private int _change;
    private int _version;
    private string _detail = string.Empty;

    internal LeagueDeckTable(League league, LeagueRun run)
    {
        _league = league;
        _run = run;
        _made = league.Now;
    }

    public string PlayerId => _run.PlayerId;

    /// <summary>Where the table is, once the host has opened it.</summary>
    public Uri? Address { get; private set; }

    /// <summary>Whether the table is done with - the deck saved, or the table closed - and the host can close it.</summary>
    public bool Done { get; private set; }

    /// <summary>The host has opened the table here.</summary>
    public void Opened(Uri address)
    {
        ArgumentNullException.ThrowIfNull(address);
        lock (_league.Gate) Address = address;
    }

    /// <summary>A ticket to the table for its player.</summary>
    internal string IssueTicket()
    {
        string ticket = BoosterPod.NewTicket();
        _tickets.Add(EventStore.HashTicket(ticket));
        return ticket;
    }

    /// <summary>Whether the table has waited long enough for a player who has not come, or has gone.</summary>
    internal bool Expired(DateTimeOffset now) =>
        !Done && (now - _made >= _league.Timing.DeckMost
            || (!_seen ? now - _made >= _league.Timing.DeckAway : _awaySince is { } away && now - away >= _league.Timing.DeckAway));

    /// <summary>Nothing more is done here; the table says why to whoever is at it.</summary>
    internal void Close(string why)
    {
        if (Done) return;
        Done = true;
        _detail = why;
        _version++;
    }

    public string EventId => _league.EventId;

    public string Name => $"{_league.Name}: your deck";

    public int Seats => 1;

    public TablePhase Phase
    {
        get { lock (_league.Gate) return Done ? TablePhase.Finished : TablePhase.DeckBuilding; }
    }

    public string Detail
    {
        get { lock (_league.Gate) return _detail; }
    }

    public int SeatFor(string ticket)
    {
        if (string.IsNullOrEmpty(ticket)) return 0;
        lock (_league.Gate) return BoosterPod.Matching(ticket, [(1, _tickets)]);
    }

    public string NameAt(int seat) => _run.Name;

    public string PlayerAt(int seat) => seat == 1 ? _run.PlayerId : string.Empty;

    public void Seated(int seat)
    {
        lock (_league.Gate) _seen = true;
    }

    public void Connected(int seat, bool connected)
    {
        lock (_league.Gate)
        {
            _awaySince = connected ? null : _league.Now;
            if (connected) _seen = true;
        }
    }

    public int Version
    {
        get { lock (_league.Gate) return _version; }
    }

    public ServerMessage? ViewFor(int seat)
    {
        lock (_league.Gate)
        {
            if (Done || seat != 1) return null;
            return _league.Cards.Deck(_league.EventId, _run.Pool, _run.Deck, _league.MinimumDeck, _change, null, _league.Now,
                $"Your deck for {_league.Name} ({_league.RecordLine(_run)})",
                [new DeckBuilderSeatView { Seat = 1, Name = _run.Name, Submitted = _run.Deck.Submitted }]);
        }
    }

    public string? Receive(int seat, ClientMessage message)
    {
        lock (_league.Gate)
        {
            if (message is not BuildDeckMessage build) return "That is not something to do at this table.";
            if (Done || seat != 1) return "Your deck is not being changed now.";
            _change = build.Change;
            (EventDeck? deck, string? refusal) = PoolDecks.Built(_run.Deck, _run.Pool.Count, build, _league.Cards.Basics, _league.MinimumDeck);
            if (deck is null) return refusal;
            _league.DeckChanged(_run, deck);
            if (deck.Submitted)
            {
                Close("Your deck is saved. Your next match is played with it.");
            }
            else
            {
                _version++;
            }

            return null;
        }
    }

    public bool ReadyToDeal => false;

    public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null) =>
        throw new InvalidOperationException("A deck table deals no game.");

    public void DealFailed(string why)
    {
    }

    public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail)
    {
    }
}
