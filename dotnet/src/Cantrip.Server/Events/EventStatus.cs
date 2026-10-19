namespace Cantrip.Server.Events;

/// <summary>
/// What anyone may see of the events a server runs: who is playing what, where each event has got to, the pairings,
/// results and standings - for a status page open to everyone. Only what the players at the table are all shown is
/// here, and nothing any one of them is not: no pool, no deck, no card a seat has been dealt or drafted, no ticket, no
/// seed (from which every shuffle follows), no player id and nothing a host keeps with an event.
/// <para>
/// Read from the data directory, which every draft and pod saves after each change, so it is as current as the events'
/// own records. What was read is kept and read again only when an event's files have changed, so a finished event is
/// read once, and a page asking every few seconds does not hold the store up.
/// </para>
/// </summary>
public sealed class EventStatusBoard
{
    private readonly EventStore? _store;
    private readonly Func<IReadOnlyList<StatusTable>>? _tables;
    private readonly Func<IReadOnlyCollection<string>>? _hosting;
    private readonly Dictionary<string, (DateTime Written, StatusEventDetail Detail)> _read = new(StringComparer.Ordinal);
    private readonly object _gate = new();

    /// <param name="store">The data directory's events; null for a host that keeps none, which has only its tables to show.</param>
    /// <param name="tables">
    /// The host's tables that are not events in the store - a table still forming, a game hosted on its own - or null for
    /// a host with none.
    /// </param>
    /// <param name="hosting">
    /// The events the host is running, by id; null to take every event the store has as live as being played. An event
    /// the store has as live that the host is not running - played by another host in the same directory, or stopped for
    /// good - is not being played: it is shown among the events over, as not finished.
    /// </param>
    public EventStatusBoard(EventStore? store, Func<IReadOnlyList<StatusTable>>? tables = null, Func<IReadOnlyCollection<string>>? hosting = null)
    {
        _store = store;
        _tables = tables;
        _hosting = hosting;
    }

    /// <summary>Every event still being played, newest first, and the host's other tables.</summary>
    public StatusActive Active()
    {
        List<StatusEvent> events = [.. All().Select(detail => detail.Event).Where(summary => summary.Live).OrderByDescending(summary => summary.Created)];
        return new StatusActive(events, _tables?.Invoke() ?? []);
    }

    /// <summary>
    /// The events over - complete or called off - that <paramref name="query"/> asks for, the most recently finished first;
    /// and every cube and player any finished event has had, to filter by.
    /// </summary>
    public StatusHistory History(StatusQuery query)
    {
        ArgumentNullException.ThrowIfNull(query);
        List<StatusEvent> over = [.. All().Select(detail => detail.Event).Where(summary => !summary.Live)];
        List<StatusEvent> matching = [.. over
            .Where(summary => query.From is not { } from || summary.Updated >= from)
            .Where(summary => query.To is not { } to || summary.Updated <= to)
            .Where(summary => query.Kind is not { Length: > 0 } kind || summary.Kind == kind)
            .Where(summary => query.Cube is not { Length: > 0 } cube || string.Equals(summary.Cube, cube, StringComparison.OrdinalIgnoreCase))
            .Where(summary => query.Player is not { Length: > 0 } player
                || summary.Players.Any(seat => seat.Name.Contains(player, StringComparison.OrdinalIgnoreCase)))
            .Where(summary => query.Outcome is not { Length: > 0 } outcome || Outcome(summary).Equals(outcome, StringComparison.OrdinalIgnoreCase))
            .OrderByDescending(summary => summary.Updated)];
        int total = matching.Count;
        return new StatusHistory(
            [.. matching.Take(Math.Clamp(query.Limit, 1, 1000))],
            total,
            [.. over.Select(summary => summary.Cube).Where(cube => cube.Length > 0).Distinct(StringComparer.OrdinalIgnoreCase).Order(StringComparer.OrdinalIgnoreCase)],
            [.. over.SelectMany(summary => summary.Players.Select(seat => seat.Name)).Distinct(StringComparer.OrdinalIgnoreCase).Order(StringComparer.OrdinalIgnoreCase)]);
    }

