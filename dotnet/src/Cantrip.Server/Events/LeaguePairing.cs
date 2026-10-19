namespace Cantrip.Server.Events;

/// <summary>A run looking for a league match: whose it is, how it stands, and since when it has looked.</summary>
public sealed record LeagueSearcher(string PlayerId, string RunId, int Wins, int Losses, DateTimeOffset Since);

/// <summary>
/// Who is paired with whom among the runs looking for a match, by the league's rules
/// (<see cref="LeagueSettings.Pairing"/>, <see cref="LeagueSettings.AllowRepeatPairings"/>). Pure: the league keeps who is
/// looking, and makes the matches.
/// </summary>
public static class LeaguePairing
{
    /// <summary>
    /// The matches to make now. Whoever has looked longest is paired first, with whoever else has looked longest that
    /// the rules let them play; anyone left goes on looking.
    /// </summary>
    /// <param name="met">Whether a run has already played a player, for a league that pairs no one twice.</param>
    public static IReadOnlyList<(LeagueSearcher First, LeagueSearcher Second)> Pair(
        IReadOnlyList<LeagueSearcher> searching, LeagueSettings settings, Func<string, string, bool> met, DateTimeOffset now)
    {
        ArgumentNullException.ThrowIfNull(searching);
        ArgumentNullException.ThrowIfNull(settings);
        ArgumentNullException.ThrowIfNull(met);

        List<LeagueSearcher> waiting = [.. searching.OrderBy(each => each.Since).ThenBy(each => each.RunId, StringComparer.Ordinal)];
        List<(LeagueSearcher, LeagueSearcher)> pairs = [];
        while (waiting.Count > 1)
        {
            LeagueSearcher first = waiting[0];
            waiting.RemoveAt(0);
            LeagueSearcher? second = waiting.FirstOrDefault(other => Allowed(first, other, settings, met, now));
            if (second is null) continue;
            waiting.Remove(second);
            pairs.Add((first, second));
        }

        return pairs;
    }

    private static bool Allowed(LeagueSearcher first, LeagueSearcher second, LeagueSettings settings, Func<string, string, bool> met, DateTimeOffset now)
    {
        if (first.PlayerId == second.PlayerId) return false;
        if (!settings.AllowRepeatPairings && (met(first.RunId, second.PlayerId) || met(second.RunId, first.PlayerId))) return false;
        return settings.Pairing switch
        {
            LeaguePairingMode.Free => true,
            // The settings refuse the record modes until they are played (LeagueSettings.Problem); here is where they go.
            _ => throw new NotSupportedException($"Pairing by record ({settings.Pairing}) is not played yet."),
        };
    }
}
