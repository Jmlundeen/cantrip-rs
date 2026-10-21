using System.Security.Cryptography;
using Cantrip.Cards.Model;
using Cantrip.Core.Random;
using Cantrip.Datasets;
using Cantrip.Projection;
using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>What a host says about a Winston draft it is creating.</summary>
public sealed class WinstonDraftSetup
{
    /// <summary>The event's id, for a host that named it before making it (<see cref="EventStore.NewEventId"/>); null makes one.</summary>
    public string? Id { get; init; }

    public required string Name { get; init; }

    /// <summary>The cube's name, for the record.</summary>
    public required string Cube { get; init; }

    /// <summary>The cube's cards by name. A name no loaded card has is left out, and said so in the result.</summary>
    public required IReadOnlyList<string> CubeCards { get; init; }

    /// <summary>The cube's cards best first (<see cref="CubeList.PickOrder"/>), for a bot to draft by; empty for none.</summary>
    public IReadOnlyList<string> PickOrder { get; init; } = [];

    public int Piles { get; init; } = 3;

    /// <summary>How many cards are drafted, drawn at random from the cube; zero drafts the whole cube.</summary>
    public int Cards { get; init; }

    /// <summary>The players by id and name, seated in this order.</summary>
    public required IReadOnlyList<(string Id, string Name)> Players { get; init; }

    public ulong Seed { get; init; }

    public int MinimumDeck { get; init; } = 40;

    /// <summary>The event this draft is part of - a league - or null.</summary>
    public string? ParentId { get; init; }
}

/// <summary>What a host says about a grid draft it is creating: two players, and the number of grids.</summary>
public sealed class GridDraftSetup
{
    /// <summary>The event's id, for a host that named it before making it (<see cref="EventStore.NewEventId"/>); null makes one.</summary>
    public string? Id { get; init; }

    public required string Name { get; init; }

    /// <summary>The cube's name, for the record.</summary>
    public required string Cube { get; init; }

    /// <summary>The cube's cards by name. A name no loaded card has is left out, and said so in the result.</summary>
    public required IReadOnlyList<string> CubeCards { get; init; }

    /// <summary>The cube's cards best first (<see cref="CubeList.PickOrder"/>), for a bot to draft by; empty for none.</summary>
    public IReadOnlyList<string> PickOrder { get; init; } = [];

    /// <summary>How many grids - packs of nine - are laid out, drawn at random from the cube.</summary>
    public int Packs { get; init; } = 18;

    /// <summary>The two players by id and name, seated in this order.</summary>
    public required IReadOnlyList<(string Id, string Name)> Players { get; init; }

    public ulong Seed { get; init; }

    public int MinimumDeck { get; init; } = 40;

    /// <summary>The event this draft is part of - a league - or null.</summary>
    public string? ParentId { get; init; }
}

/// <summary>
/// How long everything in a draft at one table is given, as a booster draft's <see cref="BoosterTiming"/> is: nothing
/// waits for a player who is not there. Not saved with the event: a host decides, and a test gives the clocks seconds
/// rather than minutes.
/// </summary>
public sealed class DraftTiming
{
    /// <summary>How long the players have to take their seats, by their tickets, before the event is called off.</summary>
    public TimeSpan Seating { get; init; } = TimeSpan.FromMinutes(10);

    /// <summary>A grid draft's clock on each pick: a row or a column of the grid.</summary>
    public TimeSpan GridPick { get; init; } = TimeSpan.FromSeconds(45);

    /// <summary>A Winston draft's clock on each turn: every pile looked at, to the one taken.</summary>
    public TimeSpan WinstonTurn { get; init; } = TimeSpan.FromSeconds(60);

    /// <summary>How long decks are built for once the draft is over, before each deck as it stands is submitted.</summary>
    public TimeSpan DeckBuilding { get; init; } = TimeSpan.FromMinutes(10);

    /// <summary>How long a player who is not at the table when the game is dealt has to come to it before they forfeit.</summary>
    public TimeSpan NoShow { get; init; } = TimeSpan.FromMinutes(3);

    /// <summary>How long a player may be gone from the table while the game is played before they forfeit.</summary>
    public TimeSpan Away { get; init; } = TimeSpan.FromMinutes(5);

    /// <summary>
    /// Each player's clock for the game: a chess clock, running while the game waits on them - the mulligans, priority,
    /// every choice. A player whose clock runs out loses. An hour, as every table's (docs/events.md, "Clocks"); zero leaves
    /// the players the table's own.
    /// </summary>
    public TimeSpan MatchClock { get; init; } = TimeSpan.FromHours(1);
}

/// <summary>What a draft played at one table keeps of its own, whatever the format: what each seat has taken, and whose turn it is.</summary>
public interface IPoolDraft
{
    /// <summary>What each seat has taken, in the order it came: seat 1's first.</summary>
    List<List<string>> Pools { get; set; }

    IReadOnlyList<string> PoolOf(int seat);

    /// <summary>Whose turn it is, by seat; zero once the draft is over.</summary>
    int Active { get; }

    bool IsOver { get; }
}

/// <summary>A new event, the cube names that were left out of it for want of a card, and a ticket for each player.</summary>
public sealed record CreatedEvent(EventRecord Record, IReadOnlyList<string> Missing, IReadOnlyDictionary<string, string> Tickets);

