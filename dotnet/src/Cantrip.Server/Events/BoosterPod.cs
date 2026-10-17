using System.Security.Cryptography;
using Cantrip.Cards.Model;
using Cantrip.Core.Random;
using Cantrip.Datasets;
using Cantrip.Projection;
using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>What a host says about a booster draft it is creating.</summary>
public sealed class BoosterDraftSetup
{
    /// <summary>The event's id, for a host that named it before making it; null makes one.</summary>
    public string? Id { get; init; }

    public required string Name { get; init; }

    /// <summary>The cube's name, for the record.</summary>
    public required string Cube { get; init; }

    /// <summary>The cube's cards by name. A name no loaded card has is left out, and said so in the result.</summary>
    public required IReadOnlyList<string> CubeCards { get; init; }

    /// <summary>
    /// The cube's cards best first (<see cref="CubeList.PickOrder"/>): what the pick clock takes for a player, and what a
    /// bot is told to take. None leaves the clock taking at random.
    /// </summary>
    public IReadOnlyList<string> PickOrder { get; init; } = [];

    public int Packs { get; init; } = 3;

    public int PackSize { get; init; } = 15;

    /// <summary>How many Swiss rounds are played once the decks are built.</summary>
    public int Rounds { get; init; } = 3;

    /// <summary>The players by id and name, seated round the table in this order.</summary>
    public required IReadOnlyList<(string Id, string Name)> Players { get; init; }

    public ulong Seed { get; init; }

    public int MinimumDeck { get; init; } = 40;

    /// <summary>The event this draft is part of - a league - or null.</summary>
    public string? ParentId { get; init; }
}

/// <summary>
/// How long everything in a booster draft is given. Not saved with the event: a host decides, and a test gives the
/// clocks seconds rather than minutes.
/// </summary>
public sealed class BoosterTiming
{
    /// <summary>The pick clock, per card left in the pack (<see cref="BoosterDraft.SecondsFor"/>).</summary>
    public int SecondsPerCard { get; init; } = 5;

    /// <summary>
    /// How long a player with the table open is waited on, once their pick clock runs out, to say which card they had
    /// chosen (<see cref="DraftMessage.PickDue"/>): a round trip, with room to spare.
    /// </summary>
    public TimeSpan PickAnswer { get; init; } = TimeSpan.FromSeconds(2);

    /// <summary>How long decks are built for once the draft is over, before each deck as it stands is submitted.</summary>
    public TimeSpan DeckBuilding { get; init; } = TimeSpan.FromMinutes(10);

    /// <summary>How long a player has to change their deck from their sideboard between the games of a match.</summary>
    public TimeSpan Sideboarding { get; init; } = TimeSpan.FromMinutes(3);

    /// <summary>How long the first round's pairings are shown before its matches begin.</summary>
    public TimeSpan FirstRound { get; init; } = TimeSpan.FromSeconds(15);

    /// <summary>How long after a round's last match ends the next round begins.</summary>
    public TimeSpan BetweenRounds { get; init; } = TimeSpan.FromMinutes(1);

    /// <summary>How long a player has to come to their match's table before it is forfeited to their opponent.</summary>
    public TimeSpan NoShow { get; init; } = TimeSpan.FromMinutes(3);

    /// <summary>How long a player may be gone from their match's table before it is forfeited to their opponent.</summary>
    public TimeSpan Away { get; init; } = TimeSpan.FromMinutes(5);

    /// <summary>
    /// Each player's match clock: a chess clock across every game of a match, running while the game waits on them -
    /// priority, the mulligans, every choice. A player whose clock runs out loses the match. Zero plays without one.
    /// </summary>
    public TimeSpan MatchClock { get; init; } = TimeSpan.FromMinutes(25);

    /// <summary>How long the end of a game is shown before its players are taken on to the next game, or back to the pairings.</summary>
    public TimeSpan AfterGame { get; init; } = TimeSpan.FromSeconds(6);
}

/// <summary>
/// A booster draft of a pod - eight, classically - and the Swiss rounds played with what was drafted. The pod's own
/// table is where the players draft (<see cref="BoosterDraft"/>), build their decks, and come back to between rounds to
/// see the pairings, how the other matches are going and, at the end, the final standings. Each game of each match is
/// played at a table of its own (<see cref="MatchGame"/>), which a host opens when the pod asks for one
/// (<see cref="TablesToOpen"/>) and closes when it is done with (<see cref="TablesDone"/>): a game's players are moved
/// there and back by <see cref="TableMoveMessage"/>, and their pod ticket always brings them to wherever they should be.
/// <para>
/// Nothing waits for a player who is not there. The pick clock takes a card for them - asking first, of a player who is
/// there, for the card they had chosen and waiting a moment for it (<see cref="BoosterTiming.PickAnswer"/>) - deck
/// building's clock submits
/// their deck as it stands (made up to the minimum with basic lands), and a player who does not come to their match, or
/// is gone from it for too long, forfeits it and is dropped from the rounds after.
/// </para>
/// <para>
/// The record (<see cref="EventRecord"/>) and the draft are saved after everything that changes them. A game cut off by
/// a restart is dealt again from its start, from the same seed; every clock starts again from the restart.
/// </para>
/// </summary>
public sealed class BoosterPod : ITableEvent
{
    private const int TicketsKept = 8;

    private readonly EventStore _store;
    private readonly EventRecord _record;
    private readonly BoosterDraft _draft;
    private readonly CardCatalog _cards;
    private readonly CardOracle _oracle;
    private readonly Dictionary<string, CardId> _basics;
    private readonly TimeProvider _time;