    /// <summary>Everything public about one event; null for an id the store has no event under.</summary>
    public StatusEventDetail? Detail(string id) =>
        _store is not null && EventStore.ValidId(id) && Read(id) is { } detail ? Hosted(detail, _hosting?.Invoke()) : null;

    /// <summary>How an event over ended: <see cref="StatusOutcomes"/>.</summary>
    private static string Outcome(StatusEvent summary) => summary.Stage switch
    {
        nameof(EventStage.Complete) => StatusOutcomes.Complete,
        nameof(EventStage.Cancelled) => StatusOutcomes.Cancelled,
        _ => StatusOutcomes.Unfinished,
    };

    /// <summary>An event as live as its record says only while the host runs it.</summary>
    private static StatusEventDetail Hosted(StatusEventDetail detail, IReadOnlyCollection<string>? hosting) =>
        detail.Event.Live && hosting is not null && !hosting.Contains(detail.Event.Id)
            ? detail with { Event = detail.Event with { Live = false } }
            : detail;

    private IEnumerable<StatusEventDetail> All()
    {
        IReadOnlyCollection<string>? hosting = _hosting?.Invoke();
        return [.. Saved().Select(detail => Hosted(detail, hosting))];
    }

    private IEnumerable<StatusEventDetail> Saved()
    {
        if (_store is null) return [];
        List<string> ids;
        try
        {
            ids = [.. _store.EventIds()];
        }
        catch (DirectoryNotFoundException)
        {
            return [];
        }

        lock (_gate)
        {
            // An event gone from the directory is forgotten.
            foreach (string gone in _read.Keys.Except(ids, StringComparer.Ordinal).ToList())
            {
                _read.Remove(gone);
            }
        }

        // A league has pages of its own, and its runs' drafts are its own business.
        return [.. ids.Select(Read).OfType<StatusEventDetail>().Where(detail => detail.Event.ParentId is null && detail.Event.Kind != EventKinds.League)];
    }

    private StatusEventDetail? Read(string id)
    {
        if (_store is null) return null;
        DateTime written = _store.EventWrittenAt(id);
        lock (_gate)
        {
            if (_read.TryGetValue(id, out var kept) && kept.Written == written)
            {
                return kept.Detail;
            }
        }

        if (_store.LoadEvent(id) is not { } record)
        {
            return null;
        }

        StatusEventDetail detail = Project(record, record.Draft?.Format switch
        {
            DraftFormats.Booster => Booster(record, _store.LoadDraft<BoosterDraft>(id)),
            DraftFormats.Grid => Grid(record, _store.LoadDraft<GridDraft>(id)),
            DraftFormats.Winston => Winston(record, _store.LoadDraft<WinstonDraft>(id)),
            _ => null,
        });

        lock (_gate)
        {
            _read[id] = (written, detail);
        }

        return detail;
    }

