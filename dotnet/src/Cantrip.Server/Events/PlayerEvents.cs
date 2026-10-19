namespace Cantrip.Server.Events;

/// <summary>
/// One event as its own player sees it in the list of their events: what it is, where it stands, and how it has gone
/// for them - their record, their place, whether they dropped. Names and numbers only: no pool, deck, ticket or seed.
/// </summary>
/// <param name="Live">Being played: live in the record and hosted now. An event the host no longer runs is not live.</param>
/// <param name="Rank">Their place in the standings, from 1, once a round has been played; null before.</param>
/// <param name="Outcome">How it went or is going for them, in a few words: "Won", "2nd of 8", "Dropped", "Round 2 of 3".</param>
/// <param name="Opponents">By name: the other player of a draft of two, or whom they were paired with round by round.</param>
public sealed record PlayerEventSummary(
    string Id,
    string Name,
    string Format,
    string Cube,
    EventStage Stage,
    string Detail,
    DateTimeOffset Created,
    DateTimeOffset Updated,
    bool Live,
    bool Dropped,
    int Wins,
    int Losses,
    int Draws,
    int? Rank,
    int Players,
    int Round,
    int Rounds,
    string Outcome,
    IReadOnlyList<string> Opponents);

/// <summary>
/// The events a player is in, from their record in the data directory (<see cref="PlayerRecord.Events"/>): the ones
/// still being played, and the most recent of those over.
/// </summary>
public static class PlayerEvents
{
    /// <summary>
    /// Every event of <paramref name="playerId"/>'s being played, then the <paramref name="recent"/> latest over, newest first.
    /// </summary>
    /// <param name="hosted">Whether the host runs an event now; null takes every live record as being played.</param>
    public static IReadOnlyList<PlayerEventSummary> Of(EventStore store, string playerId, Func<string, bool>? hosted = null, int recent = 10)
    {
        ArgumentNullException.ThrowIfNull(store);
        if (store.LoadPlayer(playerId) is not { } player) return [];

        List<PlayerEventSummary> all = [];
        foreach (string id in player.Events.Select(entry => entry.EventId).Distinct(StringComparer.Ordinal))
        {
            if (!EventStore.ValidId(id) || store.LoadEvent(id) is not { } record || record.Participant(playerId) is null) continue;
            all.Add(Summary(record, playerId, record.IsLive && (hosted?.Invoke(id) ?? true)));
        }

        return [.. all.Where(each => each.Live).OrderByDescending(each => each.Updated),
            .. all.Where(each => !each.Live).OrderByDescending(each => each.Updated).Take(Math.Max(0, recent))];
    }

    /// <summary>One event, as <paramref name="playerId"/> - who is in it - sees it.</summary>
    public static PlayerEventSummary Summary(EventRecord record, string playerId, bool live)
    {
        ArgumentNullException.ThrowIfNull(record);
        EventParticipant me = record.Participant(playerId) ?? throw new ArgumentException($"'{playerId}' is not in event {record.Id}.", nameof(playerId));
        List<string> seats = [.. record.Participants.OrderBy(participant => participant.Seat).Select(participant => participant.PlayerId)];
        HashSet<string> dropped = [.. record.Participants.Where(participant => participant.Dropped).Select(participant => participant.PlayerId)];
        SwissStanding? standing = Swiss.Standings(seats, record.Matches, dropped).FirstOrDefault(each => each.PlayerId == playerId);
        bool played = record.Matches.Any(match => match.Complete && match.Players.Contains(playerId));
        string format = record.Draft?.Format ?? record.Kind;
        int rounds = record.Draft is { Rounds: > 0 } draft ? draft.Rounds : 1;

        return new PlayerEventSummary(
            record.Id,
            record.Name,
            format,
            record.Draft?.Cube ?? string.Empty,
            record.Stage,
            record.Detail,
            record.Created,
            record.Updated,
            live,
            me.Dropped,
            standing?.Wins ?? 0,
            standing?.Losses ?? 0,
            standing?.Draws ?? 0,
            played && standing is not null ? standing.Rank : null,
            record.Participants.Count,
            record.Round,
            rounds,
            Outcome(record, me, standing, played, live, rounds),
            Opponents(record, playerId));
    }

    /// <summary>
    /// Who the player has played or is to play, by name, in the order they met: the other player of a draft of two, the
    /// opponents of each round of a pod.
    /// </summary>
    private static List<string> Opponents(EventRecord record, string playerId)
    {
        if (record.Participants.Count <= 2)
            return [.. record.Participants.Where(participant => participant.PlayerId != playerId).Select(participant => participant.Name)];
        return [.. record.Matches.Where(match => match.Players.Contains(playerId)).OrderBy(match => match.Round)
            .SelectMany(match => match.Players.Where(player => player != playerId))
            .Select(player => record.Participant(player)?.Name ?? player)];
    }

    private static string Outcome(EventRecord record, EventParticipant me, SwissStanding? standing, bool played, bool live, int rounds)
    {
        if (record.Stage == EventStage.Cancelled) return "Called off";
        if (me.Dropped) return "Dropped";
        if (record.IsLive && !live) return "Not finished";
        if (record.IsLive)
        {
            return record.Stage switch
            {
                EventStage.Seating or EventStage.NotActive => "Taking seats",
                EventStage.Drafting => "Drafting",
                EventStage.DeckBuilding => "Building decks",
                _ => rounds > 1 ? $"Round {Math.Max(1, record.Round)} of {rounds}" : "Playing",
            };
        }

        if (!played || standing is null) return "Complete";
        if (record.Participants.Count <= 2)
        {
            return standing.Wins > 0 ? "Won" : standing.Losses > 0 ? "Lost" : "Drawn";
        }

        return $"{Ordinal(standing.Rank)} of {record.Participants.Count}";
    }

    /// <summary>1st, 2nd, 3rd, 4th, ... 11th, 12th, 13th, 21st.</summary>
    public static string Ordinal(int n)
    {
        int tens = n % 100;
        string suffix = tens is >= 11 and <= 13 ? "th" : (n % 10) switch { 1 => "st", 2 => "nd", 3 => "rd", _ => "th" };
        return $"{n}{suffix}";
    }
}
