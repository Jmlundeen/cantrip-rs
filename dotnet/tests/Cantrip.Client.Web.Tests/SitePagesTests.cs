using System.Net;
using System.Net.Sockets;
using System.Security.Claims;
using Cantrip.Art;
using Cantrip.Client.Decks;
using Cantrip.Discord;
using Cantrip.Server;
using Cantrip.Server.Events;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// The bot's site in a browser, as its players use it: every page but a game table, at a desktop's width and a phone's,
/// signed out and signed in, each with the same menu bar - the logo, the sections, who is signed in - and footer; the
/// way from page to page by that bar, and on a phone by its menu; a player's own page with what they are playing and
/// their events over; dropping out of a draft from it, asked first; and the deck library, opening a deck in the editor.
/// Every page is pictured (TestResults\web\SitePagesTests) to be looked at.
/// <para>
/// Discord's sign-in is Discord's, so here the player is a header every request of the page carries, which the site's
/// test hook (<c>before</c>) turns into the claims the sign-in cookie would have.
/// </para>
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class SitePagesTests(WebClient web) : IAsyncLifetime
{
    private const string AsHeader = "X-Test-Account";
    private const ulong Alice = 1001;

    private static readonly (int Width, int Height, string Name)[] Sizes = [(1440, 900, "desktop"), (390, 844, "phone")];

    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-site-pages-" + Guid.NewGuid().ToString("N"));
    private readonly string _pictures = WebClient.Pictures(nameof(SitePagesTests));
    private readonly List<string> _errors = [];
    private WebApplication _site = null!;
    private DraftHost _drafts = null!;
    private DraftQueues _queues = null!;
    private Matchmaker _games = null!;
    private Uri _address = null!;
    private string _live = string.Empty;
    private string _finished = string.Empty;

    public async Task InitializeAsync()
    {
        string built = Path.Combine(WebClient.Root, "dotnet", "src", "Cantrip.Client.Web", "bin", "Debug", "net10.0-browserwasm", "wwwroot");
        List<int> ports = FreePorts(8);
        var store = new EventStore(_data);
        IReadOnlyList<string> cube = CubeList.Load(Path.Combine(WebClient.Root, "decks", "rw-aggro.txt"));
        _drafts = new DraftHost(new DraftHostOptions { ContentRoot = WebClient.Content, Store = store, Cube = "LSVCube", CubeCards = cube, Cards = 18, Ports = ports[1..4] });
        _queues = new DraftQueues(new DraftQueuesOptions
        {
            ContentRoot = WebClient.Content, Store = store, Drafts = _drafts, Ports = ports[1..4], CubeDirectories = [Path.Combine(WebClient.Root, "docs", "cube-lists")],
        });
        string decks = Path.Combine(WebClient.Root, "decks");
        DeckList rwAggro = DeckList.Load(DeckList.Find(decks, "rw-aggro"));
        _games = new Matchmaker(new MatchmakerOptions
        {
            ContentRoot = WebClient.Content,
            Deck = rwAggro,
            Formats = new Dictionary<string, IReadOnlyList<DeckList>> { ["standard"] = [rwAggro, DeckList.Load(DeckList.Find(decks, "ub-reanimator"))] },
            Ports = ports[4..],
        });

        var site = new Uri($"http://127.0.0.1:{ports[0]}/");
        var signIn = new SiteSignInOptions { Site = site, ClientId = "1234567890", ClientSecret = "not-a-secret", Keys = Path.Combine(_data, "site-keys") };
        var status = new EventStatusBoard(store, hosting: () => [.. _drafts.Drafts.Where(draft => draft.IsLive).Select(draft => draft.Id)]);
        var queuePage = new QueueSiteOptions { Queues = _queues, Board = status, Drafts = _drafts, Client = site, SignIn = signIn };
        var playPage = new PlaySiteOptions { Games = _games, Client = site, SignIn = signIn };
        var deckPage = new DeckSiteOptions { ContentRoot = WebClient.Content, Store = store, SignIn = signIn };
        var home = new HomeSiteOptions { SignIn = signIn, Drafts = queuePage, Constructed = playPage, HasDecks = true, Store = store };
        var art = new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache });
        _site = await ClientSite.StartAsync("127.0.0.1", ports[0], built, art, status: status, signIn: signIn, queues: queuePage, play: playPage, home: home, decks: deckPage,
            before: SignInByHeader);
        _address = site;

        // Alice's history: a Winston draft she won, and one called off when Bob dropped; and three decks.
        _finished = Finished(store, cube);
        DraftSeating called = await _drafts.OpenAsync(new DiscordPlayer(Alice, "Alice"));
        await _drafts.JoinAsync(called.Draft!.Id, new DiscordPlayer(1002, "Bob"));
        Assert.Null(await _drafts.DropAsync(called.Draft.Id, new DiscordPlayer(1002, "Bob")));
        var library = new DeckStore(store);
        library.Save(Alice, new SavedDeck { Name = "Boros Burn", Format = "modern", Main = [new DeckEntry("Mountain", 20), new DeckEntry("Lightning Bolt", 4)] });
        library.Save(Alice, new SavedDeck { Name = "Mono-Red Aggro", Format = "standard", Main = [new DeckEntry("Mountain", 24)] });
        library.Save(Alice, new SavedDeck { Name = "Aang's Journey", Format = "commander", Commanders = [new DeckEntry("Aang, at the Crossroads", 1)], Main = [new DeckEntry("Forest", 99)] });

        // Being played: a Winston draft against Carol, and a table that is filling for anyone to see.
        DraftSeating live = await _drafts.OpenAsync(new DiscordPlayer(Alice, "Alice"));
        Assert.Null((await _drafts.JoinAsync(live.Draft!.Id, new DiscordPlayer(1003, "Carol"))).Refusal);
        _live = live.Draft.Id;
        Assert.Null((await _queues.CreateTableAsync(new QueueSettings { Format = "grid", Cube = "LSVCube", MinimumDeck = 40 }, new DiscordPlayer(1004, "Dana"))).Refusal);
        Assert.Null((await _games.HostAsync(new DiscordPlayer(1005, "Erin"), "standard", "rw-aggro")).Refusal);
    }

    public async Task DisposeAsync()
    {
        await _site.DisposeAsync();
        await _queues.DisposeAsync();
        await _drafts.DisposeAsync();
        await _games.DisposeAsync();
        if (Directory.Exists(_data)) Directory.Delete(_data, recursive: true);
    }

    [Fact]
    public async Task Every_page_shares_one_menu_bar_and_footer_at_desktop_and_phone_width_signed_out_and_in()
    {
        string[] pages = ["", "about", "home", "draft", "constructed", "decks", "status", $"status#event={_live}", "no-such-page"];
        foreach ((int width, int height, string size) in Sizes)
        {
            foreach (bool signedIn in new[] { false, true })
            {
                IPage page = await OpenAsync(width, height, signedIn);
                foreach (string path in pages.Where(path => !(signedIn && path.Length == 0)))
                {
                    await page.GotoAsync(new Uri(_address, path).AbsoluteUri);
                    await page.WaitForLoadStateAsync(LoadState.NetworkIdle);
                    await Task.Delay(300);

                    // The logo is the bar's way home, and every section is in it; the page's own is lit.
                    ILocator logo = page.Locator(".site-header .site-logo img");
                    Assert.Equal("Cantrip", await logo.GetAttributeAsync("alt"));
                    Assert.True(await logo.IsVisibleAsync());
                    Assert.Equal(1, await page.Locator(".site-footer").CountAsync());
                    Assert.Equal(signedIn ? 1 : 0, await page.Locator(".site-header .site-me").CountAsync());
                    Assert.Equal(signedIn ? 0 : 1, await page.Locator(".site-header [data-site-signin]").CountAsync());

                    // Nothing wider than the window.
                    int scroll = await page.EvaluateAsync<int>("document.documentElement.scrollWidth");
                    Assert.True(scroll <= width, $"{path} at {size} is {scroll} wide");

                    string name = (path.Length == 0 ? "landing" : path.Replace("#event=", "-event-", StringComparison.Ordinal).Replace('/', '-')) + "-" + size + (signedIn ? "-in" : "-out");
                    await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, name + ".png"), FullPage = true });
                }

                if (width < 600)
                {
                    // On a phone the sections are behind the menu button.
                    await page.GotoAsync(new Uri(_address, "draft").AbsoluteUri);
                    Assert.False(await page.Locator(".site-nav a[href='decks']").IsVisibleAsync());
                    await page.Locator("[data-site-menu]").ClickAsync();
                    Assert.Equal("true", await page.Locator("[data-site-menu]").GetAttributeAsync("aria-expanded"));
                    await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, $"menu-{size}{(signedIn ? "-in" : "-out")}.png") });
                    await page.Locator(".site-nav a[href='decks']").ClickAsync();
                    await page.WaitForURLAsync("**/decks");
                }

                await page.Context.CloseAsync();
            }
        }

        Assert.Empty(_errors);
    }

    [Fact]
    public async Task A_player_finds_everything_from_the_menu_bar_and_drops_out_of_their_draft_from_their_page_after_saying_so()
    {
        IPage page = await OpenAsync(1440, 900, signedIn: true);

        // The front door goes to the player's own page: what they're playing, and what they've played.
        await page.GotoAsync(_address.AbsoluteUri);
        await page.WaitForURLAsync("**/home");
        await page.Locator("h1", new PageLocatorOptions { HasText = "Welcome, Alice" }).WaitForAsync();
        ILocator playing = page.Locator(".card", new PageLocatorOptions { HasText = "Winston draft: LSVCube" }).First;
        await playing.WaitForAsync();
        Assert.Equal(2, await page.Locator("#content .rows .row").CountAsync());
        Assert.True(await page.Locator(".row", new PageLocatorOptions { HasText = "Won" }).IsVisibleAsync());
        Assert.True(await page.Locator(".row", new PageLocatorOptions { HasText = "Called off" }).IsVisibleAsync());

        // Every section is a click away, and comes back.
        foreach ((string link, string url, string heading) in new[] { ("Draft", "**/draft", "Draft"), ("Constructed", "**/constructed", "Constructed"), ("Decks", "**/decks", "Decks"), ("Events", "**/status", "Events"), ("Home", "**/home", "Welcome, Alice") })
        {
            await page.Locator(".site-nav a", new PageLocatorOptions { HasText = link }).ClickAsync();
            await page.WaitForURLAsync(url);
            await page.Locator("h1", new PageLocatorOptions { HasText = heading }).WaitForAsync();
        }

        // A draft being played shows on every other page, with the way back to the seat.
        await page.Locator(".site-nav a", new PageLocatorOptions { HasText = "Decks" }).ClickAsync();
        await page.Locator("#site-banner", new PageLocatorOptions { HasText = "Back to my draft" }).WaitForAsync();
        await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, "banner-on-decks.png") });

        // The deck library opens a deck in the editor.
        Assert.Equal(3, await page.Locator("#content .row").CountAsync());
        string? edit = await page.Locator("a[aria-label='Edit Boros Burn']").GetAttributeAsync("href");
        Assert.StartsWith("decks/edit?deck=", edit, StringComparison.Ordinal);

        // Dropping is asked first: no keeps the seat.
        await page.Locator(".site-nav a", new PageLocatorOptions { HasText = "Home" }).ClickAsync();
        await playing.WaitForAsync();
        await playing.Locator("button", new() { HasText = "Drop" }).ClickAsync();
        ILocator dialog = page.Locator("dialog[open]");
        await dialog.WaitForAsync();
        Assert.Contains("Drop from Winston draft: LSVCube?", await dialog.Locator("h2").InnerTextAsync(), StringComparison.Ordinal);
        await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, "drop-question.png") });
        await dialog.Locator("button", new() { HasText = "Cancel" }).ClickAsync();
        Assert.True(_drafts.InDraft(Alice));

        // Yes drops her: the draft is over, listed with the others, and she is free to play again.
        await playing.Locator("button", new() { HasText = "Drop" }).ClickAsync();
        await page.Locator("dialog[open] button", new() { HasText = "Drop" }).ClickAsync();
        await page.Locator(".site-toast", new() { HasText = "You dropped from Winston draft: LSVCube." }).WaitForAsync();
        await page.Locator("#content .rows .row").Nth(2).WaitForAsync();
        Assert.False(_drafts.InDraft(Alice));
        Assert.Equal(3, await page.Locator("#content .rows .row").CountAsync());
        Assert.True(await page.Locator(".empty", new() { HasText = "You're not in anything right now." }).IsVisibleAsync());
        await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, "after-drop.png"), FullPage = true });

        // An event's results are a click away.
        await page.Locator("#content .rows .row a", new() { HasText = "Winston draft" }).First.ClickAsync();
        await page.WaitForURLAsync("**/status#event=*");
        await page.Locator("#drawer.on").WaitForAsync();
        await page.ScreenshotAsync(new PageScreenshotOptions { Path = Path.Combine(_pictures, "results.png") });

        // Signing out goes back to the front door, signed out.
        await page.GotoAsync(new Uri(_address, "draft").AbsoluteUri);
        await page.Context.SetExtraHTTPHeadersAsync([]);
        await page.Locator("[data-site-signout]").ClickAsync();
        await page.WaitForURLAsync(_address.AbsoluteUri);
        await page.Locator(".hero h1").WaitForAsync();
        Assert.Empty(_errors);
    }

    private async Task<IPage> OpenAsync(int width, int height, bool signedIn)
    {
        IBrowserContext context = await web.Browser.NewContextAsync(new BrowserNewContextOptions
        {
            ViewportSize = new ViewportSize { Width = width, Height = height },
            ExtraHTTPHeaders = signedIn ? [new(AsHeader, $"{Alice}:Alice")] : [],
        });
        IPage page = await context.NewPageAsync();
        page.Console += (_, message) =>
        {
            if (message.Type == "error" && !message.Text.Contains("404", StringComparison.Ordinal)) _errors.Add($"{page.Url}: {message.Text}");
        };
        page.PageError += (_, error) => _errors.Add($"{page.Url}: {error}");
        return page;
    }

    /// <summary>A Winston draft Alice won against Bob, as the data directory keeps one over.</summary>
    private static string Finished(EventStore store, IReadOnlyList<string> cube)
    {
        CreatedEvent created = DraftPod.CreateWinston(store, Cantrip.Server.ContentCache.Get(WebClient.Content), new WinstonDraftSetup
        {
            Name = "Winston draft: LSVCube", Cube = "LSVCube", CubeCards = cube, Cards = 18, MinimumDeck = 10, Seed = 3,
            Players = [($"discord-{Alice}", "Alice"), ("discord-1002", "Bob")],
        });
        EventRecord record = store.LoadEvent(created.Record.Id)!;
        DateTimeOffset now = DateTimeOffset.UtcNow;
        record.Stage = EventStage.Complete;
        record.Round = 1;
        record.Detail = "Alice won round 1.";
        record.Matches.Add(new MatchRecord
        {
            Id = "r1", Round = 1, Players = [$"discord-{Alice}", "discord-1002"], Winners = [$"discord-{Alice}"], Complete = true,
            Games = [new GameRecord { Number = 1, Started = now.AddMinutes(-20), Ended = now.AddMinutes(-2), Ending = "won", Winners = [$"discord-{Alice}"], Detail = "Bob lost." }],
        });
        store.SaveEvent(record);
        return record.Id;
    }

    private static Task SignInByHeader(HttpContext context, Func<Task> next)
    {
        if (context.Request.Headers.TryGetValue(AsHeader, out var who) && who.ToString().Split(':', 2) is [{ } id, { } name])
        {
            context.User = new ClaimsPrincipal(new ClaimsIdentity([new Claim(ClaimTypes.NameIdentifier, id), new Claim(ClaimTypes.Name, name)], "test"));
        }

        return next();
    }

    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }
}