    /// <summary>The tables of the games being played or about to be, by match and game number.</summary>
    private readonly Dictionary<string, MatchGame> _games = new(StringComparer.Ordinal);

    /// <summary>Seats to move off the pod's table - to their match - with what to tell them.</summary>
    private readonly List<(int Seat, ServerMessage Farewell)> _vacated = [];

    private readonly HashSet<int> _connected = [];

    /// <summary>The seats whose pick clock has run out and who have been asked for the card chosen: which pick, and until when.</summary>
    private readonly Dictionary<int, (int Pick, DateTimeOffset By)> _asked = [];
    private readonly Dictionary<int, int> _changes = [];
    private int _version;
    private Uri? _address;

    private BoosterPod(EventStore store, EventRecord record, BoosterDraft draft, ContentLibrary content, TimeProvider time, BoosterTiming timing)
    {
        _store = store;
        _record = record;
        _draft = draft;
        _time = time;
        Timing = timing;
        _cards = content.Catalog();
        _oracle = new CardOracle(_cards);
        _basics = PoolDecks.Basics(content);

        // Read from the cards as they are loaded now, so a draft saved before a card had its draft ability has it too.
        _draft.AdditionalPicks = AdditionalPicks(content, [.. _draft.Unopened.SelectMany(round => round.SelectMany(pack => pack)),
            .. _draft.Queues.SelectMany(queue => queue.SelectMany(pack => pack.Cards)), .. _draft.Pools.SelectMany(pool => pool)]);
        TakeUp();
    }

    /// <summary>The cards of these with Cogwork Librarian's draft ability (<see cref="BoosterDraft.AdditionalPicks"/>).</summary>
    private static List<string> AdditionalPicks(ContentLibrary content, IEnumerable<string> cards) =>
        [.. cards.Distinct(StringComparer.Ordinal).Where(id =>
            content.Cards.TryGet(new CardId(id), out CardDefinition? card) && card!.DraftAbilities.Contains(DraftAbility.AdditionalPickPutThisIntoPack))];

    /// <summary>Everything the pod and its games' tables do is done under this.</summary>
    internal object Gate { get; } = new();

    internal BoosterTiming Timing { get; }

    internal EventRecord Record => _record;

    internal DateTimeOffset Now => _time.GetUtcNow();

    /// <summary>
    /// Makes a booster draft and saves it, ready for its players to sit down: the cube's loaded cards, enough of them
    /// drawn at random for every pack, and a ticket for each player. Each player's record notes they are in it.
    /// </summary>
    public static CreatedEvent Create(EventStore store, ContentLibrary content, BoosterDraftSetup setup, TimeProvider? time = null)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(content);
        ArgumentNullException.ThrowIfNull(setup);
        time ??= TimeProvider.System;

        if (setup.Players.Count is < 2 or > 8)
            throw new ArgumentException("A booster draft pod has two to eight players.", nameof(setup));
        if (setup.Players.Select(player => player.Id).Distinct(StringComparer.Ordinal).Count() != setup.Players.Count
            || setup.Players.Any(player => !EventStore.ValidId(player.Id) || string.IsNullOrWhiteSpace(player.Name)))
            throw new ArgumentException("Each player needs an id of their own and a name.", nameof(setup));
        if (setup.Id is not null && (!EventStore.ValidId(setup.Id) || store.LoadEvent(setup.Id) is not null))
            throw new ArgumentException($"'{setup.Id}' is not a new event id.", nameof(setup));
        if (setup.Rounds is < 1 or > 8)
            throw new ArgumentException("A booster draft plays one to eight rounds.", nameof(setup));

        (List<string> cube, List<string> missing) = Loaded(content, setup.CubeCards);
        int count = setup.Players.Count * setup.Packs * setup.PackSize;
        if (count > cube.Count)
            throw new ArgumentException($"The cube has {cube.Count} cards that can be played; {setup.Players.Count} players opening {setup.Packs} packs of {setup.PackSize} need {count}.", nameof(setup));

        ulong seed = setup.Seed != 0 ? setup.Seed : BitConverter.ToUInt64(RandomNumberGenerator.GetBytes(8));
        DateTimeOffset now = time.GetUtcNow();
        BoosterDraft draft = BoosterDraft.Start(cube, setup.Players.Count, setup.Packs, setup.PackSize, new Pcg32Random(seed), seed, now,
            pickOrder: setup.PickOrder, additionalPicks: AdditionalPicks(content, cube));

