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