    /// <summary>The public face of an event, its draft as <paramref name="draft"/> says it stands.</summary>
    public static StatusEventDetail Project(EventRecord record, StatusDraft? draft)
    {
        ArgumentNullException.ThrowIfNull(record);
        string Name(string playerId) => record.Participant(playerId)?.Name ?? "Someone";
        bool building = record.Stage == EventStage.DeckBuilding;

        List<StatusPlayer> players = [.. record.Participants.OrderBy(participant => participant.Seat).Select(participant => new StatusPlayer(
            participant.Name, participant.Seat, participant.Dropped, participant.Seated, building && participant.Deck.Submitted))];

        int rounds = record.Draft?.Rounds ?? 0;
        HashSet<string> dropped = [.. record.Participants.Where(participant => participant.Dropped).Select(participant => participant.PlayerId)];
        List<StatusStanding> standings = rounds > 0 && record.Matches.Any(match => match.Complete)
            ? [.. Swiss.Standings([.. record.Participants.OrderBy(participant => participant.Seat).Select(participant => participant.PlayerId)], record.Matches, dropped)
                .Select(standing => new StatusStanding(standing.Rank, Name(standing.PlayerId), standing.Points,
                    standing.Wins, standing.Losses, standing.Draws, standing.GameWins, standing.GameLosses, standing.GameDraws,
                    standing.OpponentMatchWin, standing.GameWin, standing.OpponentGameWin, standing.Dropped))]
            : [];

        List<StatusRound> played = [.. record.Matches.GroupBy(match => match.Round).OrderBy(round => round.Key).Select(round => new StatusRound(round.Key,
            [.. round.OrderBy(match => match.Table).Select(match => new StatusMatch(
                match.Table,
                [.. match.Players.Select(player => new StatusMatchPlayer(Name(player), Swiss.GamesOf(match, player).Won,
                    match.ClockMs.TryGetValue(player, out long left) ? left : null))],
                match.Complete,
                match.Bye,
                match.Forfeit,
                match.Winners.Count > 0 ? string.Join(" and ", match.Winners.Select(Name)) : null,
                match.TimedOut is { } timedOut ? Name(timedOut) : null,
                match.GamesToWin,
                PublicDetail(match.Detail),
                [.. match.Games.Select(game => new StatusGame(game.Number, game.Started, game.Ended, game.Ending,
                    game.Winners.Count > 0 ? string.Join(" and ", game.Winners.Select(Name)) : null, PublicDetail(game.Detail)))]))]))];

        string? winner = record.Stage != EventStage.Complete
            ? null
            : standings is [{ } first, ..]
                ? first.Name
                : record.Matches.LastOrDefault() is { Winners.Count: > 0 } last ? string.Join(" and ", last.Winners.Select(Name)) : null;

        var summary = new StatusEvent(
            record.Id,
            record.Kind,
            record.Name,
            record.Draft?.Format ?? string.Empty,
            record.Draft?.Cube ?? string.Empty,
            record.Stage.ToString(),
            record.IsLive,
            record.Round,
            rounds,
            record.Created,
            record.Updated,
            record.Deadline,
            PublicDetail(record.Detail),
            Progress(record, draft),
            players,
            winner,
            record.ParentId);

        return new StatusEventDetail(summary, record.Draft is { } settings ? new StatusSettings(settings.Format, settings.Cube, settings.Cards,
            settings.Piles, settings.Packs, settings.PackSize, settings.Rounds, record.MinimumDeck) : null, draft, standings, played);
    }

    /// <summary>
    /// The line on how an event stands, less any reason a game could not be dealt: that names a card from a player's pool
    /// (<see cref="DraftPod"/>), which is theirs to keep.
    /// </summary>
    private static string PublicDetail(string detail) =>
        detail.StartsWith(DraftPod.DealFailedLine, StringComparison.Ordinal) ? DraftPod.DealFailedLine + "." : detail;

    /// <summary>Where an event stands, in a few words, for a list of events.</summary>
    private static string Progress(EventRecord record, StatusDraft? draft) => record.Stage switch
    {
        EventStage.Seating => $"Seating: {record.Participants.Count(participant => participant.Seated)} of {record.Participants.Count} seated",
        EventStage.Drafting when draft is { Format: DraftFormats.Booster } => $"Drafting pack {draft.Pack} of {draft.Packs}",
        EventStage.Drafting when draft is { Format: DraftFormats.Grid } => $"Drafting grid {draft.Pack} of {draft.Packs}",
        EventStage.Drafting when draft is { Format: DraftFormats.Winston } => $"Drafting: {draft.Stack} cards left in the stack",
        EventStage.Drafting => "Drafting",
        EventStage.DeckBuilding => $"Building decks: {record.Participants.Count(participant => participant.Deck.Submitted)} of {record.Participants.Count} submitted",
        EventStage.Playing when (record.Draft?.Rounds ?? 0) > 0 => $"Round {record.Round} of {record.Draft!.Rounds}",
        EventStage.Playing => "Playing",
        EventStage.Complete => "Complete",
        EventStage.Cancelled => "Called off",
        _ => record.Stage.ToString(),
    };