        var record = new EventRecord
        {
            Id = setup.Id ?? EventStore.NewEventId(EventKinds.BoosterDraft, now),
            Kind = EventKinds.BoosterDraft,
            Name = setup.Name,
            ParentId = setup.ParentId,
            Stage = EventStage.Seating,
            Created = now,
            Updated = now,
            Seed = seed,
            MinimumDeck = setup.MinimumDeck,
            Draft = new DraftSettings
            {
                Format = DraftFormats.Booster, Cube = setup.Cube, Cards = count, Packs = setup.Packs, PackSize = setup.PackSize, Rounds = setup.Rounds,
            },
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

        store.SaveDraft(record.Id, draft);
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

    /// <summary>How many cards a cube has that can be drafted: the ones loaded.</summary>
    public static int Playable(ContentLibrary content, IReadOnlyList<string> cubeCards) => Loaded(content, cubeCards).Cube.Count;

    /// <summary>Opens a saved booster draft to be played, wherever it had got to.</summary>
    public static BoosterPod Open(EventStore store, string eventId, ContentLibrary content, TimeProvider? time = null, BoosterTiming? timing = null)
    {
        ArgumentNullException.ThrowIfNull(store);
        ArgumentNullException.ThrowIfNull(content);
        EventRecord record = store.LoadEvent(eventId) ?? throw new InvalidOperationException($"No event '{eventId}' in {store.Root}.");
        if (record.Kind != EventKinds.BoosterDraft)
            throw new InvalidOperationException($"Event '{eventId}' is a {record.Kind}, not a booster draft.");
        BoosterDraft draft = store.LoadDraft<BoosterDraft>(eventId) ?? throw new InvalidOperationException($"Event '{eventId}' has no draft saved.");
        return new BoosterPod(store, record, draft, content, time ?? TimeProvider.System, timing ?? new BoosterTiming());
    }

    // --- For the host. ---------------------------------------------------------------------------------------------

    public string EventId => _record.Id;

    public string Name => _record.Name;

    public int Seats => _record.Participants.Count;

    public EventStage Stage
    {
        get { lock (Gate) return _record.Stage; }
    }

    /// <summary>Where the event stands, in a line.</summary>
    public string Detail
    {
        get { lock (Gate) return _record.Detail; }
    }

    public IReadOnlyList<(string PlayerId, string Name, int Seat)> Players
    {
        get { lock (Gate) return [.. _record.Participants.Select(participant => (participant.PlayerId, participant.Name, participant.Seat))]; }
    }

    /// <summary>The pod's own table is open at this address: where its players are sent back to after their games.</summary>
    public void Hosted(Uri address)
    {
        lock (Gate) _address = address;
    }

    /// <summary>The games whose tables are wanted and not open yet: the host opens each and says where (<see cref="MatchGame.Opened"/>).</summary>
    public IReadOnlyList<MatchGame> TablesToOpen()
    {
        lock (Gate) return [.. _games.Values.Where(game => game.Address is null && !game.Done)];
    }

    /// <summary>The games whose tables are done with - everyone has gone on - for the host to close, each said once.</summary>
    public IReadOnlyList<MatchGame> TablesDone()
    {
        lock (Gate)
        {
            List<MatchGame> done = [.. _games.Values.Where(game => game.Done)];
            foreach (MatchGame game in done)
            {
                _games.Remove(game.Key);
            }

            return done;
        }
    }

    /// <summary>A fresh ticket to the pod for a player, which takes them to wherever they should be. Safe from any thread.</summary>
    public string IssueTicket(string playerId)
    {
        lock (Gate)
        {
            EventParticipant participant = _record.Participant(playerId) ?? throw new ArgumentException($"'{playerId}' is not in this event.", nameof(playerId));
            string ticket = AddTicket(participant);
            Save();
            return ticket;
        }
    }

    /// <summary>Notes something the host keeps with the event (<see cref="EventRecord.HostData"/>), and saves it.</summary>
    public void Keep(string key, string value)
    {
        lock (Gate)
        {
            _record.HostData[key] = value;
            Save();
        }
    }

    public string? Kept(string key)
    {
        lock (Gate) return _record.HostData.GetValueOrDefault(key);
    }

    /// <summary>Calls the event off, for good.</summary>
    public void Cancel(string detail)
    {
        lock (Gate)
        {
            if (!_record.IsLive) return;
            _record.Stage = EventStage.Cancelled;
            _record.Detail = detail;
            foreach (MatchGame game in _games.Values) game.Close();
            Changed();
            Save();
        }
    }

    /// <summary>The draft log so far: every pick in order. A copy.</summary>
    public IReadOnlyList<BoosterPick> Picks
    {
        get { lock (Gate) return [.. _draft.Log]; }
    }

    // --- The pod's table. -------------------------------------------------------------------------------------------

    public TablePhase Phase
    {
        get
        {
            lock (Gate)
            {
                return _record.Stage switch
                {
                    EventStage.Drafting => TablePhase.Drafting,
                    EventStage.DeckBuilding => TablePhase.DeckBuilding,
                    EventStage.Playing or EventStage.Complete => TablePhase.Pairings,
                    EventStage.Cancelled => TablePhase.Finished,
                    _ => TablePhase.Setup,
                };
            }
        }
    }

    public int SeatFor(string ticket)
    {
        if (string.IsNullOrEmpty(ticket)) return 0;
        lock (Gate) return Matching(ticket, _record.Participants.Select(participant => (participant.Seat, (IEnumerable<string>)participant.TicketHashes)));
    }

    public string NameAt(int seat)
    {
        lock (Gate) return _record.AtSeat(seat)?.Name ?? $"Seat {seat}";
    }

    public string PlayerAt(int seat)
    {
        lock (Gate) return _record.AtSeat(seat)?.PlayerId ?? string.Empty;
    }

    public void Seated(int seat)
    {
        lock (Gate)
        {
            if (_record.AtSeat(seat) is not { } participant || participant.Seated) return;
            participant.Seated = true;
            if (_record.Stage == EventStage.Seating && _record.Participants.All(each => each.Seated))
            {
                _record.Stage = EventStage.Drafting;
                _record.Detail = "Pack 1 is open.";
                _draft.Resume(Now);
                _store.SaveDraft(_record.Id, _draft);
                Changed();
            }

            Save();
        }
    }

    public void Connected(int seat, bool connected)
    {
        lock (Gate)
        {
            if (!connected)
            {
                _connected.Remove(seat);
                return;
            }

            _connected.Add(seat);
            SendToGame(seat);
        }
    }

    public int Version
    {
        get { lock (Gate) return _version; }
    }

    public ServerMessage? ViewFor(int seat)
    {
        lock (Gate)
        {
            return _record.Stage switch
            {
                EventStage.Drafting => DraftView(seat),
                EventStage.DeckBuilding => DeckView(seat),
                EventStage.Playing or EventStage.Complete => PairingsView(),
                _ => null,
            };
        }
    }

    public string? Receive(int seat, ClientMessage message)
    {
        lock (Gate)
        {
            return message switch
            {
                DraftPickMessage pick => Pick(seat, pick),
                BuildDeckMessage build => Build(seat, build),
                DropMessage => Drop(seat),
                _ => "That is not something to do in this event.",
            };
        }
    }

    /// <summary>The pod's own table deals nothing: its games are played at their own tables.</summary>
    public bool ReadyToDeal => false;

    public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null) =>
        throw new InvalidOperationException("A pod's table deals no game of its own.");