/// <summary>
/// A draft played at one table: players take their seats, draft, build decks from what they drafted, and play. Owns the
/// event's record and its draft while the table runs, and saves both after everything that changes them, so the event
/// picks up where it was after a restart, and a player picks up where they were from any computer.
/// <para>
/// A Winston draft (<see cref="WinstonDraft"/>) or a grid draft (<see cref="GridDraft"/>). Another format is another
/// draft state behind the same record: the pool, deck, rounds and matches belong to the event, not to the way it was
/// drafted.
/// </para>
/// <para>
/// A round cut off by a restart is dealt again from its start, from the same seed: the game itself is not saved.
/// </para>
/// <para>
/// Nothing waits for a player who is not there (<see cref="DraftTiming"/>). Players who do not take their seats call the
/// event off; deck building's clock submits each deck as it stands; and in the game, each player has a match clock, and
/// one who does not come to the table or is gone from it too long forfeits (<see cref="ITableEvent.TakeForfeits"/>).
/// </para>
/// <para>
/// Every pick is on a clock, whether its player is there or not. One that runs out picks for
/// them - a grid's line by the cube's pick order (<see cref="GridDraft.ClockLine"/>), the rest of a Winston turn by a
/// plain rule (<see cref="WinstonDraft.TakeForClock"/>) - so nobody waits on a player who has gone. The clock starts
/// again from a restart.
/// </para>
/// </summary>
public sealed class DraftPod : ITableEvent
{
    /// <summary>The basic lands a deck may add as many of as it likes (CR 100.2a).</summary>
    public static IReadOnlyList<string> BasicLands => PoolDecks.BasicLands;

    /// <summary>How the event's line begins when its game could not be dealt; the reason follows, which may name a pool's card.</summary>
    public const string DealFailedLine = "The game could not be dealt";

    /// <summary>How many tickets per player are kept working: the newest few.</summary>
    private const int TicketsKept = 8;

    private readonly object _gate = new();
    private readonly EventStore _store;
    private readonly EventRecord _record;
    private readonly IPoolDraft _draft;
    private readonly CardCatalog _cards;
    private readonly CardOracle _oracle;
    private readonly Dictionary<string, CardId> _basics;
    private readonly TimeProvider _time;

    /// <summary>
    /// The last change each seat's client sent that has been dealt with - applied, or refused - which its deck view
    /// carries back (<see cref="DeckBuildingMessage.Change"/>). Only for the connection that sent them, so not saved.
    /// </summary>
    private readonly Dictionary<int, int> _changes = [];

    /// <summary>How long each pick is given; zero for no clock.</summary>
    private readonly TimeSpan _pickTime;

    /// <summary>The pick clock running: the turn it is on (<see cref="Turn"/>) and when it runs out. Not saved.</summary>
    private (int Turn, DateTimeOffset By)? _clock;

    /// <summary>The line of the grid the active player has chosen but not yet taken, and the pick it was chosen for.</summary>
    private (int Decision, int Line)? _chosenLine;

    private int _version;

    private readonly DraftTiming _timing;

    /// <summary>The seats with the table open.</summary>
    private readonly HashSet<int> _connected = [];

    /// <summary>When the game being played was dealt, and when each seat was first at the table since; null before a deal.</summary>
    private DateTimeOffset? _dealtAt;
    private readonly Dictionary<int, DateTimeOffset> _seenSinceDeal = [];
    private readonly Dictionary<int, DateTimeOffset> _awaySince = [];

    /// <summary>Seats that have forfeited the game being played, for the table to end it (<see cref="TakeForfeits"/>).</summary>
    private readonly List<int> _forfeits = [];

    /// <summary>Whether the game being played was forfeited: the table ends it, and its match is marked so.</summary>
    private bool _forfeited;

    /// <summary>Why a seat forfeited, when it was not for being away: its player dropped (<see cref="Drop"/>).</summary>
    private readonly Dictionary<int, string> _forfeitReasons = [];

    private DraftPod(EventStore store, EventRecord record, IPoolDraft draft, ContentLibrary content, TimeProvider time, DraftTiming timing)
    {
        _store = store;
        _record = record;
        _draft = draft;
        _time = time;
        _timing = timing;
        _pickTime = draft is GridDraft ? timing.GridPick : timing.WinstonTurn;
        _cards = content.Catalog();
        _oracle = new CardOracle(_cards);
        _basics = PoolDecks.Basics(content);
        Reconcile();
        TakeUp();
    }

    /// <summary>
    /// Starts the seating clock of an event nobody has sat down at yet, and gives a clock that ran out while the table was
    /// down a minute more: time the table was down for is nobody's.
    /// </summary>
    private void TakeUp()
    {
        DateTimeOffset now = _time.GetUtcNow();
        // Seating, or decks being built in an event saved before deck building had a clock.
        if (_record.Deadline is null && (_record.Stage == EventStage.Seating
            || (_record.Stage == EventStage.DeckBuilding && _record.Participants.Any(participant => !participant.Deck.Submitted))))
        {
            _record.Deadline = now + (_record.Stage == EventStage.Seating ? _timing.Seating : _timing.DeckBuilding);
            Save();
        }

        TimeSpan grace = TimeSpan.FromMinutes(1);
        if (_record.Stage is EventStage.Seating or EventStage.DeckBuilding && _record.Deadline is { } deadline && deadline < now + grace)
        {
            _record.Deadline = now + grace;
            Save();
        }
    }