    /// <summary>
    /// A booster draft as the table sees it: the pack being drafted, which way it goes, and for each seat how many cards
    /// it has taken, how many packs wait for it and when its pick clock runs out. Not one card.
    /// </summary>
    private static StatusDraft? Booster(EventRecord record, BoosterDraft? draft)
    {
        if (draft is null) return null;
        return new StatusDraft(
            DraftFormats.Booster,
            draft.Pack,
            draft.Packs,
            draft.PackSize,
            draft.IsOver ? 0 : draft.Direction,
            null,
            [.. Enumerable.Range(1, draft.Seats).Select(seat => new StatusDraftSeat(
                record.AtSeat(seat)?.Name ?? $"Seat {seat}",
                seat,
                draft.PoolOf(seat).Count,
                draft.QueueOf(seat).Count,
                draft.IsOver || seat > draft.Deadlines.Count ? null : draft.Deadlines[seat - 1],
                false))],
            0,
            [],
            -1,
            0,
            draft.Seats * draft.Packs * draft.PackSize);
    }

    /// <summary>A grid draft: the grid being laid out, whose pick it is, and how many cards each player has taken.</summary>
    private static StatusDraft? Grid(EventRecord record, GridDraft? draft)
    {
        if (draft is null) return null;
        return new StatusDraft(
            DraftFormats.Grid,
            draft.IsOver ? 0 : draft.Pack,
            draft.Packs,
            GridDraft.PackSize,
            0,
            draft.IsOver ? null : draft.PickOfPack,
            [.. Enumerable.Range(1, draft.Seats).Select(seat => new StatusDraftSeat(
                record.AtSeat(seat)?.Name ?? $"Seat {seat}", seat, draft.PoolOf(seat).Count, 0, null, draft.Active == seat))],
            0,
            [],
            -1,
            draft.Discarded.Count,
            draft.Packs * GridDraft.PackSize);
    }

    /// <summary>A Winston draft: the stack's and the piles' sizes, which pile is being looked at and by whom.</summary>
    private static StatusDraft? Winston(EventRecord record, WinstonDraft? draft)
    {
        if (draft is null) return null;
        int drafted = draft.Pools.Sum(pool => pool.Count);
        return new StatusDraft(
            DraftFormats.Winston,
            0,
            0,
            0,
            0,
            null,
            [.. Enumerable.Range(1, draft.Seats).Select(seat => new StatusDraftSeat(
                record.AtSeat(seat)?.Name ?? $"Seat {seat}", seat, draft.PoolOf(seat).Count, 0, null, draft.Active == seat))],
            draft.Stack.Count,
            [.. draft.Piles.Select(pile => pile.Count)],
            draft.IsOver ? -1 : draft.Looking,
            0,
            drafted + draft.Stack.Count + draft.Piles.Sum(pile => pile.Count));
    }
}

/// <summary>What the history of events is filtered by: when they finished, their kind, cube, a player's name, how they ended.</summary>
public sealed record StatusQuery(
    DateTimeOffset? From = null,
    DateTimeOffset? To = null,
    string? Kind = null,
    string? Cube = null,
    string? Player = null,
    string? Outcome = null,
    int Limit = 200);

/// <summary>How an event over ended, as <see cref="StatusQuery.Outcome"/> asks for it.</summary>
public static class StatusOutcomes
{
    public const string Complete = "Complete";
    public const string Cancelled = "Cancelled";

    /// <summary>Neither: left where it was by a host that stopped running it.</summary>
    public const string Unfinished = "Unfinished";
}

/// <summary>The events being played, and the host's tables that are not events.</summary>
public sealed record StatusActive(IReadOnlyList<StatusEvent> Events, IReadOnlyList<StatusTable> Tables);

/// <summary>Events over that a query matched (the first of them, of <see cref="Total"/>), and what there is to filter by.</summary>
public sealed record StatusHistory(IReadOnlyList<StatusEvent> Events, int Total, IReadOnlyList<string> Cubes, IReadOnlyList<string> Players);

