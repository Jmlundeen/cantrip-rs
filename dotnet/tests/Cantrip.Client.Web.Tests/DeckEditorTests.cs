using System.Security.Claims;
using Cantrip.Art;
using Cantrip.Discord;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Logging;
using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// The constructed deck editor (<c>?decks</c>) in a browser, over the bot's own deck API (<see cref="DeckSite"/>) and a
/// data directory of its own: a deck built from the search by double-click and by drag, checked as it goes, named and
/// saved, a card dragged back out, the question asked before another deck replaces unsaved changes, a deck opened from
/// the list, a Commander deck's command zone and the color identity it holds the search to, a deck deleted after asking,
/// and the browser asking before a page with unsaved changes is left.
/// <para>
/// Discord's sign-in is Discord's, so here the player is a header every request of the page carries, which a middleware
/// turns into the claims the sign-in cookie would have.
/// </para>
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class DeckEditorTests(WebClient web) : IAsyncLifetime
{
    private const string AsHeader = "X-Test-Account";

    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-decks-" + Guid.NewGuid().ToString("N"));
    private WebApplication _site = null!;

    public async Task InitializeAsync()
    {
        string built = Path.Combine(WebClient.Root, "dotnet", "src", "Cantrip.Client.Web", "bin", "Debug", "net10.0-browserwasm", "wwwroot");
        WebApplicationBuilder builder = WebApplication.CreateBuilder();
        builder.WebHost.UseUrls("http://127.0.0.1:0");
        builder.Logging.ClearProviders();
        WebApplication app = builder.Build();
        app.Use((context, next) =>
        {
            if (context.Request.Headers.TryGetValue(AsHeader, out var id))
            {
                context.User = new ClaimsPrincipal(new ClaimsIdentity([new Claim(ClaimTypes.NameIdentifier, id.ToString()), new Claim(ClaimTypes.Name, "Tester")], "test"));
            }

            return next(context);
        });
        app.UseWebClient(built);
        app.UseRouting();
        app.MapCardArt(new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }));
        app.MapDecks(new DeckSiteOptions
        {
            ContentRoot = WebClient.Content,
            Store = new EventStore(_data),
            SignIn = new SiteSignInOptions { Site = new Uri("http://127.0.0.1/") },
        });
        await app.StartAsync();
        _site = app;
    }

    public async Task DisposeAsync()
    {
        await _site.DisposeAsync();
        if (Directory.Exists(_data)) Directory.Delete(_data, recursive: true);
    }

    [Fact]
    public async Task A_deck_is_built_from_the_search_saved_opened_again_and_deleted_and_Commander_holds_the_search_to_its_identity()
    {
        await using TablePage page = await TablePage.OpenAsync(web.Browser, new Uri(_site.Urls.First() + "/"), "decks", WebClient.Pictures(nameof(DeckEditorTests)),
            headers: [new(AsHeader, "4242")]);
        DeckEditorState editor = await EditorAsync(page, state => state.DeckEditor is { Results: > 0, SignedIn: true });
        Assert.Equal("standard", editor.Format);
        Assert.Empty(editor.Decks);
        await page.PictureAsync("empty");

        // Into the deck by double-click, and by dragging from the results into the columns.
        await page.TypeAsync("Search cards: name, type or rules text", "Dracosaur");
        CardState dracosaur = await SearchCardAsync(page, "Bonehoard Dracosaur");
        await page.DoubleClickCardAsync(dracosaur);
        editor = await EditorAsync(page, state => state.DeckEditor!.Main.Contains("1 Bonehoard Dracosaur"));
        Assert.True(editor.Unsaved);
        Assert.Contains(editor.Problems, problem => problem.Contains("at least 60", StringComparison.Ordinal));

        await ClearSearchAsync(page);
        await page.TypeAsync("Search cards: name, type or rules text", "Clockwork Percussionist");
        CardState percussionist = await SearchCardAsync(page, "Clockwork Percussionist");
        CardState inDeck = (await page.SettledAsync()).Cards.First(card => card.Row == "pool" && card.Name == "Bonehoard Dracosaur");
        await page.DragAsync(percussionist, inDeck.X - (inDeck.W * 4), inDeck.Y + 40);
        editor = await EditorAsync(page, state => state.DeckEditor!.Main.Contains("1 Clockwork Percussionist"));
        Assert.Equal(["1 Bonehoard Dracosaur", "1 Clockwork Percussionist"], editor.Main);
        await page.PictureAsync("two-cards");

        // Named and saved, illegal as it is: listed, and nothing unsaved.
        await page.TypeAsync("Deck name", "Red Test");
        await page.ClickAsync("Save", "button");
        editor = await EditorAsync(page, state => state.DeckEditor!.Decks.Count == 1 && !state.DeckEditor.Unsaved);
        Assert.Equal(["standard: Red Test (2)"], editor.Decks);
        Assert.NotEmpty(editor.Id);

        // A card dragged back onto the search leaves the deck.
        CardState drag = (await page.SettledAsync()).Cards.First(card => card.Row == "pool" && card.Name == "Bonehoard Dracosaur");
        CardState anyResult = (await page.SettledAsync()).Cards.First(card => card.Row == "search");
        await page.DragAsync(drag, anyResult.X, anyResult.Y);
        editor = await EditorAsync(page, state => !state.DeckEditor!.Main.Contains("1 Bonehoard Dracosaur"));
        Assert.True(editor.Unsaved);

        // Another deck asks first: Cancel keeps this one, Discard lets the change go.
        await page.ClickAsync("New deck", "button");
        editor = await EditorAsync(page, state => state.DeckEditor!.Asking);
        Assert.Equal("Save the changes to Red Test?", editor.Question);
        await page.PictureAsync("unsaved-question");
        await page.ClickAsync("Cancel", "button");
        editor = await EditorAsync(page, state => !state.DeckEditor!.Asking);
        Assert.Equal("Red Test", editor.Name);
        await page.ClickAsync("New deck", "button");
        await EditorAsync(page, state => state.DeckEditor!.Asking);
        await page.ClickAsync("Discard", "button");
        editor = await EditorAsync(page, state => !state.DeckEditor!.Asking && state.DeckEditor.Main.Count == 0);
        Assert.Equal(string.Empty, editor.Name);

        // Opened from the list, as it was saved.
        await page.ClickAsync("Open Red Test", "button");
        editor = await EditorAsync(page, state => state.DeckEditor!.Name == "Red Test");
        Assert.Equal(["1 Bonehoard Dracosaur", "1 Clockwork Percussionist"], editor.Main);
        Assert.False(editor.Unsaved);

        // Commander: a commander into the command zone holds the search to its color identity, and a card that can't be
        // a commander is not taken there.
        await page.ChooseAsync("Format", "Commander");
        editor = await EditorAsync(page, state => state.DeckEditor!.Format == "commander" && state.DeckEditor.CommandZone is not null);
        await ClearSearchAsync(page);
        await page.TypeAsync("Search cards: name, type or rules text", "Balin");
        CardState balin = await SearchCardAsync(page, "Balin, Loremaster");
        await page.DragAsync(balin, editor.CommandZone!.X, editor.CommandZone.Y);
        editor = await EditorAsync(page, state => state.DeckEditor!.Commanders.Contains("1 Balin, Loremaster"));
        await page.PictureAsync("commander");

        await ClearSearchAsync(page);
        await page.TypeAsync("Search cards: name, type or rules text", "Badgermole");
        editor = await EditorAsync(page, state => state.DeckEditor!.Results == 0);
        await ClearSearchAsync(page);
        await page.TypeAsync("Search cards: name, type or rules text", "Dracosaur");
        await SearchCardAsync(page, "Bonehoard Dracosaur");

        CardState notLegendary = (await page.SettledAsync()).Cards.First(card => card.Row == "pool" && card.Name == "Clockwork Percussionist");
        await page.DragAsync(notLegendary, editor.CommandZone!.X, editor.CommandZone.Y + 80);
        editor = await EditorAsync(page, state => true);
        Assert.Equal(["1 Balin, Loremaster"], editor.Commanders);
        Assert.Contains("1 Clockwork Percussionist", editor.Main);

        // Deleted after asking; the deck open was that one, so what is in the editor is now a deck not saved.
        await page.ClickAsync("Delete Red Test", "button");
        editor = await EditorAsync(page, state => state.DeckEditor!.Asking);
        Assert.Equal("Delete Red Test?", editor.Question);
        await page.ClickAsync("Delete", "button");
        editor = await EditorAsync(page, state => state.DeckEditor!.Decks.Count == 0 && !state.DeckEditor.Asking);
        Assert.Equal(string.Empty, editor.Id);
        Assert.True(editor.Unsaved);

        // Leaving with changes unsaved, the browser asks.
        string? asked = null;
        page.Page.Dialog += async (_, dialog) =>
        {
            asked = dialog.Type;
            await dialog.DismissAsync();
        };
        await page.Page.CloseAsync(new PageCloseOptions { RunBeforeUnload = true });
        for (int wait = 0; wait < 50 && asked is null; wait++) await Task.Delay(100);
        Assert.Equal("beforeunload", asked);
        Assert.Empty(page.Errors);
    }

    /// <summary>
    /// Every card the search finds can be scrolled to, though only the rows near the view are laid out: the whole of
    /// Vintage, unfiltered, wheeled down to its last card by name, past the 120 the search once stopped at; and a search
    /// typed after that shows its cards from the top.
    /// </summary>
    [Fact]
    public async Task The_whole_search_can_be_scrolled_to_its_last_card_with_only_the_rows_in_view_laid_out()
    {
        await using TablePage page = await TablePage.OpenAsync(web.Browser, new Uri(_site.Urls.First() + "/"), "decks", WebClient.Pictures(nameof(DeckEditorTests) + "-scroll"),
            headers: [new(AsHeader, "4343")]);
        await EditorAsync(page, state => state.DeckEditor is { Results: > 0 });
        await page.ChooseAsync("Format", "Vintage");
        DeckEditorState editor = await EditorAsync(page, state => state.DeckEditor!.Format == "vintage");

        // The whole format is found, far more than a screenful, and only a few rows of it are laid out.
        int vintage = VintageCards();
        Assert.Equal(vintage, editor.Results);
        Assert.InRange(editor.LaidOut, 1, 200);

        TableState state = await page.SettledAsync();
        CardState first = state.Cards.First(card => card.Row == "search" && card.Visible);
        Assert.DoesNotContain(state.Cards, card => card.Row == "search" && card.Name == LastVintage);

        // Down the wheel, until the last card by name is in view.
        CardState? last = null;
        for (int turn = 0; turn < 80 && last is null; turn++)
        {
            await page.Page.Mouse.MoveAsync(first.X, first.Y);
            await page.Page.Mouse.WheelAsync(0, 2000);
            state = await page.SettledAsync();
            last = state.Cards.FirstOrDefault(card => card.Row == "search" && card.Visible && card.Name == LastVintage);
        }

        Assert.NotNull(last);
        editor = state.DeckEditor!;
        Assert.Equal(vintage, editor.Results);
        Assert.InRange(editor.LaidOut, 1, 200);
        Assert.Contains(state.Cards, card => card.Row == "search" && card.Visible && card.Name == "Zuran Orb");
        await page.PictureAsync("scrolled-to-the-end");

        // The last card can be taken into the deck from there, as any other.
        await page.DoubleClickCardAsync(last);
        await EditorAsync(page, state => state.DeckEditor!.Main.Contains($"1 {LastVintage}"));

        // A new search starts from the top.
        await page.TypeAsync("Search cards: name, type or rules text", "Zuran");
        state = await page.SettledAsync(state => state.DeckEditor!.Results == 1);
        Assert.Contains(state.Cards, card => card.Row == "search" && card.Visible && card.Name == "Zuran Orb");
        Assert.Empty(page.Errors);
    }

    /// <summary>
    /// Cards of the deck picked together: a rectangle dragged out from no card over three columns picks the three cards
    /// it touches and not the one beyond; Ctrl-click picks another and lets it go again; the three dragged by one of them
    /// go together into another column, in their order, and then together into the sideboard; a click on no card lets
    /// them go; and over the search a drag from no card draws nothing and picks nothing.
    /// </summary>
    [Fact]
    public async Task Cards_picked_with_a_rectangle_are_dragged_together_and_the_search_picks_none()
    {
        await using TablePage page = await TablePage.OpenAsync(web.Browser, new Uri(_site.Urls.First() + "/"), "decks", WebClient.Pictures(nameof(DeckEditorTests) + "-picks"),
            headers: [new(AsHeader, "4444")]);
        await EditorAsync(page, state => state.DeckEditor is { Results: > 0 });

        // One card each of mana value 1, 2, 3 and 5: four columns.
        (string Search, string Name)[] cards = [("Doombot", "Aerial Doombot"), ("Abrade", "Abrade"), ("Oculus", "Abhorrent Oculus"), ("Aclazotz", "Aclazotz, Deepest Betrayal")];
        foreach ((string search, string name) in cards)
        {
            await ClearSearchAsync(page);
            await page.TypeAsync("Search cards: name, type or rules text", search);
            await page.DoubleClickCardAsync(await SearchCardAsync(page, name));
            await EditorAsync(page, state => state.DeckEditor!.Main.Contains($"1 {name}"));
        }

        CardState doombot = await DeckCardAsync(page, "Aerial Doombot");
        CardState oculus = await DeckCardAsync(page, "Abhorrent Oculus");
        CardState aclazotz = await DeckCardAsync(page, "Aclazotz, Deepest Betrayal");

        // From the gap left of the first column, below its card, up and right into the middle of the third's card.
        float fromX = doombot.X - (doombot.W / 2) - 4;
        float fromY = doombot.Y + (doombot.H / 2) + 30;
        await page.Page.Mouse.MoveAsync(fromX, fromY);
        await page.Page.Mouse.DownAsync();
        await page.Page.Mouse.MoveAsync(oculus.X, oculus.Y, new MouseMoveOptions { Steps = 10 });
        TableState drawing = await page.SettledAsync(state => state.DeckEditor!.Marquee is not null);
        Assert.Empty(drawing.DeckEditor!.Picked!);
        await page.PictureAsync("rectangle");
        await page.Page.Mouse.UpAsync();
        DeckEditorState editor = await EditorAsync(page, state => state.DeckEditor!.Picked!.Count > 0);
        Assert.Equal(["Aerial Doombot", "Abrade", "Abhorrent Oculus"], editor.Picked);
        Assert.Null(editor.Marquee);
        await page.PictureAsync("picked");

        // Ctrl-click picks one more, and lets it go again.
        await page.Page.Keyboard.DownAsync("Control");
        await page.Page.Mouse.ClickAsync(aclazotz.X, aclazotz.Y);
        await EditorAsync(page, state => state.DeckEditor!.Picked!.Count == 4);
        await page.Page.Mouse.ClickAsync(aclazotz.X, aclazotz.Y + 20);
        await page.Page.Keyboard.UpAsync("Control");
        editor = await EditorAsync(page, state => state.DeckEditor!.Picked!.Count == 3);
        Assert.DoesNotContain("Aclazotz, Deepest Betrayal", editor.Picked!);

        // Dragged by the middle one, all three go into Aclazotz's column under it, in their order, and stay picked.
        CardState abrade = await DeckCardAsync(page, "Abrade");
        await page.DragAsync(abrade, aclazotz.X, aclazotz.Y + aclazotz.H);
        TableState moved = await page.SettledAsync(state => state.Cards.Count(card => card.Row == "pool" && MathF.Abs(card.X - aclazotz.X) < 2) == 4);
        Assert.Equal(["Aclazotz, Deepest Betrayal", "Aerial Doombot", "Abrade", "Abhorrent Oculus"],
            moved.Cards.Where(card => card.Row == "pool" && MathF.Abs(card.X - aclazotz.X) < 2).OrderBy(card => card.Y).Select(card => card.Name));
        Assert.Equal(["Aerial Doombot", "Abrade", "Abhorrent Oculus"], moved.DeckEditor!.Picked);
        await page.PictureAsync("moved-together");

        // Into the sideboard together, by the first of them.
        await page.ClickAsync("Show sideboard", "button");
        ControlState hide = (await page.SettledAsync()).Controls.First(control => control.Label == "Hide sideboard");
        await page.DragAsync(await DeckCardAsync(page, "Aerial Doombot"), hide.X, hide.Y + 120);
        editor = await EditorAsync(page, state => state.DeckEditor!.Side.Count == 3);
        Assert.Equal(["1 Aclazotz, Deepest Betrayal"], editor.Main);
        Assert.Equal(["Aerial Doombot", "Abrade", "Abhorrent Oculus"], editor.Picked);
        await page.PictureAsync("into-the-sideboard");

        // A click on no card lets them go.
        aclazotz = await DeckCardAsync(page, "Aclazotz, Deepest Betrayal");
        await page.Page.Mouse.ClickAsync(aclazotz.X + (aclazotz.W * 2), aclazotz.Y);
        editor = await EditorAsync(page, state => state.DeckEditor!.Picked!.Count == 0);

        // Over the search, a drag from no card across a card draws no rectangle, picks nothing and moves nothing.
        await ClearSearchAsync(page);
        await page.TypeAsync("Search cards: name, type or rules text", "Doombot");
        CardState result = await SearchCardAsync(page, "Aerial Doombot");
        await page.Page.Mouse.MoveAsync(result.X + (result.W * 2), result.Y + (result.H / 2) - 5);
        await page.Page.Mouse.DownAsync();
        await page.Page.Mouse.MoveAsync(result.X - (result.W / 2) - 2, result.Y - (result.H / 2) + 2, new MouseMoveOptions { Steps = 10 });
        TableState over = await page.SettledAsync();
        Assert.Null(over.DeckEditor!.Marquee);
        await page.Page.Mouse.UpAsync();
        editor = await EditorAsync(page, state => true);
        Assert.Empty(editor.Picked!);
        Assert.Equal(["1 Aclazotz, Deepest Betrayal"], editor.Main);
        Assert.Equal(3, editor.Side.Count);
        Assert.Empty(page.Errors);
    }

    private static async Task<CardState> DeckCardAsync(TablePage page, string name)
    {
        TableState state = await page.SettledAsync(state => state.Cards.Any(card => (card.Row is "pool" or "sideboard") && card.Visible && card.Name == name));
        return state.Cards.First(card => (card.Row is "pool" or "sideboard") && card.Visible && card.Name == name);
    }

    /// <summary>The last card of Vintage by name, as the search sorts: past Z, by its accented capital.</summary>
    private const string LastVintage = "Óin the Brave";

    /// <summary>How many of the content's cards are legal (or restricted) in Vintage, counted from the files themselves.</summary>
    private static int VintageCards()
    {
        using var legality = System.Text.Json.JsonDocument.Parse(File.ReadAllText(Path.Combine(WebClient.Content, "mtg", "legality.json")));
        System.Text.Json.JsonElement cards = legality.RootElement.GetProperty("cards");
        HashSet<string> names = [];
        foreach (string file in Directory.EnumerateFiles(WebClient.Content, "*.json", SearchOption.AllDirectories).Where(path => path.Contains($"{Path.DirectorySeparatorChar}cards{Path.DirectorySeparatorChar}", StringComparison.Ordinal)))
        {
            using var card = System.Text.Json.JsonDocument.Parse(File.ReadAllText(file));
            if (card.RootElement.ValueKind == System.Text.Json.JsonValueKind.Object && card.RootElement.TryGetProperty("name", out System.Text.Json.JsonElement name))
            {
                names.Add(name.GetString()!);
            }
        }

        return names.Count(name => cards.TryGetProperty(name, out System.Text.Json.JsonElement entry)
            && entry.TryGetProperty("legalities", out System.Text.Json.JsonElement legalities)
            && legalities.TryGetProperty("vintage", out System.Text.Json.JsonElement vintage)
            && vintage.GetString() is "legal" or "restricted");
    }

    private static async Task<DeckEditorState> EditorAsync(TablePage page, Func<TableState, bool> until)
    {
        TableState state = await page.SettledAsync(state => state.DeckEditor is not null && until(state));
        return state.DeckEditor!;
    }

    private static async Task<CardState> SearchCardAsync(TablePage page, string name)
    {
        TableState state = await page.SettledAsync(state => state.Cards.Any(card => card.Row == "search" && card.Name == name));
        return state.Cards.First(card => card.Row == "search" && card.Name == name);
    }

    private static async Task ClearSearchAsync(TablePage page)
    {
        await page.ClickAsync("Search cards: name, type or rules text", "textbox");
        await page.Page.Keyboard.PressAsync("Control+A");
        await page.Page.Keyboard.PressAsync("Backspace");
    }
}
