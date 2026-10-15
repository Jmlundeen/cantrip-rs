using Cantrip.Server.Events;
using Xunit;

namespace Cantrip.Server.Tests;

public sealed class SwissTests
{
    private static readonly List<string> Eight = ["a", "b", "c", "d", "e", "f", "g", "h"];
    private static readonly HashSet<string> Nobody = [];

    /// <summary>A finished match: the first player named won it two games to <paramref name="lost"/>.</summary>
    private static MatchRecord Won(int round, string winner, string loser, int lost = 0)
    {
        var match = new MatchRecord { Round = round, Players = [winner, loser], Winners = [winner], Complete = true, GamesToWin = 2 };
        for (int i = 0; i < lost; i++) match.Games.Add(new GameRecord { Ending = "won", Winners = [loser] });
        match.Games.Add(new GameRecord { Ending = "won", Winners = [winner] });
        match.Games.Add(new GameRecord { Ending = "won", Winners = [winner] });
        return match;
    }

    [Fact]
    public void Round_one_pairs_across_the_table()
    {
        IReadOnlyList<(string, string?)> pairs = Swiss.Pair(1, Eight, [], Nobody);

        Assert.Equal([("a", "e"), ("b", "f"), ("c", "g"), ("d", "h")], pairs);
    }

    [Fact]
    public void Round_two_pairs_winners_with_winners_and_never_a_rematch()
    {
        List<MatchRecord> one = [Won(1, "a", "e"), Won(1, "b", "f", lost: 1), Won(1, "g", "c"), Won(1, "h", "d", lost: 1)];

        IReadOnlyList<(string First, string? Second)> pairs = Swiss.Pair(2, Eight, one, Nobody);

        Assert.Equal(4, pairs.Count);
        HashSet<string> winners = ["a", "b", "g", "h"];
        foreach ((string first, string? second) in pairs)
        {
            Assert.Equal(winners.Contains(first), winners.Contains(second!));
            Assert.DoesNotContain(one, match => match.Players.Contains(first) && match.Players.Contains(second!));
        }
    }

    [Fact]
    public void Three_rounds_leave_one_player_undefeated_and_standings_break_ties_by_opponents()
    {
        List<MatchRecord> matches = [Won(1, "a", "e"), Won(1, "b", "f"), Won(1, "c", "g"), Won(1, "d", "h")];
        foreach ((string first, string? second) in Swiss.Pair(2, Eight, matches, Nobody))
        {
            // The earlier seat wins every match.
            string winner = string.CompareOrdinal(first, second) < 0 ? first : second!;
            matches.Add(Won(2, winner, winner == first ? second! : first));
        }

        foreach ((string first, string? second) in Swiss.Pair(3, Eight, matches, Nobody))
        {
            string winner = string.CompareOrdinal(first, second) < 0 ? first : second!;
            matches.Add(Won(3, winner, winner == first ? second! : first));
        }

        IReadOnlyList<SwissStanding> standings = Swiss.Standings(Eight, matches, Nobody);
        Assert.Equal("a", standings[0].PlayerId);
        Assert.Equal((9, 3, 0), (standings[0].Points, standings[0].Wins, standings[0].Losses));
        Assert.Single(standings, standing => standing.Points == 9);
        Assert.Equal(Enumerable.Range(1, 8), standings.Select(standing => standing.Rank));
        Assert.All(standings, standing => Assert.InRange(standing.OpponentMatchWin, 1.0 / 3.0, 1.0));
        // No pair met twice.
        Assert.Equal(12, matches.Select(match => string.Join("-", match.Players.Order())).Distinct().Count());
    }

    [Fact]
    public void A_dropped_player_is_not_paired_and_the_odd_one_out_gets_a_bye_that_counts_as_a_win()
    {
        List<MatchRecord> one = [Won(1, "a", "e"), Won(1, "b", "f"), Won(1, "c", "g"), Won(1, "d", "h")];
        HashSet<string> dropped = ["h"];

        IReadOnlyList<(string First, string? Second)> pairs = Swiss.Pair(2, Eight, one, dropped);

        Assert.DoesNotContain(pairs, pair => pair.First == "h" || pair.Second == "h");
        (string bye, _) = Assert.Single(pairs, pair => pair.Second is null);
        // The bye goes to the lowest-placed player still in: one of the round's losers.
        Assert.Contains(bye, new[] { "e", "f", "g" });

        var byeMatch = new MatchRecord { Round = 2, Players = [bye], Winners = [bye], Bye = true, Complete = true };
        SwissStanding standing = Swiss.Standings(Eight, [.. one, byeMatch], dropped).Single(each => each.PlayerId == bye);
        Assert.Equal((3, 1, 1, 2, 2), (standing.Points, standing.Wins, standing.Losses, standing.GameWins, standing.GameLosses));
        Assert.True(Swiss.Standings(Eight, one, dropped)[^1].Dropped);
    }

    [Fact]
    public void A_forfeit_is_two_games_to_none_and_a_double_no_show_is_a_loss_for_both()
    {
        var forfeit = new MatchRecord { Round = 1, Players = ["a", "e"], Winners = ["a"], Forfeit = true, Complete = true };
        var nobody = new MatchRecord { Round = 1, Players = ["b", "f"], Forfeit = true, Complete = true };

        IReadOnlyList<SwissStanding> standings = Swiss.Standings(Eight, [forfeit, nobody], Nobody);

        SwissStanding a = standings.Single(each => each.PlayerId == "a");
        Assert.Equal((3, 2, 0), (a.Points, a.GameWins, a.GameLosses));
        SwissStanding b = standings.Single(each => each.PlayerId == "b");
        Assert.Equal((0, 1, 0, 0, 2), (b.Points, b.Losses, b.Draws, b.GameWins, b.GameLosses));
    }
}
