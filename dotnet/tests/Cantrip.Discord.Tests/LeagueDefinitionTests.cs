using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>The leagues defined under leagues/: each one loads, and its cube has the cards its quick draft deals.</summary>
public sealed class LeagueDefinitionTests
{
    [Fact]
    public void Every_league_defined_loads_and_its_cube_has_enough_playable_cards_for_its_pod()
    {
        (List<LeagueDefinition> leagues, List<string> problems) = LeagueDefinition.LoadAll(Path.Combine(RepoRoot.Path, "leagues"));
        Assert.Empty(problems);
        Assert.Contains(leagues, league => league.Id == "vintage-cube-quick-draft");
        foreach (LeagueDefinition league in leagues)
        {
            LeagueSettings rules = league.Settings;
            string cube = CubeList.Find(rules.Cube, Path.Combine(RepoRoot.Path, "docs", "cube-lists"), Path.Combine(RepoRoot.Path, "decks"));
            int playable = BoosterPod.Playable(ContentCache.Get(RepoRoot.ContentRoot), CubeList.Load(cube));
            Assert.True(playable >= rules.PodSize * rules.Packs * rules.PackSize, $"{league.Id}: {playable} playable cards in {rules.Cube}");
        }
    }

    [Fact]
    public void A_league_file_that_is_not_one_is_said_so_and_not_run()
    {
        string folder = Directory.CreateTempSubdirectory("cantrip-leagues").FullName;
        try
        {
            File.WriteAllText(Path.Combine(folder, "bad.json"), """{ "id": "bad", "name": "Bad", "settings": { "cube": "x", "pairing": "recordStrict" } }""");
            File.WriteAllText(Path.Combine(folder, "good.json"), """{ "id": "good", "name": "Good", "settings": { "cube": "x", "gamesToWin": 2, "maxEntries": 1, "deckEditing": "untilFirstMatch" } }""");
            (List<LeagueDefinition> leagues, List<string> problems) = LeagueDefinition.LoadAll(folder);
            LeagueDefinition good = Assert.Single(leagues);
            Assert.Equal((2, 1, LeagueDeckEditing.UntilFirstMatch), (good.Settings.GamesToWin, good.Settings.MaxEntries, good.Settings.DeckEditing));
            Assert.Contains("not played yet", Assert.Single(problems), StringComparison.Ordinal);
        }
        finally
        {
            Directory.Delete(folder, recursive: true);
        }
    }
}
