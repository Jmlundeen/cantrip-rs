using System.Text.Json;
using Cantrip.Art;
using Cantrip.Protocol;
using Cantrip.Protocol.InPerson;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A tablet short of memory, at an in-person table, through a cube's worth of grids: it draws its art at Medium and
/// without animations unless told otherwise, and what it keeps of the cards it has shown - their faces and their art -
/// stays within the device's budgets however many it shows (DeviceClass.cs, FaceCache.cs, CardArt.cs). A device that says
/// nothing of its memory, as the test's browser is made to, keeps the sharpest art and its animations.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class TabletMemoryTests(WebClient web)
{
    /// <summary>What a device of 2 GB says of itself, as an old Android tablet's Chrome does.</summary>
    private const string TwoGigabytes = "Object.defineProperty(Navigator.prototype, 'deviceMemory', { get: () => 2, configurable: true });";

    /// <summary>What a browser that keeps its memory to itself says, as Safari and Firefox do.</summary>
    private const string Unsaid = "delete Navigator.prototype.deviceMemory;";

    private const long Megabyte = 1024 * 1024;

    [Fact]
    public async Task A_tablet_short_of_memory_keeps_a_cube_of_grids_within_its_budgets()
    {
        await using Microsoft.AspNetCore.Builder.WebApplication art = await ArtServer.StartAsync(
            new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }), "127.0.0.1", 0);
        string query = $"art={Uri.EscapeDataString(art.Urls.First())}";
        await using TabletDriver tablet = await TabletDriver.OpenAsync(web, query, WebClient.Pictures(nameof(TabletMemoryTests)), "constrained", init: TwoGigabytes);
        await tablet.FeedAsync(new DeviceWelcomeMessage { Role = DeviceRole.Tablet, Side = 0, PlayerNames = ["Bot1", "Bot2"] });

        const int grids = 40;
        List<string> names = CardsWithArt(grids * 9);
        long mostDurable = 0;
        long mostArt = 0;
        for (int grid = 1; grid <= grids; grid++)
        {
            await tablet.FeedAsync(new PublicGridMessage
            {
                Decision = grid * 2 - 1,
                Grid = grid,
                Grids = grids,
                Pick = 1,
                Active = 1 + (grid % 2),
                Places = [.. names.Skip((grid - 1) * 9).Take(9).Select(name => new ObjectView { Name = name, Known = true })],
                Drafted = [grid, grid],
            });

            TabletState state = await tablet.SettledAsync(each => each.Screen == "grid" && each.Grid?.Grid == grid);
            mostDurable = Math.Max(mostDurable, state.DurableBytes);
            mostArt = Math.Max(mostArt, state.ArtBytes);
        }

        TabletState last = await tablet.SettledAsync();
        await tablet.PictureAsync("last-grid");

        Assert.True(last.Constrained);
        Assert.Equal("Medium", last.ArtQuality);
        Assert.False(last.Animates);

        // Art arrived: the grid is drawn with it, and some of what came was let go to stay within the budget.
        Assert.True(mostArt > 0, "No art arrived for the grids.");
        Assert.True(mostDurable <= 32 * Megabyte, $"The durable faces held {mostDurable / Megabyte} MB, over the 32 MB a constrained device keeps.");
        Assert.True(mostArt <= 24 * Megabyte, $"The art held {mostArt / Megabyte} MB, over the 24 MB a constrained device keeps.");
        Assert.Empty(tablet.Errors);
    }

    [Fact]
    public async Task A_tablet_that_says_nothing_of_its_memory_keeps_the_sharpest_art_and_its_animations()
    {
        await using TabletDriver tablet = await TabletDriver.OpenAsync(web, string.Empty, WebClient.Pictures(nameof(TabletMemoryTests)), "unsaid", init: Unsaid);
        await tablet.FeedAsync(new DeviceWelcomeMessage { Role = DeviceRole.Tablet, Side = 0, PlayerNames = ["Bot1", "Bot2"] });

        TabletState state = await tablet.SettledAsync();

        Assert.False(state.Constrained);
        Assert.Equal("High", state.ArtQuality);
        Assert.True(state.Animates);
    }

    /// <summary>The names of the first cards, in file order, that have art to draw.</summary>
    private static List<string> CardsWithArt(int count)
    {
        string images = Path.Combine(WebClient.Content, "mtg", "images");
        string cards = Path.Combine(WebClient.Content, "mtg", "cards");
        List<string> names = [];
        foreach (string image in Directory.EnumerateFiles(images, "*.jpg").Order(StringComparer.Ordinal))
        {
            string card = Path.Combine(cards, Path.GetFileNameWithoutExtension(image) + ".json");
            if (File.Exists(card))
            {
                using JsonDocument json = JsonDocument.Parse(File.ReadAllText(card));
                names.Add(json.RootElement.GetProperty("name").GetString()!);
                if (names.Count == count)
                {
                    return names;
                }
            }
        }

        Assert.Fail($"Only {names.Count} cards have art here; the test wants {count} (tools: fetch-card-art).");
        return names;
    }
}
