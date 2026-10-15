namespace Cantrip.Server.Events;

/// <summary>A player's place in a Swiss event: their record, their points and the tiebreakers that order equal points.</summary>
public sealed record SwissStanding(
    string PlayerId,
    int Rank,
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

/// <summary>
/// Swiss pairings and standings, as a tournament runs them (MTR appendix C): three match points for a win and one for a
/// draw, players of equal points paired with each other, nobody paired with someone they have already played while
/// anyone else will do, and ties broken by opponents' match-win percentage, game-win percentage and opponents' game-win
/// percentage, each percentage no lower than a third.
/// <para>
/// Round one pairs across the table - seat 1 with the seat half the table away - as a draft pod does, so the players
/// who passed to each other are not paired first.
/// </para>
/// </summary>
public static class Swiss
{
    private const double Floor = 1.0 / 3.0;

    /// <summary>
    /// The standings after the matches finished so far: the players still in, best first, then those who dropped.
    /// </summary>
    /// <param name="players">Every player, in seat order, which breaks a tie the tiebreakers leave.</param>
    public static IReadOnlyList<SwissStanding> Standings(IReadOnlyList<string> players, IEnumerable<MatchRecord> matches, IReadOnlySet<string> dropped)
    {
        ArgumentNullException.ThrowIfNull(players);
        ArgumentNullException.ThrowIfNull(matches);
        ArgumentNullException.ThrowIfNull(dropped);
        List<MatchRecord> finished = [.. matches.Where(match => match.Complete)];

        Dictionary<string, Tally> tallies = players.ToDictionary(player => player, _ => new Tally(), StringComparer.Ordinal);
        foreach (MatchRecord match in finished)
        {
            foreach (string player in match.Players)
            {
                if (!tallies.TryGetValue(player, out Tally? tally)) continue;
                (int won, int lost, int drawn) = GamesOf(match, player);
                tally.GameWins += won;
                tally.GameLosses += lost;
                tally.GameDraws += drawn;
                if (match.Winners.Contains(player)) tally.Wins++;
                else if (match.Winners.Count == 0 && !match.Bye && !match.Forfeit) tally.Draws++;
                else tally.Losses++;
                if (!match.Bye)
                {
                    tally.Opponents.AddRange(match.Players.Where(other => other != player));
                }
            }
        }

        double MatchWin(string player)
        {
            Tally tally = tallies[player];
            int played = tally.Wins + tally.Losses + tally.Draws;
            return played == 0 ? Floor : Math.Max(Floor, (3.0 * tally.Wins + tally.Draws) / (3.0 * played));
        }

        double GameWin(string player)
        {
            Tally tally = tallies[player];
            int played = tally.GameWins + tally.GameLosses + tally.GameDraws;
            return played == 0 ? Floor : Math.Max(Floor, (3.0 * tally.GameWins + tally.GameDraws) / (3.0 * played));
        }

        double Average(string player, Func<string, double> of) =>
            tallies[player].Opponents.Where(tallies.ContainsKey).Select(of).DefaultIfEmpty(Floor).Average();

        List<SwissStanding> standings = [.. players.Select(player =>
        {
            Tally tally = tallies[player];
            return new SwissStanding(player, 0, 3 * tally.Wins + tally.Draws, tally.Wins, tally.Losses, tally.Draws,
                tally.GameWins, tally.GameLosses, tally.GameDraws, Average(player, MatchWin), GameWin(player), Average(player, GameWin), dropped.Contains(player));
        })];

        Dictionary<string, int> seatOrder = players.Select((player, index) => (player, index)).ToDictionary(pair => pair.player, pair => pair.index, StringComparer.Ordinal);
        List<SwissStanding> ordered = [.. standings
            .OrderBy(standing => standing.Dropped)
            .ThenByDescending(standing => standing.Points)
            .ThenByDescending(standing => Round(standing.OpponentMatchWin))
            .ThenByDescending(standing => Round(standing.GameWin))
            .ThenByDescending(standing => Round(standing.OpponentGameWin))
            .ThenBy(standing => seatOrder[standing.PlayerId])];

        return [.. ordered.Select((standing, index) => standing with { Rank = index + 1 })];
    }

    /// <summary>
    /// The pairings for a round: pairs of players, and - with an odd number still in - the one given the bye, as a pair
    /// with no second player.
    /// </summary>
    /// <param name="players">Every player, in seat order.</param>
    public static IReadOnlyList<(string First, string? Second)> Pair(int round, IReadOnlyList<string> players, IEnumerable<MatchRecord> matches, IReadOnlySet<string> dropped)
    {
        ArgumentNullException.ThrowIfNull(players);
        ArgumentNullException.ThrowIfNull(matches);
        ArgumentNullException.ThrowIfNull(dropped);
        List<MatchRecord> played = [.. matches];
        List<string> active = [.. players.Where(player => !dropped.Contains(player))];
        List<(string First, string? Second)> pairs = [];

        if (round <= 1)
        {
            // Across the table: seat 1 with the seat half the table away, and so on round.
            int half = active.Count / 2;
            for (int i = 0; i < half; i++)
            {
                pairs.Add((active[i], active[i + half]));
            }

            if (active.Count % 2 == 1)
            {
                pairs.Add((active[^1], null));
            }

            return pairs;
        }

        // Best first, by the standings; the bye goes to the lowest-placed player who has not had one.
        List<string> ranked = [.. Standings(players, played, dropped).Where(standing => !standing.Dropped).Select(standing => standing.PlayerId)];
        string? bye = null;
        if (ranked.Count % 2 == 1)
        {
            bye = Enumerable.Reverse(ranked).FirstOrDefault(player => !played.Any(match => match.Bye && match.Players.Contains(player))) ?? ranked[^1];
            ranked.Remove(bye);
        }

        HashSet<(string, string)> met = [];
        foreach (MatchRecord match in played.Where(match => match.Players.Count == 2))
        {
            met.Add((match.Players[0], match.Players[1]));
            met.Add((match.Players[1], match.Players[0]));
        }

        List<(string, string)> chosen = [];
        if (!PairUp(ranked, met, chosen))
        {
            // Everyone left has met everyone else they could be paired with: best with next best, rematches and all.
            chosen.Clear();
            for (int i = 0; i + 1 < ranked.Count; i += 2)
            {
                chosen.Add((ranked[i], ranked[i + 1]));
            }
        }

        pairs.AddRange(chosen.Select(pair => (pair.Item1, (string?)pair.Item2)));
        if (bye is not null)
        {
            pairs.Add((bye, null));
        }

        return pairs;
    }

    /// <summary>
    /// Pairs the best player left with the best they have not met, and so on down, going back to try the next one along
    /// whenever that leaves players below who cannot all be paired.
    /// </summary>
    private static bool PairUp(List<string> left, HashSet<(string, string)> met, List<(string, string)> chosen)
    {
        if (left.Count == 0)
        {
            return true;
        }

        string first = left[0];
        for (int i = 1; i < left.Count; i++)
        {
            string second = left[i];
            if (met.Contains((first, second)))
            {
                continue;
            }

            List<string> rest = [.. left.Where((_, index) => index != 0 && index != i)];
            chosen.Add((first, second));
            if (PairUp(rest, met, chosen))
            {
                return true;
            }

            chosen.RemoveAt(chosen.Count - 1);
        }

        return false;
    }

    /// <summary>
    /// A player's games in a match: won, lost and drawn. A bye is two games won, and a match forfeited is two games
    /// won for whoever was there and two lost for whoever was not.
    /// </summary>
    public static (int Won, int Lost, int Drawn) GamesOf(MatchRecord match, string player)
    {
        ArgumentNullException.ThrowIfNull(match);
        if (match.Bye)
        {
            return (2, 0, 0);
        }

        if (match.Forfeit && match.Games.All(game => game.Ending.Length == 0 || game.Ending == "abandoned"))
        {
            return match.Winners.Contains(player) ? (2, 0, 0) : (0, 2, 0);
        }

        int won = 0, lost = 0, drawn = 0;
        foreach (GameRecord game in match.Games.Where(game => game.Ending is "won" or "draw"))
        {
            if (game.Winners.Contains(player)) won++;
            else if (game.Winners.Count == 0) drawn++;
            else lost++;
        }

        return (won, lost, drawn);
    }

    /// <summary>Percentages compared to the tenth of a percent, as they are printed, so a rounding hair does not decide a place.</summary>
    private static double Round(double value) => Math.Round(value, 3);

    private sealed class Tally
    {
        public int Wins { get; set; }
        public int Losses { get; set; }
        public int Draws { get; set; }
        public int GameWins { get; set; }
        public int GameLosses { get; set; }
        public int GameDraws { get; set; }
        public List<string> Opponents { get; } = [];
    }
}
