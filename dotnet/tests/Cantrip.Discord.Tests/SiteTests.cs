using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using Cantrip.Art;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The draft queue and constructed play pages as the bot serves them, over real HTTP with real tables behind them: what
/// anyone may see of the queues, the tables forming and the games, that taking a seat wants a Discord sign-in, and that
/// signing in goes to Discord asking only who the player is, to come back to the address registered for it.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class SiteTests : IDisposable
{
    private const ulong Channel = 4242;
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-site-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data)) Directory.Delete(_data, recursive: true);
    }

    [Fact]
    public async Task The_queue_page_shows_every_queue_and_table_by_name_and_a_seat_wants_signing_in()
    {
        await using Fixture fixture = await Fixture.StartAsync(_data, signIn: false);
        Assert.Null(await fixture.Queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "grid", Packs = 4, MinimumDeck = 10 }));
        QueueSeating alice = await fixture.Queues.CreateTableAsync(new QueueSettings { Cube = "rw-aggro", Cards = 18, MinimumDeck = 20 }, new DiscordPlayer(1001, "Alice"));
        Assert.Null(alice.Refusal);

        HttpResponseMessage page = await fixture.Http.GetAsync(fixture.Address("queue"));
        Assert.Equal(HttpStatusCode.OK, page.StatusCode);
        Assert.Contains("Draft Queues", await page.Content.ReadAsStringAsync(), StringComparison.Ordinal);

        string text = await fixture.Http.GetStringAsync(fixture.Address("queue/api/board"));
        using JsonDocument board = JsonDocument.Parse(text);
        JsonElement value = board.RootElement.GetProperty("value");
        Assert.False(value.GetProperty("signIn").GetBoolean());
        Assert.Equal(JsonValueKind.Null, value.GetProperty("me").ValueKind);
        Assert.Contains("rw-aggro", value.GetProperty("cubes").EnumerateArray().Select(cube => cube.GetString()));

        // The channel's id as text, past what a number in JavaScript holds; the table of Alice's own in no channel.
        JsonElement queue = Assert.Single(value.GetProperty("queues").EnumerateArray());
        Assert.Equal(Channel.ToString(System.Globalization.CultureInfo.InvariantCulture), queue.GetProperty("id").GetString());
        Assert.Equal("grid", queue.GetProperty("settings").GetProperty("format").GetString());
        JsonElement table = Assert.Single(value.GetProperty("tables").EnumerateArray());
        Assert.Equal(JsonValueKind.Null, table.GetProperty("queue").ValueKind);
        Assert.Equal(["Alice"], table.GetProperty("players").EnumerateArray().Select(player => player.GetString()));
        Assert.Equal(2, table.GetProperty("seats").GetInt32());

        // Names only: no player id, and no ticket.
        Assert.DoesNotContain("discord-", text, StringComparison.Ordinal);
        Assert.DoesNotContain(alice.Ticket!, text, StringComparison.Ordinal);

        // Nobody signed in: no seat, at the table or from the queue, and no table of one's own.
        foreach ((string path, string body) in new[]
        {
            ("queue/api/join", $$"""{"table":"{{table.GetProperty("id").GetString()}}"}"""),
            ("queue/api/join", $$"""{"queue":"{{Channel}}"}"""),
            ("queue/api/create", """{"format":"winston","cube":"rw-aggro","cards":18}"""),
            ("queue/api/return", "{}"),
        })
        {
            HttpResponseMessage answer = await fixture.Http.PostAsync(fixture.Address(path), new StringContent(body, Encoding.UTF8, "application/json"));
            Assert.Equal(HttpStatusCode.Unauthorized, answer.StatusCode);
        }

        // Without sign-in set up, Sign in only comes back to the page.
        HttpResponseMessage signIn = await fixture.Http.GetAsync(fixture.Address("signin?then=queue"));
        Assert.Equal(HttpStatusCode.Redirect, signIn.StatusCode);
        Assert.Equal("/queue", signIn.Headers.Location!.OriginalString);
    }

    [Fact]
    public async Task The_play_page_lists_each_formats_decks_and_its_games_by_their_players_names_alone()
    {
        await using Fixture fixture = await Fixture.StartAsync(_data, signIn: false);
        Seating hosted = await fixture.Games.HostAsync(new DiscordPlayer(1001, "Alice"), "standard", "ub-reanimator");
        Assert.Null(hosted.Refusal);
        await fixture.Games.HostAsync(new DiscordPlayer(1002, "Bob"));

        HttpResponseMessage page = await fixture.Http.GetAsync(fixture.Address("play"));
        Assert.Equal(HttpStatusCode.OK, page.StatusCode);
        Assert.Contains("Constructed Play", await page.Content.ReadAsStringAsync(), StringComparison.Ordinal);

        string text = await fixture.Http.GetStringAsync(fixture.Address("play/api/board"));
        using JsonDocument board = JsonDocument.Parse(text);
        JsonElement value = board.RootElement.GetProperty("value");
        JsonElement format = Assert.Single(value.GetProperty("formats").EnumerateArray());
        Assert.Equal("standard", format.GetProperty("key").GetString());
        Assert.Equal(["rw-aggro", "ub-reanimator"], format.GetProperty("decks").EnumerateArray().Select(deck => deck.GetProperty("name").GetString()).Order());

        // Alice's game, waiting, with her name and not her deck; Bob's game from Discord is not the page's.
        JsonElement game = Assert.Single(value.GetProperty("games").EnumerateArray());
        Assert.Equal((hosted.Game!.Id, "Open"), (game.GetProperty("id").GetString(), game.GetProperty("stage").GetString()));
        Assert.Equal(["Alice"], game.GetProperty("players").EnumerateArray().Select(player => player.GetString()));
        Assert.False(game.TryGetProperty("deck", out _));
        Assert.DoesNotContain(hosted.Ticket!, text, StringComparison.Ordinal);

        foreach ((string path, string body) in new[]
        {
            ("play/api/host", """{"format":"standard","deck":"rw-aggro"}"""),
            ("play/api/join", $$"""{"game":"{{hosted.Game.Id}}","deck":"rw-aggro"}"""),
            ("play/api/cancel", $$"""{"game":"{{hosted.Game.Id}}"}"""),
        })
        {
            HttpResponseMessage answer = await fixture.Http.PostAsync(fixture.Address(path), new StringContent(body, Encoding.UTF8, "application/json"));
            Assert.Equal(HttpStatusCode.Unauthorized, answer.StatusCode);
        }
    }

    [Fact]
    public void A_deck_is_named_as_its_list_names_it_or_from_its_file()
    {
        string decks = Path.Combine(RepoRoot.Path, "decks");
        Assert.Equal("Mono-Red Aggro", PlaySite.Title(DeckList.Load(Path.Combine(decks, "standard", "mono-red-aggro.txt"))));
        Assert.Equal("Rw Aggro", PlaySite.Title(DeckList.Parse("rw-aggro", "4 Mountain", Path.Combine(decks, "no-such-file.txt"))));
    }

    [Fact]
    public async Task Signing_in_goes_to_Discord_for_the_accounts_identity_alone_and_back_to_the_registered_address()
    {
        await using Fixture fixture = await Fixture.StartAsync(_data, signIn: true);
        HttpResponseMessage answer = await fixture.Http.GetAsync(fixture.Address("signin?then=play"));
        Assert.Equal(HttpStatusCode.Redirect, answer.StatusCode);
        Uri discord = answer.Headers.Location!;
        Assert.Equal("https://discord.com/oauth2/authorize", discord.GetLeftPart(UriPartial.Path));

        Dictionary<string, string> query = discord.Query.TrimStart('?').Split('&')
            .Select(pair => pair.Split('=', 2)).ToDictionary(pair => pair[0], pair => Uri.UnescapeDataString(pair[1]));
        Assert.Equal("1234567890", query["client_id"]);
        Assert.Equal("identify", query["scope"]);
        Assert.Equal("code", query["response_type"]);
        Assert.Equal("S256", query["code_challenge_method"]);
        Assert.NotEmpty(query["state"]);

        // Back to the site's public address, whatever address the request came in at.
        Assert.Equal("https://cantrip.example.com/signin/discord", query["redirect_uri"]);
        Assert.Equal(new Uri("https://cantrip.example.com/signin/discord"), SiteSignIn.Callback(new Uri("https://cantrip.example.com/")));

        // The secret goes to Discord from the server alone, never through the browser.
        Assert.DoesNotContain("not-a-secret", discord.AbsoluteUri, StringComparison.Ordinal);

        // Still a 401 for a seat, rather than a redirect the page's fetch would follow.
        HttpResponseMessage seat = await fixture.Http.PostAsync(fixture.Address("queue/api/return"), new StringContent("{}", Encoding.UTF8, "application/json"));
        Assert.Equal(HttpStatusCode.Unauthorized, seat.StatusCode);
    }

    /// <summary>The bot's site with both pages on it, over queues and games with real tables, and a client that follows no redirect.</summary>
    private sealed class Fixture : IAsyncDisposable
    {
        private readonly string _root;
        private readonly int _port;
        private readonly Microsoft.AspNetCore.Builder.WebApplication _site;
        private readonly DraftHost _drafts;

        private Fixture(string root, int port, Microsoft.AspNetCore.Builder.WebApplication site, DraftQueues queues, DraftHost drafts, Matchmaker games)
        {
            _root = root;
            _port = port;
            _site = site;
            Queues = queues;
            _drafts = drafts;
            Games = games;
        }

        public DraftQueues Queues { get; }

        public Matchmaker Games { get; }

        public HttpClient Http { get; } = new(new HttpClientHandler { AllowAutoRedirect = false });

        public Uri Address(string path) => new($"http://127.0.0.1:{_port}/{path}");

        public static async Task<Fixture> StartAsync(string data, bool signIn)
        {
            List<int> ports = FreePorts(6);
            var store = new EventStore(data);
            var drafts = new DraftHost(new DraftHostOptions { ContentRoot = RepoRoot.ContentRoot, Store = store, Cube = "rw-aggro", CubeCards = [], Ports = ports[1..4] });
            var queues = new DraftQueues(new DraftQueuesOptions
            {
                ContentRoot = RepoRoot.ContentRoot, Store = store, Drafts = drafts, Ports = ports[1..4], CubeDirectories = [Path.Combine(RepoRoot.Path, "decks")],
            });
            string decks = Path.Combine(RepoRoot.Path, "decks");
            DeckList rwAggro = DeckList.Load(DeckList.Find(decks, "rw-aggro"));
            var games = new Matchmaker(new MatchmakerOptions
            {
                ContentRoot = RepoRoot.ContentRoot,
                Deck = rwAggro,
                Formats = new Dictionary<string, IReadOnlyList<DeckList>> { ["standard"] = [rwAggro, DeckList.Load(DeckList.Find(decks, "ub-reanimator"))] },
                Ports = ports[4..],
            });

            string root = Directory.CreateTempSubdirectory("cantrip-site-client").FullName;
            await File.WriteAllTextAsync(Path.Combine(root, "index.html"), "<!doctype html><title>Cantrip</title>");
            var art = new ArtStore(new ArtStoreOptions { ContentRoot = root, CacheDirectory = Path.Combine(root, "art") });
            var site = new Uri(signIn ? "https://cantrip.example.com/" : $"http://127.0.0.1:{ports[0]}/");
            var account = new SiteSignInOptions
            {
                Site = site,
                ClientId = signIn ? "1234567890" : null,
                ClientSecret = signIn ? "not-a-secret" : null,
                Keys = Path.Combine(data, "site-keys"),
            };
            var queuePage = new QueueSiteOptions
            {
                Queues = queues,
                Board = new EventStatusBoard(store, hosting: () => [.. drafts.Drafts.Select(draft => draft.Id)]),
                Drafts = drafts,
                Client = site,
                SignIn = account,
            };
            var playPage = new PlaySiteOptions { Games = games, Client = site, SignIn = account };
            Microsoft.AspNetCore.Builder.WebApplication web = await ClientSite.StartAsync("127.0.0.1", ports[0], root, art, signIn: account, queues: queuePage, play: playPage);
            return new Fixture(root, ports[0], web, queues, drafts, games);
        }

        public async ValueTask DisposeAsync()
        {
            Http.Dispose();
            await _site.DisposeAsync();
            await Queues.DisposeAsync();
            await _drafts.DisposeAsync();
            await Games.DisposeAsync();
            Directory.Delete(_root, recursive: true);
        }
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