    /// <summary>
    /// Takes the pools from the draft while it runs, since the draft is saved before the event after every pick: a
    /// process stopped between the two saves leaves the event a pick behind, never a card short or a card over. A draft
    /// that finished in that gap finishes here.
    /// </summary>
    private void Reconcile()
    {
        if (_record.Stage is not (EventStage.Seating or EventStage.Drafting))
        {
            return;
        }

        if (_draft.Pools.Count == 0)
        {
            _draft.Pools = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new List<string>(participant.Pool))];
        }

        bool behind = false;
        foreach (EventParticipant participant in _record.Participants)
        {
            if (!participant.Pool.SequenceEqual(_draft.PoolOf(participant.Seat)))
            {
                participant.Pool = [.. _draft.PoolOf(participant.Seat)];
                behind = true;
            }
        }

        if (_record.Stage == EventStage.Drafting && _draft.IsOver)
        {
            FinishDraft();
            behind = true;
        }

        if (behind)
        {
            Save();
        }
    }

    /// <summary>
    /// Makes a Winston draft and saves it, ready for its players to sit down: the cube's cards that are loaded, drawn
    /// at random down to the number to draft and shuffled into the stack, who looks first chosen at random, and a
    /// ticket for each player. Each player's record notes they are in it.
    /// </summary>
    public static CreatedEvent CreateWinston(EventStore store, ContentLibrary content, WinstonDraftSetup setup, TimeProvider? time = null)
    {
        ArgumentNullException.ThrowIfNull(setup);

        // A table seats two to four.
        if (setup.Players.Count is < 2 or > 4)
            throw new ArgumentException("A Winston draft at one table needs two to four players.", nameof(setup));

        var made = new Made(setup.Id, setup.Name, setup.ParentId, setup.Players, setup.Seed, setup.MinimumDeck, setup.CubeCards);
        return Create(store, content, made, EventKinds.WinstonDraft, DraftFormats.Winston, time, (cube, first, random) =>
        {
            int count = setup.Cards == 0 ? cube.Count : setup.Cards;
            if (count > cube.Count)
                throw new ArgumentException($"The cube has {cube.Count} cards that can be played; {count} were asked for.", nameof(setup));
            return (WinstonDraft.Start(cube[..count], setup.Piles, setup.Players.Count, first, random, CardIdsOf(content, setup.PickOrder)),
                new DraftSettings { Format = DraftFormats.Winston, Cube = setup.Cube, Piles = setup.Piles, Cards = count });
        });
    }

    /// <summary>
    /// Makes a grid draft and saves it, ready for its two players to sit down: nine of the cube's loaded cards a pack,
    /// drawn at random, the first grid laid out, who picks first from it chosen at random, and a ticket for each player.
    /// </summary>
    public static CreatedEvent CreateGrid(EventStore store, ContentLibrary content, GridDraftSetup setup, TimeProvider? time = null)
    {
        ArgumentNullException.ThrowIfNull(content);
        ArgumentNullException.ThrowIfNull(setup);
        if (setup.Players.Count != 2)
            throw new ArgumentException("A grid draft is for two players.", nameof(setup));
        if (setup.Packs < 1)
            throw new ArgumentException("A grid draft lays out at least one grid.", nameof(setup));

        var made = new Made(setup.Id, setup.Name, setup.ParentId, setup.Players, setup.Seed, setup.MinimumDeck, setup.CubeCards);
        return Create(store, content, made, EventKinds.GridDraft, DraftFormats.Grid, time, (cube, first, random) =>
        {
            int count = setup.Packs * GridDraft.PackSize;
            if (count > cube.Count)
                throw new ArgumentException($"The cube has {cube.Count} cards that can be played; {setup.Packs} grids of {GridDraft.PackSize} take {count}.", nameof(setup));

            return (GridDraft.Start(cube[..count], setup.Packs, first, random, CardIdsOf(content, setup.PickOrder)),
                new DraftSettings { Format = DraftFormats.Grid, Cube = setup.Cube, Packs = setup.Packs, PackSize = GridDraft.PackSize, Cards = count });
        });
    }

    /// <summary>A pick order's names as card ids, for the draft to keep as far as its own cards go; a name no loaded card has is left out.</summary>
    private static List<string> CardIdsOf(ContentLibrary content, IReadOnlyList<string> pickOrder)
    {
        if (pickOrder.Count == 0)
        {
            return [];
        }

        Dictionary<string, string> ids = new(StringComparer.OrdinalIgnoreCase);
        foreach (CardDefinition card in content.Cards.InOrder())
        {
            ids.TryAdd(card.Name, card.Id.Value);
        }

        return [.. pickOrder.Select(name => ids.GetValueOrDefault(name)).OfType<string>()];
    }

    /// <summary>What every draft at one table is made with, whatever its format.</summary>
    private sealed record Made(string? Id, string Name, string? ParentId, IReadOnlyList<(string Id, string Name)> Players, ulong Seed, int MinimumDeck, IReadOnlyList<string> CubeCards);

    /// <summary>
    /// Makes a draft event and saves it: the cube's loaded cards, less those the format bans (<see cref="Banlist"/>),
    /// shuffled, the draft started from them by <paramref name="start"/> - given who goes first, chosen at random - and
    /// a ticket for each player. Each player's record notes they are in it.
    /// </summary>
    private static CreatedEvent Create(EventStore store, ContentLibrary content, Made setup, string kind, string format, TimeProvider? time,
        Func<List<string>, int, IRandom, (IPoolDraft Draft, DraftSettings Settings)> start)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(content);
        time ??= TimeProvider.System;

        if (setup.Players.Select(player => player.Id).Distinct(StringComparer.Ordinal).Count() != setup.Players.Count
            || setup.Players.Any(player => !EventStore.ValidId(player.Id) || string.IsNullOrWhiteSpace(player.Name)))
            throw new ArgumentException("Each player needs an id of their own and a name.", nameof(setup));
        if (setup.Id is not null && (!EventStore.ValidId(setup.Id) || store.LoadEvent(setup.Id) is not null))
            throw new ArgumentException($"'{setup.Id}' is not a new event id.", nameof(setup));
        if (setup.ParentId is not null && !EventStore.ValidId(setup.ParentId))
            throw new ArgumentException($"'{setup.ParentId}' is not an event id.", nameof(setup));

        Dictionary<string, CardId?> byName = new(StringComparer.OrdinalIgnoreCase);
        foreach (CardDefinition card in content.Cards.InOrder())
        {
            byName[card.Name] = byName.ContainsKey(card.Name) ? null : card.Id;
        }

        IReadOnlySet<string> banned = Banlist.For(format);
        List<string> cube = [];
        List<string> missing = [];
        foreach (string name in setup.CubeCards)
        {
            if (banned.Contains(name)) continue;
            if (byName.TryGetValue(name, out CardId? id) && id is { } found) cube.Add(found.Value);
            else missing.Add(name);
        }

        ulong seed = setup.Seed != 0 ? setup.Seed : BitConverter.ToUInt64(RandomNumberGenerator.GetBytes(8));
        var random = new Pcg32Random(seed);

        // Drawn from the cube at random, then shuffled again by the draft itself.
        for (int i = cube.Count - 1; i > 0; i--)
        {
            int j = random.NextInt(i + 1);
            (cube[i], cube[j]) = (cube[j], cube[i]);
        }

        int first = random.NextInt(setup.Players.Count) + 1;
        (IPoolDraft draft, DraftSettings settings) = start(cube, first, random);

        DateTimeOffset now = time.GetUtcNow();
        var record = new EventRecord
        {
            Id = setup.Id ?? EventStore.NewEventId(kind, now),
            Kind = kind,
            Name = setup.Name,
            ParentId = setup.ParentId,
            Stage = EventStage.Seating,
            Created = now,
            Updated = now,
            Seed = seed,
            MinimumDeck = setup.MinimumDeck,
            Draft = settings,
            Detail = "Waiting for the players to take their seats.",
        };

        Dictionary<string, string> tickets = new(StringComparer.Ordinal);
        for (int i = 0; i < setup.Players.Count; i++)
        {
            (string id, string name) = setup.Players[i];
            string ticket = NewTicket();
            record.Participants.Add(new EventParticipant { PlayerId = id, Name = name.Trim(), Seat = i + 1, TicketHashes = [EventStore.HashTicket(ticket)] });
            tickets[id] = ticket;
        }

        store.SaveDraft(record.Id, (object)draft);
        store.SaveEvent(record);
        if (record.ParentId is { } parentId && store.LoadEvent(parentId) is { } parent && !parent.Children.Contains(record.Id))
        {
            parent.Children.Add(record.Id);
            parent.Updated = now;
            store.SaveEvent(parent);
        }

        foreach (EventParticipant participant in record.Participants)
        {
            store.Enlist(participant.PlayerId, participant.Name, record, now);
        }

        return new CreatedEvent(record, missing, tickets);
    }

    /// <summary>How many of a cube's cards are loaded and not banned in the format (<see cref="Banlist"/>), so can be drafted.</summary>
    public static int Playable(ContentLibrary content, IReadOnlyList<string> cubeCards, string format)
    {
        ArgumentNullException.ThrowIfNull(content);
        ArgumentNullException.ThrowIfNull(cubeCards);
        HashSet<string> names = new(content.Cards.InOrder().Select(card => card.Name), StringComparer.OrdinalIgnoreCase);
        names.ExceptWith(Banlist.For(format));
        return cubeCards.Count(names.Contains);
    }

    /// <summary>Opens a saved draft event to be played at a table, wherever it had got to.</summary>
    public static DraftPod Open(EventStore store, string eventId, ContentLibrary content, TimeProvider? time = null, DraftTiming? timing = null)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(content);
        EventRecord record = store.LoadEvent(eventId) ?? throw new InvalidOperationException($"No event '{eventId}' in {store.Root}.");
        IPoolDraft draft = record.Kind switch
        {
            EventKinds.WinstonDraft => (IPoolDraft?)store.LoadDraft<WinstonDraft>(eventId),
            EventKinds.GridDraft => store.LoadDraft<GridDraft>(eventId),
            _ => throw new InvalidOperationException($"Event '{eventId}' is a {record.Kind}, not a draft played at one table."),
        } ?? throw new InvalidOperationException($"Event '{eventId}' has no draft saved.");
        return new DraftPod(store, record, draft, content, time ?? TimeProvider.System, timing ?? new DraftTiming());
    }

    /// <summary>
    /// A fresh ticket for a player, which takes them to their seat from any computer; the newest few stay good. What a
    /// host gives a player who has lost their link. Safe from any thread.
    /// </summary>
    public string IssueTicket(string playerId)
    {
        lock (_gate)
        {
            EventParticipant participant = _record.Participant(playerId) ?? throw new ArgumentException($"'{playerId}' is not in this event.", nameof(playerId));
            string ticket = NewTicket();
            participant.TicketHashes.Add(EventStore.HashTicket(ticket));
            if (participant.TicketHashes.Count > TicketsKept)
            {
                participant.TicketHashes.RemoveRange(0, participant.TicketHashes.Count - TicketsKept);
            }

            Save();
            return ticket;
        }
    }

    public string EventId => _record.Id;

    public string Name => _record.Name;

    /// <summary>Notes something the host keeps with the event (<see cref="EventRecord.HostData"/>), and saves it. Safe from any thread.</summary>
    public void Keep(string key, string value)
    {
        lock (_gate)
        {
            _record.HostData[key] = value;
            Save();
        }
    }

    /// <summary>Something the host kept with the event, or null.</summary>
    public string? Kept(string key)
    {
        lock (_gate) return _record.HostData.GetValueOrDefault(key);
    }

    /// <summary>
    /// Calls the event off, for good: it is saved as cancelled, and a table hosting it should be closed. Safe from any
    /// thread.
    /// </summary>
    public void Cancel(string detail)
    {
        lock (_gate)
        {
            if (!_record.IsLive) return;
            _record.Stage = EventStage.Cancelled;
            _record.Detail = detail;
            Save();
        }
    }

    /// <summary>
    /// A player drops from the event - the site's Drop. A draft of two has nothing left to play without them: while the
    /// game is being played they forfeit it, and the other player wins the match; before then the event is called off.
    /// Safe from any thread. Null once they have dropped; otherwise why not.
    /// </summary>
    public string? Drop(string playerId)
    {
        lock (_gate)
        {
            if (_record.Participant(playerId) is not { } participant) return "You are not in this draft.";
            if (!_record.IsLive) return "The draft is over: there is nothing to drop from.";
            if (participant.Dropped) return "You have already dropped.";

            participant.Dropped = true;
            string why = $"{participant.Name} dropped from the draft.";
            if (_record.Stage == EventStage.Playing && CurrentGame() is { Ending.Length: 0 })
            {
                _forfeitReasons[participant.Seat] = why;
                if (_dealtAt is not null)
                {
                    // At the table: the table ends the game against them (TakeForfeits), and GameEnded records it.
                    if (!_forfeited)
                    {
                        _forfeits.Add(participant.Seat);
                        _forfeited = true;
                    }

                    Save();
                    _version++;
                    return null;
                }

                // Not dealt again since a restart: decided here, as the table would have.
                _forfeited = true;
                GameEnded(false, [.. _record.Participants.Where(each => each.Seat != participant.Seat).Select(each => each.Seat)], why);
                _version++;
                return null;
            }

            _record.Stage = EventStage.Cancelled;
            _record.Deadline = null;
            _record.Detail = why;
            Save();
            _version++;
            return null;
        }
    }

    /// <summary>Whether a player has dropped from the event.</summary>
    public bool HasDropped(string playerId)
    {
        lock (_gate) return _record.Participant(playerId)?.Dropped == true;
    }

    public string? ForfeitReason(int seat)
    {
        lock (_gate) return _forfeitReasons.GetValueOrDefault(seat);
    }

    /// <summary>Where the event stands, in a line, as saved.</summary>
    public string Detail
    {
        get { lock (_gate) return _record.Detail; }
    }

    public int Seats => _record.Participants.Count;

    /// <summary>A copy of the record as it stands, for a host to read without reaching into the table's own.</summary>
    public EventStage Stage
    {
        get { lock (_gate) return _record.Stage; }
    }

    public IReadOnlyList<(string PlayerId, string Name, int Seat)> Players
    {
        get { lock (_gate) return [.. _record.Participants.Select(participant => (participant.PlayerId, participant.Name, participant.Seat))]; }
    }

    public TablePhase Phase
    {
        get
        {
            lock (_gate)
            {
                return _record.Stage switch
                {
                    EventStage.Drafting => TablePhase.Drafting,
                    EventStage.DeckBuilding => TablePhase.DeckBuilding,
                    EventStage.Complete or EventStage.Cancelled => TablePhase.Finished,
                    _ => TablePhase.Setup,
                };
            }
        }
    }

    public int SeatFor(string ticket)
    {
        if (string.IsNullOrEmpty(ticket))
        {
            return 0;
        }

        byte[] presented = Convert.FromHexString(EventStore.HashTicket(ticket));
        lock (_gate)
        {
            int seat = 0;
            foreach (EventParticipant participant in _record.Participants)
            {
                foreach (string hash in participant.TicketHashes)
                {
                    if (CryptographicOperations.FixedTimeEquals(Convert.FromHexString(hash), presented))
                    {
                        seat = participant.Seat;
                    }
                }
            }

            return seat;
        }
    }

    public string NameAt(int seat)
    {
        lock (_gate) return _record.AtSeat(seat)?.Name ?? $"Seat {seat}";
    }

    public string PlayerAt(int seat)
    {
        lock (_gate) return _record.AtSeat(seat)?.PlayerId ?? string.Empty;
    }

    public void Seated(int seat)
    {
        lock (_gate)
        {
            if (_record.AtSeat(seat) is not { } participant || participant.Seated)
            {
                return;
            }

            participant.Seated = true;
            if (_record.Stage == EventStage.Seating && _record.Participants.All(each => each.Seated))
            {
                _record.Stage = EventStage.Drafting;
                _record.Deadline = null;
                _record.Detail = _draft is GridDraft
                    ? $"{_record.AtSeat(_draft.Active)?.Name} picks first from the first grid."
                    : $"{_record.AtSeat(_draft.Active)?.Name} looks at the piles first.";
            }

            Save();
        }
    }

    public ServerMessage? ViewFor(int seat)
    {
        lock (_gate)
        {
            return _record.Stage switch
            {
                EventStage.Drafting => DraftView(seat),
                EventStage.DeckBuilding => DeckView(seat),
                _ => null,
            };
        }
    }

    public string? Receive(int seat, ClientMessage message)
    {
        lock (_gate)
        {
            return message switch
            {
                DraftPickMessage pick => Pick(seat, pick),
                BuildDeckMessage build => Build(seat, build),
                _ => "That is not something to do in this event.",
            };
        }
    }

    public bool ReadyToDeal
    {
        get
        {
            lock (_gate)
            {
                return _record.Stage switch
                {
                    EventStage.DeckBuilding => _record.Participants.All(participant => participant.Deck.Submitted),
                    // A round that was being played when the table stopped: dealt again.
                    EventStage.Playing => CurrentGame() is { Ending.Length: 0 },
                    _ => false,
                };
            }
        }
    }

    public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null)
    {
        lock (_gate)
        {
            bool next = _record.Stage == EventStage.DeckBuilding;
            int round = next ? _record.Round + 1 : _record.Round;
            ulong seed = next
                ? unchecked(_record.Seed + ((ulong)round * 0x9E3779B97F4A7C15UL))
                : CurrentGame()?.Seed ?? throw new InvalidOperationException("No game is waiting to be dealt.");

            // Dealt before anything is saved, so a deal that fails leaves the event where it was. A saved pool outlives
            // the content it was drafted from; a card that is no longer loaded is what the deal refuses, by name.
            List<RulesSeat> seats = [.. _record.Participants.OrderBy(participant => participant.Seat).Select((participant, index) => new RulesSeat(
                index < names.Count ? names[index] : participant.Name,
                [.. DeckOf(participant).Select(card => NameOf(card))],
                [.. participant.Pool.Where((_, place) => !participant.Deck.Main.Contains(place)).Select(id => NameOf(new CardId(id)))]))];

            // A Limited sideboard is the rest of the pool and any number of basic lands (CR 100.4b; Magic Tournament Rules).
            IRulesHost game = (rules ?? RulesHostFactory.Native).Create(seats, seed,
                new RulesGameOptions { BasicLandsOutside = [.. PoolDecks.BasicLands.Select(name => NameOf(_basics[name]))] });

            if (next)
            {
                _record.Stage = EventStage.Playing;
                _record.Round = round;
                var match = new MatchRecord
                {
                    Id = $"r{round}",
                    Round = round,
                    Players = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => participant.PlayerId)],
                };
                match.Games.Add(new GameRecord { Number = 1, Seed = seed, Started = _time.GetUtcNow() });
                _record.Matches.Add(match);
            }

            _record.Detail = $"Round {_record.Round} is being played.";
            _record.Deadline = null;
            Save();

            // Who is at the table as it is dealt; whoever is not has a while to come (DraftTiming.NoShow).
            DateTimeOffset now = _time.GetUtcNow();
            _dealtAt = now;
            _forfeits.Clear();
            _forfeited = false;
            _seenSinceDeal.Clear();
            _awaySince.Clear();
            foreach (int seat in _connected)
            {
                _seenSinceDeal[seat] = now;
            }

            return game;
        }
    }

    /// <summary>
    /// A pool's card as the rules name it: its id for the engine, which is its name for every card the content has
    /// (CR 201.2); the loaded card's name, which is how the native engine knows cards, or the id as it stands
    /// for one the content no longer has, which the host then refuses by name.
    /// </summary>
    private string NameOf(CardId card) =>
        _cards.TryGet(card, out CardDefinition? loaded) ? loaded!.Name : card.Value;

    public void DealFailed(string why)
    {
        lock (_gate)
        {
            // Back to building, every deck taken back, so the players can change what would not deal.
            if (_record.Stage == EventStage.Playing && _record.Matches.LastOrDefault() is { } match && match.Games.All(game => game.Ending.Length == 0))
            {
                _record.Matches.Remove(match);
                _record.Round--;
            }

            if (_record.Stage is EventStage.Playing or EventStage.DeckBuilding)
            {
                _record.Stage = EventStage.DeckBuilding;
                _record.Deadline = _time.GetUtcNow() + _timing.DeckBuilding;
                _dealtAt = null;
                foreach (EventParticipant participant in _record.Participants)
                {
                    participant.Deck.Submitted = false;
                }
            }

            _record.Detail = $"{DealFailedLine}: {why}";
            Save();
        }
    }

    public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail)
    {
        lock (_gate)
        {
            if (!_record.IsLive || CurrentGame() is not { } game || _record.Matches.LastOrDefault() is not { } match)
            {
                return;
            }

            _dealtAt = null;
            List<string> winners = [.. winningSeats.Select(seat => _record.AtSeat(seat)?.PlayerId).OfType<string>()];
            game.Ended = _time.GetUtcNow();
            game.Ending = abandoned ? "abandoned" : winners.Count == 0 ? "draw" : "won";
            game.Winners = winners;
            game.Detail = detail;

            // One game a match, one round, for now; a match of three games, or more rounds, is decided here.
            match.Forfeit = _forfeited;
            match.Winners = winners;
            match.Complete = true;
            _record.Stage = EventStage.Complete;
            _record.Detail = abandoned
                ? $"Round {_record.Round} was abandoned."
                : winners.Count == 0
                    ? $"Round {_record.Round} was a draw."
                    : $"{string.Join(" and ", winners.Select(id => _record.Participant(id)!.Name))} won round {_record.Round}.";
            if (!abandoned && _forfeitReasons.Count > 0) _record.Detail += " " + string.Join(" ", _forfeitReasons.Values);
            Save();
        }
    }

    public int Version
    {
        get { lock (_gate) return _version; }
    }

    /// <summary>
    /// Lets time pass: players who did not take their seats call the event off, a pick clock that has run out picks for
    /// its player, deck building's clock submits every deck as it stands, and a player not at the game forfeits it.
    /// </summary>
    public void Tick(DateTimeOffset now)
    {
        lock (_gate)
        {
            switch (_record.Stage)
            {
                case EventStage.Seating when _record.Deadline is { } seating && now >= seating:
                    List<string> absent = [.. _record.Participants.Where(participant => !participant.Seated).Select(participant => participant.Name)];
                    _record.Stage = EventStage.Cancelled;
                    _record.Deadline = null;
                    _record.Detail = $"{string.Join(" and ", absent)} did not take {(absent.Count == 1 ? "their seat" : "their seats")} in time.";
                    Save();
                    _version++;
                    return;
                case EventStage.DeckBuilding when _record.Deadline is { } building && now >= building:
                    foreach (EventParticipant participant in _record.Participants.Where(participant => !participant.Deck.Submitted))
                    {
                        participant.Deck = PoolDecks.Submitted(participant.Pool, participant.Deck, _record.MinimumDeck, id => Card(id).Colors);
                    }

                    _record.Deadline = null;
                    Save();
                    _version++;
                    return;
                case EventStage.Playing:
                    Absent(now);
                    return;
            }

            if (PickBy() is not { } by || now < by)
            {
                return;
            }

            switch (_draft)
            {
                case GridDraft grid:
                    grid.TakeForClock(_chosenLine is { } chosen && chosen.Decision == grid.Decision ? chosen.Line : null);
                    break;
                case WinstonDraft winston:
                    winston.TakeForClock();
                    break;
            }

            Picked();
            _version++;
        }
    }

    /// <summary>
    /// While the game is played: a seat not at the table since it was dealt for <see cref="DraftTiming.NoShow"/>, or gone
    /// from it for <see cref="DraftTiming.Away"/>, forfeits.
    /// </summary>
    private void Absent(DateTimeOffset now)
    {
        if (_dealtAt is not { } dealt || _forfeited || CurrentGame() is not { Ending.Length: 0 })
        {
            return;
        }

        List<int> gone = [.. _record.Participants.Select(participant => participant.Seat).Where(seat => !_seenSinceDeal.ContainsKey(seat)
            ? now - dealt >= _timing.NoShow
            : _awaySince.TryGetValue(seat, out DateTimeOffset away) && now - away >= _timing.Away)];
        if (gone.Count == 0)
        {
            return;
        }

        // Marked on the match only as the game is ended for it (GameEnded), so a restart in between deals it clean.
        _forfeits.AddRange(gone);
        _forfeited = true;
    }

    public IReadOnlyList<int> TakeForfeits()
    {
        lock (_gate)
        {
            if (_forfeits.Count == 0) return [];
            List<int> forfeits = [.. _forfeits];
            _forfeits.Clear();
            return forfeits;
        }
    }

    public void Connected(int seat, bool connected)
    {
        lock (_gate)
        {
            DateTimeOffset now = _time.GetUtcNow();
            if (connected)
            {
                _connected.Add(seat);
                _awaySince.Remove(seat);
                if (_dealtAt is not null) _seenSinceDeal.TryAdd(seat, now);
            }
            else
            {
                _connected.Remove(seat);
                _awaySince[seat] = now;
            }
        }
    }

    public TimeSpan? ClockAtDeal(int seat)
    {
        lock (_gate)
        {
            if (_timing.MatchClock <= TimeSpan.Zero || _record.Matches.LastOrDefault() is not { } match || _record.AtSeat(seat) is not { } participant) return null;
            return match.ClockMs.TryGetValue(participant.PlayerId, out long left) ? TimeSpan.FromMilliseconds(left) : _timing.MatchClock;
        }
    }

    public void ClockUsed(int seat, TimeSpan left)
    {
        lock (_gate)
        {
            if (_record.Matches.LastOrDefault() is not { } match || _record.AtSeat(seat) is not { } participant) return;
            match.ClockMs[participant.PlayerId] = Math.Max(0, (long)left.TotalMilliseconds);
            Save();
        }
    }

    public void ClocksKept(IReadOnlyDictionary<int, TimeSpan> left)
    {
        lock (_gate)
        {
            if (_record.Matches.LastOrDefault() is not { } match) return;
            foreach ((int seat, TimeSpan each) in left)
            {
                if (_record.AtSeat(seat) is { } participant) match.ClockMs[participant.PlayerId] = Math.Max(0, (long)each.TotalMilliseconds);
            }

            Save();
        }
    }

    public void ClockRanOut(int seat) => ClocksRanOut([seat]);

    /// <summary>
    /// The players whose clocks ran out: one alone loses the match (<see cref="MatchRecord.TimedOut"/>); both together, and
    /// neither is preferred - the game ends as a draw, and with it the match.
    /// </summary>
    public void ClocksRanOut(IReadOnlyList<int> seats)
    {
        lock (_gate)
        {
            List<string> out_ = [.. seats.Select(seat => _record.AtSeat(seat)?.PlayerId).OfType<string>().Distinct()];
            if (out_.Count == 1 && _record.Matches.LastOrDefault() is { } match) match.TimedOut = out_[0];
        }
    }

    /// <summary>What the pick clock counts: a grid draft's picks, a Winston draft's turns.</summary>
    private int Turn => _draft switch
    {
        GridDraft grid => grid.Decision,
        WinstonDraft winston => winston.Turns,
        _ => 0,
    };

    /// <summary>
    /// When the pick being made runs out, starting its clock if it has none yet; null when nothing is being picked or
    /// the draft has no clock.
    /// </summary>
    private DateTimeOffset? PickBy()
    {
        if (_record.Stage != EventStage.Drafting || _draft.IsOver || _pickTime <= TimeSpan.Zero)
        {
            return null;
        }

        int turn = Turn;
        if (_clock is not { } clock || clock.Turn != turn)
        {
            _clock = clock = (turn, _time.GetUtcNow() + _pickTime);
        }

        return clock.By;
    }

    /// <summary>How long the pick being made has left, for the seats' views; zero for none.</summary>
    private int PickLeftMs() =>
        PickBy() is { } by ? Math.Max(1, (int)Math.Min(int.MaxValue, Math.Max(0, (by - _time.GetUtcNow()).TotalMilliseconds))) : 0;

    /// <summary>The game of the current round still to finish, or the last one dealt; null before any.</summary>
    private GameRecord? CurrentGame() => _record.Matches.LastOrDefault()?.Games.LastOrDefault();

    private string? Pick(int seat, DraftPickMessage pick)
    {
        if (_record.Stage != EventStage.Drafting)
        {
            return "The draft is not on.";
        }

        try
        {
            switch (_draft)
            {
                case GridDraft grid when pick.Take:
                    grid.Take(seat, pick.Decision, pick.Card);
                    break;
                case GridDraft grid:
                    // Not a pass: the line the active player has chosen but not taken, which the clock takes if it runs out.
                    if (seat != grid.Active || pick.Decision != grid.Decision) return "That pick was for a grid that has since changed.";
                    if (!grid.CanTake(pick.Card)) return "That is not a row or a column of the grid with anything left in it.";
                    _chosenLine = (grid.Decision, pick.Card);
                    return null;
                case WinstonDraft winston when pick.Take:
                    winston.Take(seat, pick.Decision);
                    break;
                case WinstonDraft winston:
                    winston.Pass(seat, pick.Decision);
                    break;
            }
        }
        catch (InvalidOperationException refused)
        {
            return refused.Message;
        }

        Picked();
        return null;
    }

    /// <summary>After any pick, the player's or the clock's: the draft saved first, then the event.</summary>
    private void Picked()
    {
        // The draft first: it is the authority on the pools until it is over (see Reconcile).
        _store.SaveDraft(_record.Id, (object)_draft);
        foreach (EventParticipant participant in _record.Participants)
        {
            participant.Pool = [.. _draft.PoolOf(participant.Seat)];
        }

        if (_draft.IsOver)
        {
            FinishDraft();
        }

        Save();
    }

    private void FinishDraft()
    {
        _record.Stage = EventStage.DeckBuilding;
        _record.Deadline = _time.GetUtcNow() + _timing.DeckBuilding;
        _record.Detail = "The draft is over. Build your decks.";
        foreach (EventParticipant each in _record.Participants)
        {
            // Everything drafted starts in the main deck: most of it usually ends up there.
            each.Deck = new EventDeck { Main = [.. Enumerable.Range(0, each.Pool.Count)] };
        }
    }

    private string? Build(int seat, BuildDeckMessage build)
    {
        if (_record.Stage != EventStage.DeckBuilding)
        {
            return "Decks are not being built.";
        }

        EventParticipant participant = _record.AtSeat(seat)!;

        // Dealt with, applied or not, so the client knows its own changes have all come back.
        _changes[seat] = build.Change;

        (EventDeck? deck, string? refusal) = PoolDecks.Built(participant.Deck, participant.Pool.Count, build, _basics, _record.MinimumDeck);
        if (deck is null)
        {
            return refusal;
        }

        participant.Deck = deck;
        Save();
        return null;
    }

    private List<CardId> DeckOf(EventParticipant participant) => PoolDecks.Cards(participant.Pool, participant.Deck, _basics);

    private DraftMessage DraftView(int seat) => _draft switch
    {
        GridDraft grid => GridView(seat, grid),
        WinstonDraft winston => WinstonView(seat, winston),
        _ => throw new InvalidOperationException("A draft of a format this table does not know."),
    };

    /// <summary>
    /// A Winston draft as one seat sees it: the piles' sizes and the stack's for everyone, the pile being looked at - and
    /// where its cards stand in the pick order - only for the seat looking at it, and only that seat's own pool.
    /// </summary>
    private DraftMessage WinstonView(int seat, WinstonDraft draft)
    {
        EventParticipant? me = _record.AtSeat(seat);
        // Only the seat looking at a pile sees it; nobody sees the stack or the piles not being looked at.
        List<string> seen = seat == draft.Active && draft.Looking >= 0 ? draft.Piles[draft.Looking] : [];
        return new DraftMessage
        {
            EventId = _record.Id,
            Format = DraftFormats.Winston,
            Active = draft.Active,
            Looking = draft.Looking,
            Decision = draft.Decision,
            Piles = [.. draft.Piles.Select(pile => pile.Count)],
            Stack = draft.Stack.Count,
            Pile = [.. seen.Select(Card)],
            PileRanks = draft.PickOrder.Count == 0 ? [] : [.. seen.Select(draft.Rank)],
            CanPass = seat == draft.Active && draft.CanPass,
            Pool = me is null ? [] : [.. me.Pool.Select(Card)],
            Seats = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new DraftSeatView
            {
                Seat = participant.Seat, Name = participant.Name, Drafted = participant.Pool.Count,
            })],
            Recent = [.. draft.History.TakeLast(8).Select(action => Said(action, seat))],
            Total = _record.Draft?.Cards ?? 0,
            PickLeftMs = PickLeftMs(),
        };
    }

    /// <summary>
    /// A grid draft, the same for both seats but for the pool: the grid face up - a place whose card is gone an unknown
    /// card - whose pick it is, and what each player took.
    /// </summary>
    private DraftMessage GridView(int seat, GridDraft draft)
    {
        EventParticipant? me = _record.AtSeat(seat);
        List<DraftLine> recent = [.. draft.History.TakeLast(8).Select(pick => Said(pick, seat))];
        return new DraftMessage
        {
            EventId = _record.Id,
            Format = DraftFormats.Grid,
            Active = draft.Active,
            Decision = draft.Decision,
            Pile = [.. draft.Grid.Select(card => card is null ? new ObjectView { Known = false } : Card(card))],
            PileRanks = draft.PickOrder.Count == 0 ? [] : [.. draft.Grid.Select(card => card is null ? draft.PickOrder.Count : draft.Rank(card))],
            PackNumber = draft.Pack,
            Packs = draft.Packs,
            PickNumber = draft.IsOver ? 0 : draft.PickOfPack,
            PackSize = GridDraft.PackSize,
            Pool = me is null ? [] : [.. me.Pool.Select(Card)],
            Seats = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new DraftSeatView
            {
                Seat = participant.Seat, Name = participant.Name, Drafted = participant.Pool.Count,
            })],
            Recent = [.. recent.Select(line => line.Text)],
            RecentCards = [.. recent.Select(line => line.Cards)],
            LastLine = draft.History.Count == 0 ? -1 : draft.History[^1].Line,
            Total = _record.Draft?.Cards ?? 0,
            PickLeftMs = PickLeftMs(),
        };
    }

    /// <summary>One pick of a grid draft in words, for the seat reading it: "You took row 2 of grid 3: A, B and C."</summary>
    private DraftLine Said(GridPick pick, int reader) => DraftLine.Write(name =>
    {
        string who = pick.Seat == reader ? "You" : _record.AtSeat(pick.Seat)?.Name ?? $"Seat {pick.Seat}";
        List<string> names = [.. pick.Cards.Select(card => name(Card(card)))];
        string cards = names.Count <= 1 ? string.Concat(names) : $"{string.Join(", ", names[..^1])} and {names[^1]}";
        return $"{who} took {GridDraft.Named(pick.Line)} of grid {pick.Pack}{(pick.Auto ? " (time ran out)" : string.Empty)}: {cards}.";
    });

    /// <summary>One decision of the draft in words, for the seat reading it.</summary>
    private string Said(WinstonAction action, int reader)
    {
        string who = action.Seat == reader ? "You" : _record.AtSeat(action.Seat)?.Name ?? $"Seat {action.Seat}";
        string pile = $"pile {action.Pile + 1}{(action.Auto ? " (time ran out)" : string.Empty)}";
        if (action.Blind)
        {
            return $"{who} passed {pile} and took the top card of the stack.";
        }

        return action.Took == 0
            ? $"{who} passed {pile}."
            : $"{who} took {pile} ({action.Took} card{(action.Took == 1 ? string.Empty : "s")}).";
    }

    private DeckBuildingMessage DeckView(int seat)
    {
        EventParticipant me = _record.AtSeat(seat)!;
        return new DeckBuildingMessage
        {
            EventId = _record.Id,
            Pool = [.. me.Pool.Select(Card)],
            Main = [.. me.Deck.Main],
            Basics = [.. BasicLands.Select(name => new BasicLandView { Name = name, Count = me.Deck.Basics.GetValueOrDefault(name), Card = Card(_basics[name].Value) })],
            Minimum = _record.MinimumDeck,
            Submitted = me.Deck.Submitted,
            Change = _changes.GetValueOrDefault(seat),
            LeftMs = _record.Deadline is { } until ? Math.Max(1, (int)Math.Min(int.MaxValue, Math.Max(0, (until - _time.GetUtcNow()).TotalMilliseconds))) : 0,
            Seats = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new DeckBuilderSeatView
            {
                Seat = participant.Seat, Name = participant.Name, Submitted = participant.Deck.Submitted,
            })],
        };
    }

    private ObjectView Card(string id) =>
        _oracle.Oracle(new CardId(id)) ?? new ObjectView { Known = true, CardId = id, Name = id };

    private void Save()
    {
        _record.Updated = _time.GetUtcNow();
        _store.SaveEvent(_record);
    }

    private static string NewTicket() => Convert.ToHexString(RandomNumberGenerator.GetBytes(32));
}