/// <summary>An event in a list: what it is, who is in it, and where it stands.</summary>
public sealed record StatusEvent(
    string Id,
    string Kind,
    string Name,
    string Format,
    string Cube,
    string Stage,
    bool Live,
    int Round,
    int Rounds,
    DateTimeOffset Created,
    DateTimeOffset Updated,
    DateTimeOffset? Deadline,
    string Detail,
    string Progress,
    IReadOnlyList<StatusPlayer> Players,
    string? Winner,
    string? ParentId);

/// <summary>A player in an event, by the name they go by at the table. <see cref="Submitted"/> only while decks are built.</summary>
public sealed record StatusPlayer(string Name, int Seat, bool Dropped, bool Seated, bool Submitted);

/// <summary>Everything public about one event.</summary>
public sealed record StatusEventDetail(
    StatusEvent Event,
    StatusSettings? Settings,
    StatusDraft? Draft,
    IReadOnlyList<StatusStanding> Standings,
    IReadOnlyList<StatusRound> Rounds);

/// <summary>How an event drafts and plays, as it was set up.</summary>
public sealed record StatusSettings(string Format, string Cube, int Cards, int Piles, int Packs, int PackSize, int Rounds, int MinimumDeck);

/// <summary>
/// A draft as everyone at its table sees it. <see cref="Pack"/> is the pack (or grid) being drafted, from 1, zero once
/// over; <see cref="Direction"/> +1 passing left, -1 right; <see cref="PickOfPack"/> a grid's first or second pick;
/// <see cref="Stack"/>, <see cref="Piles"/> and <see cref="Looking"/> a Winston draft's.
/// </summary>
public sealed record StatusDraft(
    string Format,
    int Pack,
    int Packs,
    int PackSize,
    int Direction,
    int? PickOfPack,
    IReadOnlyList<StatusDraftSeat> Seats,
    int Stack,
    IReadOnlyList<int> Piles,
    int Looking,
    int Discarded,
    int Cards);

/// <summary>A seat at a draft: how many cards it has taken, packs waiting for it, when its pick clock runs out, whether it is to pick.</summary>
public sealed record StatusDraftSeat(string Name, int Seat, int Drafted, int Queued, DateTimeOffset? PickBy, bool Active);

/// <summary>A player's place in a Swiss event, with the record and tiebreakers that put them there.</summary>
public sealed record StatusStanding(
    int Rank,
    string Name,
    int Points,
    int Wins,
    int Losses,
    int Draws,
    int GameWins,
    int GameLosses,
    int GameDraws,
    double OpponentMatchWin,
    double GameWin,
    double OpponentGameWin,
    bool Dropped);

/// <summary>A round's pairings and how each match stands.</summary>
public sealed record StatusRound(int Round, IReadOnlyList<StatusMatch> Matches);

public sealed record StatusMatch(
    int Table,
    IReadOnlyList<StatusMatchPlayer> Players,
    bool Complete,
    bool Bye,
    bool Forfeit,
    string? Winner,
    string? TimedOut,
    int GamesToWin,
    string Detail,
    IReadOnlyList<StatusGame> Games);

/// <summary>A player in a match: games won, and what is left on their match clock as last saved (null before it started).</summary>
public sealed record StatusMatchPlayer(string Name, int Wins, long? ClockMs);

public sealed record StatusGame(int Number, DateTimeOffset Started, DateTimeOffset? Ended, string Ending, string? Winner, string Detail);

/// <summary>
/// A table a host runs that is not an event in the store: a draft table still filling from a queue, or a game hosted on
/// its own. <see cref="Kind"/> is <see cref="StatusTableKinds"/>.
/// </summary>
public sealed record StatusTable(
    string Kind,
    string Format,
    string Cube,
    string Stage,
    IReadOnlyList<string> Players,
    int Seats,
    int Bots,
    DateTimeOffset Created,
    string Detail);

public static class StatusTableKinds
{
    /// <summary>A queue's table, filling with players before its draft begins.</summary>
    public const string Forming = "forming";

    /// <summary>A game of two players hosted on its own, outside any event.</summary>
    public const string Game = "game";
}