    public void DealFailed(string why)
    {
    }

    public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail)
    {
    }

    public void Tick(DateTimeOffset now)
    {
        lock (Gate)
        {
            switch (_record.Stage)
            {
                case EventStage.Drafting when _draft.Tick(now, seat => Asking(seat, now)):
                    Picked();
                    break;
                case EventStage.DeckBuilding when _record.Deadline is { } deadline && now >= deadline:
                    foreach (EventParticipant participant in _record.Participants.Where(participant => !participant.Deck.Submitted))
                    {
                        participant.Deck = PoolDecks.Submitted(participant.Pool, participant.Deck, _record.MinimumDeck, ColorsOf);
                    }

                    DecksBuilt();
                    break;
                case EventStage.Playing when _record.Deadline is { } starts && now >= starts:
                    BeginRound();
                    break;
            }
        }
    }

    public IReadOnlyList<(int Seat, ServerMessage Farewell)> TakeVacated()
    {
        lock (Gate)
        {
            if (_vacated.Count == 0) return [];
            List<(int, ServerMessage)> vacated = [.. _vacated];
            _vacated.Clear();
            return vacated;
        }
    }

    // --- The draft and deck building. -------------------------------------------------------------------------------

    private string? Pick(int seat, DraftPickMessage pick)
    {
        if (_record.Stage != EventStage.Drafting) return "The draft is not on.";
        try
        {
            if (pick.Take)
            {
                _draft.Take(seat, pick.Decision, [pick.Card, .. pick.Also ?? []], Now);
            }
            else
            {
                // Asked for the card chosen, with none chosen: the clock's card, now rather than when the wait is up.
                _draft.TakeForClock(seat, pick.Decision, Now);
            }
        }
        catch (InvalidOperationException refused)
        {
            return refused.Message;
        }

        Picked();
        return null;
    }

    /// <summary>
    /// Whether to wait on a seat whose pick clock has run out rather than take the clock's card for it: a seat with the
    /// table open is asked for the card it had chosen, once a pick, and waited on until it says or the wait is up. A seat
    /// that is not there is not waited on at all, nor one whose pack is down to its last card, which there is no choosing.
    /// </summary>
    private bool Asking(int seat, DateTimeOffset now)
    {
        if (!_connected.Contains(seat) || _draft.Current(seat) is { Cards.Count: 1 }) return false;
        int pick = _draft.Picks[seat - 1];
        if (_asked.TryGetValue(seat, out var asked) && asked.Pick == pick) return now < asked.By;
        _asked[seat] = (pick, now + Timing.PickAnswer);
        Changed();
        return true;
    }

    /// <summary>Whether a seat has been asked for the card it had chosen from the pack in front of it, and not yet answered.</summary>
    private bool PickDue(int seat) =>
        _asked.TryGetValue(seat, out var asked) && asked.Pick == _draft.Picks[seat - 1] && _draft.Current(seat) is not null;

    /// <summary>After any pick: the draft saved first - it is the authority on the pools while it runs - then the event.</summary>
    private void Picked()
    {
        _store.SaveDraft(_record.Id, _draft);
        foreach (EventParticipant participant in _record.Participants)
        {
            participant.Pool = [.. _draft.PoolOf(participant.Seat)];
        }

        if (_draft.IsOver)
        {
            _record.Stage = EventStage.DeckBuilding;
            _record.Deadline = Now + Timing.DeckBuilding;
            _record.Detail = "The draft is over. Build your decks.";
            foreach (EventParticipant each in _record.Participants)
            {
                each.Deck = new EventDeck { Main = [.. Enumerable.Range(0, each.Pool.Count)] };
            }
        }
        else
        {
            _record.Detail = $"Pack {_draft.Pack} of {_draft.Packs}.";
        }

        Changed();
        Save();
    }

    private string? Build(int seat, BuildDeckMessage build)
    {
        if (_record.Stage != EventStage.DeckBuilding) return "Decks are not being built.";
        EventParticipant participant = _record.AtSeat(seat)!;
        _changes[seat] = build.Change;
        (EventDeck? deck, string? refusal) = PoolDecks.Built(participant.Deck, participant.Pool.Count, build, _basics, _record.MinimumDeck);
        if (deck is null) return refusal;
        participant.Deck = deck;
        if (_record.Participants.All(each => each.Deck.Submitted))
        {
            DecksBuilt();
        }
        else
        {
            Changed();
            Save();
        }

        return null;
    }

    /// <summary>Every deck is in: the first round is paired, and begins after a moment to read the pairings.</summary>
    private void DecksBuilt()
    {
        _record.Stage = EventStage.Playing;
        _record.Round = 0;
        PairNext(Timing.FirstRound);
        Changed();
        Save();
    }

    // --- Rounds. ---------------------------------------------------------------------------------------------------

    private int Rounds => _record.Draft?.Rounds ?? 3;

    private List<string> SeatOrder => [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => participant.PlayerId)];

    private HashSet<string> Dropped => [.. _record.Participants.Where(participant => participant.Dropped).Select(participant => participant.PlayerId)];

    /// <summary>Pairs the next round, to begin after <paramref name="wait"/>; or ends the event when it has had its rounds.</summary>
    private void PairNext(TimeSpan wait)
    {
        HashSet<string> dropped = Dropped;
        if (_record.Round >= Rounds || _record.Participants.Count(participant => !participant.Dropped) < 2)
        {
            _record.Stage = EventStage.Complete;
            _record.Deadline = null;
            IReadOnlyList<SwissStanding> standings = Swiss.Standings(SeatOrder, _record.Matches, dropped);
            _record.Detail = standings.Count > 0 ? $"{_record.Participant(standings[0].PlayerId)!.Name} won the draft." : "The draft is over.";
            return;
        }

        int round = _record.Round + 1;
        IReadOnlyList<(string First, string? Second)> pairs = Swiss.Pair(round, SeatOrder, _record.Matches, dropped);
        int table = 0;
        foreach ((string first, string? second) in pairs)
        {
            table++;
            var match = new MatchRecord
            {
                Id = $"r{round}t{table}",
                Round = round,
                Table = table,
                Players = second is null ? [first] : [first, second],
                GamesToWin = 2,
            };

            if (second is null)
            {
                match.Bye = true;
                match.Complete = true;
                match.Winners = [first];
                match.Detail = "Bye";
            }
            else
            {
                match.Detail = "Waiting for the round to begin.";
            }

            _record.Matches.Add(match);
        }

        _record.Round = round;
        _record.Deadline = Now + wait;
        _record.Detail = $"Round {round} of {Rounds} begins shortly.";
    }

    /// <summary>
    /// A player drops from the event: they are paired no more. Before a round begins it is paired again without them,
    /// so nobody is given a match against a player who has gone; once it has begun, their match has to be over first -
    /// played out, or its games conceded. Dropping leaves the standings as they are, with the player last, marked so.
    /// </summary>
    private string? Drop(int seat)
    {
        if (_record.AtSeat(seat) is not { } participant) return "That seat is not in this event.";
        if (_record.Stage != EventStage.Playing) return "The event is over: there is nothing to drop from.";
        if (participant.Dropped) return "You have already dropped.";
        if (_record.Deadline is null && RoundMatches().Any(match => !match.Complete && match.Players.Contains(participant.PlayerId)))
            return "Your match is being played: finish it, or concede its games, before you drop.";

        participant.Dropped = true;
        if (_record.Deadline is { } begins)
        {
            // Nothing of the round has been played: it is paired again, to begin when it was going to.
            _record.Matches.RemoveAll(match => match.Round == _record.Round);
            _record.Round--;
            PairNext(begins > Now ? begins - Now : TimeSpan.Zero);
        }

        Changed();
        Save();
        return null;
    }

    /// <summary>The round's wait is over: a table is wanted for the first game of every match.</summary>
    private void BeginRound()
    {
        _record.Deadline = null;
        _record.Detail = $"Round {_record.Round} of {Rounds} is being played.";
        foreach (MatchRecord match in RoundMatches().Where(match => !match.Complete))
        {
            foreach (string player in match.Players)
            {
                _record.Participant(player)!.Deck.Submitted = true;
                match.Decks[player] = Copy(_record.Participant(player)!.Deck);
            }

            match.Detail = "Game 1 is being played.";
            Want(match, 1);
        }

        RoundOver();
        Changed();
        Save();
    }

    private IEnumerable<MatchRecord> RoundMatches() => _record.Matches.Where(match => match.Round == _record.Round);

    /// <summary>A table is wanted for a game of a match.</summary>
    private MatchGame Want(MatchRecord match, int number)
    {
        var game = new MatchGame(this, match, number);
        _games[game.Key] = game;
        return game;
    }

    /// <summary>The table wanted for the game after this one of its match, if there is one yet.</summary>
    internal MatchGame? Next(MatchGame game) => _games.GetValueOrDefault($"{game.Match.Id}g{game.Number + 1}");

    /// <summary>A game's table has opened: its players at the pod's table are sent to it.</summary>
    internal void GameOpened(MatchGame game)
    {
        foreach (string player in game.Match.Players)
        {
            if (_record.Participant(player) is { } participant && _connected.Contains(participant.Seat))
            {
                SendToGame(participant.Seat);
            }
        }
    }

    /// <summary>Sends a seat at the pod's table on to the table of the game it should be at, if there is one.</summary>
    private void SendToGame(int seat)
    {
        if (_record.AtSeat(seat) is not { } participant
            || _games.Values.Where(game => !game.Done && game.Address is not null && game.Match.Players.Contains(participant.PlayerId) && !game.Match.Complete)
                .OrderByDescending(game => game.Number).FirstOrDefault() is not { } game)
        {
            return;
        }

        if (_vacated.Any(each => each.Seat == seat)) return;
        _vacated.Add((seat, new TableMoveMessage { Server = game.Address!.AbsoluteUri, Ticket = game.IssueTicket(participant.PlayerId) }));
        Changed();
    }

    /// <summary>A ticket back to the pod's table for a player whose game is over, and where it is.</summary>
    internal TableMoveMessage? BackToPod(string playerId)
    {
        if (_address is null || _record.Participant(playerId) is not { } participant) return null;
        string ticket = AddTicket(participant);
        Save();
        return new TableMoveMessage { Server = _address.AbsoluteUri, Ticket = ticket };
    }

    /// <summary>A game of a match has ended: the match goes on to its next game, or is decided.</summary>
    internal void GameFinished(MatchGame game, IReadOnlyList<string> winners, bool abandoned, string detail)
    {
        MatchRecord match = game.Match;
        if (match.Games.FirstOrDefault(each => each.Number == game.Number) is { } record)
        {
            record.Ended = Now;
            record.Ending = abandoned ? "abandoned" : winners.Count == 0 ? "draw" : "won";
            record.Winners = [.. winners];
            record.Detail = detail;
        }

        (int first, int second, int draws) = Score(match);
        int played = first + second + draws;
        if (first >= match.GamesToWin || second >= match.GamesToWin || played >= 2 * match.GamesToWin - 1 || abandoned)
        {
            match.Complete = true;
            match.Winners = first > second ? [match.Players[0]] : second > first ? [match.Players[1]] : [];
            match.Detail = match.Winners.Count == 0
                ? $"Drawn {first}-{second}" + (draws > 0 ? $"-{draws}" : string.Empty)
                : $"{_record.Participant(match.Winners[0])!.Name} won {Math.Max(first, second)}-{Math.Min(first, second)}" + (draws > 0 ? $"-{draws}" : string.Empty);
            RoundOver();
        }
        else
        {
            // The next game, at a table of its own, after sideboarding.
            foreach (string player in match.Players)
            {
                match.Decks[player].Submitted = false;
            }

            match.Detail = $"Game {game.Number + 1} is about to begin.";
            Want(match, game.Number + 1);
        }

        Changed();
        Save();
    }

    /// <summary>Players were not at their match: it is forfeited to whoever was there, and whoever was not is dropped.</summary>
    internal void Forfeited(MatchGame game, IReadOnlyList<string> absent)
    {
        MatchRecord match = game.Match;
        if (match.Complete) return;
        if (match.Games.FirstOrDefault(each => each.Number == game.Number && each.Ending.Length == 0) is { } record)
        {
            record.Ended = Now;
            record.Ending = "abandoned";
            record.Detail = "Not finished: a player was not there.";
        }

        match.Complete = true;
        match.Forfeit = true;
        match.Winners = [.. match.Players.Where(player => !absent.Contains(player))];
        foreach (string player in absent)
        {
            _record.Participant(player)!.Dropped = true;
        }

        string names = string.Join(" and ", absent.Select(player => _record.Participant(player)!.Name));
        match.Detail = match.Winners.Count == 0
            ? $"Neither player came: no result."
            : $"{_record.Participant(match.Winners[0])!.Name} won: {names} {(absent.Count == 1 ? "was" : "were")} not there.";
        RoundOver();
        Changed();
        Save();
    }

    /// <summary>
    /// A player's match clock ran out: they lose the match, whatever the score - the game being played is their opponent's,
    /// and so is the match.
    /// </summary>
    internal void TimedOut(MatchGame game, string loser)
    {
        MatchRecord match = game.Match;
        if (match.Complete) return;
        string winner = match.Players.First(player => player != loser);
        if (match.Games.FirstOrDefault(each => each.Number == game.Number && each.Ending.Length == 0) is { } record)
        {
            record.Ended = Now;
            record.Ending = "won";
            record.Winners = [winner];
            record.Detail = $"{_record.Participant(loser)!.Name} ran out of time.";
        }

        match.Complete = true;
        match.TimedOut = loser;
        match.Winners = [winner];
        match.Detail = $"{_record.Participant(winner)!.Name} won: {_record.Participant(loser)!.Name} ran out of time.";
        RoundOver();
        Changed();
        Save();
    }

    /// <summary>When the round's last match is over, the next round is paired - or the event ends.</summary>
    private void RoundOver()
    {
        if (_record.Stage == EventStage.Playing && _record.Deadline is null && RoundMatches().All(match => match.Complete))
        {
            PairNext(Timing.BetweenRounds);
        }
    }

    /// <summary>Games won by each player of a match, and drawn.</summary>
    private static (int First, int Second, int Draws) Score(MatchRecord match)
    {
        if (match.Players.Count < 2) return (0, 0, 0);
        (int first, _, int draws) = Swiss.GamesOf(match, match.Players[0]);
        (int second, _, _) = Swiss.GamesOf(match, match.Players[1]);
        return (first, second, draws);
    }

    internal ulong SeedFor(MatchRecord match, int game) =>
        unchecked(_record.Seed + ((ulong)match.Round * 0x9E3779B97F4A7C15UL) + ((ulong)match.Table * 0xC2B2AE3D27D4EB4FUL) + ((ulong)game * 0x165667B19E3779F9UL));

    /// <summary>
    /// Deals one game of a match on the rules engine asked for: each player's deck for the match, or their event deck
    /// before any sideboarding, named card by card.
    /// </summary>
    internal IRulesHost NewGame(MatchRecord match, ulong seed, IReadOnlyList<string> names, IRulesHostFactory? rules = null)
    {
        List<RulesSeat> seats = [.. match.Players.Select((player, index) =>
        {
            EventParticipant participant = _record.Participant(player)!;
            EventDeck deck = match.Decks.GetValueOrDefault(player) ?? participant.Deck;
            return new RulesSeat(
                index < names.Count ? names[index] : participant.Name,
                [.. PoolDecks.Cards(participant.Pool, deck, _basics).Select(card => NameOf(card))],
                [.. PoolDecks.Sideboard(participant.Pool, deck).Select(card => NameOf(card))]);
        })];

        // A Limited sideboard is the rest of the pool and any number of basic lands (CR 100.4b; Magic Tournament Rules).
        return (rules ?? RulesHostFactory.Native).Create(seats, seed, new RulesGameOptions { BasicLandsOutside = [.. PoolDecks.BasicLands.Select(name => NameOf(_basics[name]))] });
    }

    /// <summary>
    /// A pool's card as the rules name it: its id for the engine, which is its name for every card the content has
    /// (CR 201.2); the loaded card's name, which is how the native engine knows cards, or the id as it stands
    /// for one the content no longer has, which the host then refuses by name.
    /// </summary>
    private string NameOf(CardId card) =>
        _cards.TryGet(card, out CardDefinition? loaded) ? loaded!.Name : card.Value;

    internal CardCatalog Cards => _cards;

    internal IReadOnlyDictionary<string, CardId> BasicCards => _basics;

    // --- What the seats are shown. ----------------------------------------------------------------------------------

    private DraftMessage DraftView(int seat)
    {
        EventParticipant? me = _record.AtSeat(seat);
        BoosterPack? pack = _draft.Current(seat);
        List<DraftLine> recent = [.. _draft.Log.Where(pick => pick.Seat == seat).TakeLast(8).Select(pick => DraftLine.Write(name =>
            $"Pack {pick.Pack} pick {pick.Pick}: {name(Card(pick.Card))}{(pick.Spent.Length > 0 ? $" (for {name(Card(pick.Spent))})" : pick.Auto ? pick.Offered.Count == 1 ? " (last card)" : " (time ran out)" : string.Empty)}"))];
        return new DraftMessage
        {
            EventId = _record.Id,
            Format = DraftFormats.Booster,
            Active = pack is null ? 0 : seat,
            Decision = seat >= 1 && seat <= _draft.Seats ? _draft.Picks[seat - 1] : 0,
            Pile = pack is null ? [] : [.. pack.Cards.Select(Card)],
            PileRanks = pack is null || _draft.PickOrder.Count == 0 ? [] : [.. pack.Cards.Select(_draft.Rank)],
            AdditionalPicks = seat >= 1 && seat <= _draft.Seats ? [.. _draft.Spendable(seat)] : [],
            Pool = me is null ? [] : [.. me.Pool.Select(Card)],
            Seats = [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new DraftSeatView
            {
                Seat = participant.Seat, Name = participant.Name, Drafted = participant.Pool.Count, Queued = _draft.QueueOf(participant.Seat).Count,
            })],
            Recent = [.. recent.Select(line => line.Text)],
            RecentCards = [.. recent.Select(line => line.Cards)],
            Total = _record.Draft?.Cards ?? 0,
            PackNumber = _draft.Pack,
            Packs = _draft.Packs,
            PackSize = _draft.PackSize,
            PickNumber = pack is null ? 0 : _draft.PackSize - pack.Cards.Count + 1,
            PickLeftMs = _draft.Left(seat, Now) is { } left ? Math.Max(1, (int)left.TotalMilliseconds) : 0,
            Direction = _draft.Direction,
            PickDue = PickDue(seat),
        };
    }

    private DeckBuildingMessage DeckView(int seat)
    {
        EventParticipant me = _record.AtSeat(seat)!;
        return Deck(me, me.Deck, _changes.GetValueOrDefault(seat), _record.Deadline, string.Empty,
            [.. _record.Participants.OrderBy(participant => participant.Seat).Select(participant => new DeckBuilderSeatView
            {
                Seat = participant.Seat, Name = participant.Name, Submitted = participant.Deck.Submitted,
            })]);
    }

    internal DeckBuildingMessage Deck(EventParticipant me, EventDeck deck, int change, DateTimeOffset? deadline, string title, List<DeckBuilderSeatView> seats) => new()
    {
        EventId = _record.Id,
        Pool = [.. me.Pool.Select(Card)],
        Main = [.. deck.Main],
        Basics = [.. PoolDecks.BasicLands.Select(name => new BasicLandView { Name = name, Count = deck.Basics.GetValueOrDefault(name), Card = Card(_basics[name].Value) })],
        Minimum = _record.MinimumDeck,
        Submitted = deck.Submitted,
        Change = change,
        Seats = seats,
        LeftMs = deadline is { } until ? Math.Max(1, (int)Math.Min(int.MaxValue, Math.Max(0, (until - Now).TotalMilliseconds))) : 0,
        Title = title,
    };

    private PairingsMessage PairingsView()
    {
        HashSet<string> dropped = Dropped;
        IReadOnlyList<SwissStanding> standings = Swiss.Standings(SeatOrder, _record.Matches, dropped);
        return new PairingsMessage
        {
            EventId = _record.Id,
            Round = _record.Round,
            Rounds = Rounds,
            StartsInMs = _record.Deadline is { } starts ? Math.Max(1, (int)Math.Max(0, (starts - Now).TotalMilliseconds)) : 0,
            Final = _record.Stage == EventStage.Complete,
            Detail = _record.Detail,
            Matches = [.. RoundMatches().OrderBy(match => match.Table).Select(Pairing)],
            Standings = [.. standings.Select(standing => new StandingView
            {
                Rank = standing.Rank,
                Seat = _record.Participant(standing.PlayerId)!.Seat,
                Name = _record.Participant(standing.PlayerId)!.Name,
                Points = standing.Points,
                Wins = standing.Wins,
                Losses = standing.Losses,
                Draws = standing.Draws,
                OpponentMatchWin = standing.OpponentMatchWin,
                GameWin = standing.GameWin,
                OpponentGameWin = standing.OpponentGameWin,
                Dropped = standing.Dropped,
            })],
        };
    }

    private PairingView Pairing(MatchRecord match)
    {
        EventParticipant first = _record.Participant(match.Players[0])!;
        EventParticipant? second = match.Players.Count > 1 ? _record.Participant(match.Players[1]) : null;
        (int firstWins, int secondWins, int draws) = Score(match);
        int game = match.Complete || _record.Deadline is not null ? 0 : match.Games.Count(each => each.Ending.Length > 0) + 1;
        return new PairingView
        {
            Table = match.Table,
            FirstSeat = first.Seat,
            First = first.Name,
            SecondSeat = second?.Seat ?? 0,
            Second = second?.Name ?? string.Empty,
            FirstWins = firstWins,
            SecondWins = secondWins,
            Draws = draws,
            Game = game,
            Complete = match.Complete,
            Status = match.Detail,
        };
    }

    internal ObjectView Card(string id) =>
        _oracle.Oracle(new CardId(id)) ?? new ObjectView { Known = true, CardId = id, Name = id };

    private IReadOnlyList<ColorView> ColorsOf(string id) => Card(id).Colors;

    internal IReadOnlyList<ColorView> ColorsOfCard(string id) => ColorsOf(id);

    // --- Keeping it. ----------------------------------------------------------------------------------------------

    /// <summary>Something the seats are shown has changed: every table of the pod shows it again.</summary>
    internal void Changed()
    {
        _version++;
        foreach (MatchGame game in _games.Values) game.Changed();
    }

    internal void Save()
    {
        _record.Updated = Now;
        _store.SaveEvent(_record);
    }

    /// <summary>
    /// Takes the event up where it was saved: the draft's pools as the authority while it runs, every clock started
    /// again from now, and a table wanted again for every game that was being played or about to be.
    /// </summary>
    private void TakeUp()
    {
        DateTimeOffset now = Now;
        // The pick clock is the host's to set, as every other clock is.
        _draft.SecondsPerCard = Timing.SecondsPerCard;
        if (_record.Stage is EventStage.Seating or EventStage.Drafting)
        {
            foreach (EventParticipant participant in _record.Participants)
            {
                participant.Pool = [.. _draft.PoolOf(participant.Seat)];
            }

            _draft.Resume(now);
        }

        TimeSpan grace = TimeSpan.FromMinutes(1);
        if (_record.Deadline is { } deadline && deadline < now + grace)
        {
            _record.Deadline = now + grace;
        }

        if (_record.Stage == EventStage.Playing && _record.Deadline is null)
        {
            foreach (MatchRecord match in RoundMatches().Where(match => !match.Complete))
            {
                int ended = match.Games.Count(game => game.Ending.Length > 0);
                bool cutOff = match.Games.Any(game => game.Ending.Length == 0);
                Want(match, cutOff ? match.Games.First(game => game.Ending.Length == 0).Number : ended + 1);
            }
        }
    }

    private static string AddTicket(EventParticipant participant)
    {
        string ticket = NewTicket();
        participant.TicketHashes.Add(EventStore.HashTicket(ticket));
        if (participant.TicketHashes.Count > TicketsKept)
        {
            participant.TicketHashes.RemoveRange(0, participant.TicketHashes.Count - TicketsKept);
        }

        return ticket;
    }

    /// <summary>The seat whose hashes include the ticket's, compared in constant time; zero for none.</summary>
    internal static int Matching(string ticket, IEnumerable<(int Seat, IEnumerable<string> Hashes)> seats)
    {
        byte[] presented = Convert.FromHexString(EventStore.HashTicket(ticket));
        int found = 0;
        foreach ((int seat, IEnumerable<string> hashes) in seats)
        {
            foreach (string hash in hashes)
            {
                if (CryptographicOperations.FixedTimeEquals(Convert.FromHexString(hash), presented))
                {
                    found = seat;
                }
            }
        }

        return found;
    }

    internal static string NewTicket() => Convert.ToHexString(RandomNumberGenerator.GetBytes(32));

    private static EventDeck Copy(EventDeck deck) => new() { Main = [.. deck.Main], Basics = new(deck.Basics, StringComparer.Ordinal), Submitted = deck.Submitted, Revision = deck.Revision };

    private static (List<string> Cube, List<string> Missing) Loaded(ContentLibrary content, IReadOnlyList<string> cubeCards)
    {
        ArgumentNullException.ThrowIfNull(content);
        ArgumentNullException.ThrowIfNull(cubeCards);
        Dictionary<string, CardId?> byName = new(StringComparer.OrdinalIgnoreCase);
        foreach (CardDefinition card in content.Cards.InOrder())
        {
            byName[card.Name] = byName.ContainsKey(card.Name) ? null : card.Id;
        }

        List<string> cube = [];
        List<string> missing = [];
        foreach (string name in cubeCards)
        {
            if (byName.TryGetValue(name, out CardId? id) && id is { } found) cube.Add(found.Value);
            else missing.Add(name);
        }

        return (cube, missing);
    }
}
