using System.Net;
using System.Net.Sockets;
using System.Text;
using System.Text.Json;
using Cantrip.Art;
using Cantrip.Client;
using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The leagues on the bot's site, over real tables: a player enters from the league's page, drafts at a table of their
/// own against bots, comes back to find a match, is paired with another player's run and given its table, and ends a
/// run - and their page lists the league. A test sign-in stands in for Discord's.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class LeagueSiteTests : IAsyncLifetime
{
    private const string AsHeader = "X-Test-Player";
    private const string League = "test-quick-draft";

    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-league-site-" + Guid.NewGuid().ToString("N"));
    private readonly string _root = Directory.CreateTempSubdirectory("cantrip-league-client").FullName;
    private readonly HttpClient _http = new(new HttpClientHandler { AllowAutoRedirect = false });
    private PodHosts _pods = null!;
    private LeagueHosts _leagues = null!;
    private Microsoft.AspNetCore.Builder.WebApplication _site = null!;
    private int _port;
    private readonly CancellationTokenSource _players = new();
    private readonly List<Task> _seats = [];

    public async Task InitializeAsync()
    {
        List<int> ports = FreePorts(6);
        _port = ports[0];
        var store = new EventStore(_data);
        _pods = new PodHosts(new PodHostsOptions { ContentRoot = RepoRoot.ContentRoot, Store = store, Ports = ports[1..], Timing = Quick });
        _leagues = new LeagueHosts(new LeagueHostsOptions
        {
            ContentRoot = RepoRoot.ContentRoot,
            Store = store,
            Pods = _pods,
            CubeDirectories = [Path.Combine(RepoRoot.Path, "decks")],
            Leagues =
            [
                new LeagueDefinition
                {
                    Id = League,
                    Name = "Test Quick Draft",
                    Description = "A quick draft of RW Aggro.",
                    CubeName = "RW Aggro",
                    Settings = new LeagueSettings { Cube = "rw-aggro", Packs = 1, PackSize = 5 },
                },
            ],
        });
        Assert.Empty(_leagues.Start());

        await File.WriteAllTextAsync(Path.Combine(_root, "index.html"), "<!doctype html><title>Cantrip</title>");
        var art = new ArtStore(new ArtStoreOptions { ContentRoot = _root, CacheDirectory = Path.Combine(_root, "art") });
        var site = new Uri($"http://127.0.0.1:{_port}/");
        var signIn = new SiteSignInOptions { Site = site, Keys = Path.Combine(_data, "site-keys") };
        var leagues = new LeagueSiteOptions { Leagues = _leagues, Client = site, SignIn = signIn };
        _site = await ClientSite.StartAsync("127.0.0.1", _port, _root, art, signIn: signIn,
            home: new HomeSiteOptions { SignIn = signIn, Leagues = leagues, Store = store }, before: SignInByHeader, leagues: leagues);
    }

    /// <summary>Clocks that draft and build at once, so a quick draft nobody plays is over in a moment.</summary>
    private static BoosterTiming Quick => new() { SecondsPerCard = 0, DeckBuilding = TimeSpan.FromMilliseconds(200) };

    public async Task DisposeAsync()
    {
        await _players.CancelAsync();
        await Task.WhenAll(_seats);
        _http.Dispose();
        await _site.DisposeAsync();
        await _leagues.DisposeAsync();
        await _pods.DisposeAsync();
        Directory.Delete(_root, recursive: true);
        if (Directory.Exists(_data)) Directory.Delete(_data, recursive: true);
    }

    [Fact]
    public async Task A_player_enters_from_the_leagues_page_drafts_against_bots_finds_a_match_and_ends_a_run()
    {
        // Anyone sees the leagues and each league's page; a league that is not there is the site's 404.
        string list = await _http.GetStringAsync(Address("leagues"));
        Assert.Contains("<a href=\"leagues\" aria-current=\"page\">Leagues</a>", list, StringComparison.Ordinal);
        Assert.Equal(HttpStatusCode.OK, (await _http.GetAsync(Address($"leagues/{League}"))).StatusCode);
        Assert.Equal(HttpStatusCode.NotFound, (await _http.GetAsync(Address("leagues/no-such-league"))).StatusCode);
        using (JsonDocument anyone = await Json(null, "leagues/api/list"))
        {
            JsonElement shown = Assert.Single(anyone.RootElement.GetProperty("value").GetProperty("leagues").EnumerateArray());
            Assert.Equal(("Test Quick Draft", "RW Aggro", 7, 3), (shown.GetProperty("name").GetString(), shown.GetProperty("cube").GetString(),
                shown.GetProperty("rules").GetProperty("winsToTrophy").GetInt32(), shown.GetProperty("rules").GetProperty("lossesToEnd").GetInt32()));
            Assert.Equal("betweenMatches", shown.GetProperty("rules").GetProperty("deckEditing").GetString());
            Assert.Equal(JsonValueKind.Null, shown.GetProperty("mine").ValueKind);
        }

        // Signed out, nobody enters.
        Assert.Equal(HttpStatusCode.Unauthorized, (await As(null, HttpMethod.Post, "leagues/api/enter", Body())).StatusCode);

        // Alice and Bob enter: each is given a seat at a draft of their own, with bots in the other seven seats, and
        // sits down at it.
        foreach ((ulong id, string name) in new[] { (1001UL, "Alice"), (1002UL, "Bob") })
        {
            SitDown(await Link(id, name, "leagues/api/enter"), name);
        }

        Assert.Contains("already have a run", await Refusal(1001, "Alice", "leagues/api/enter"), StringComparison.Ordinal);

        // Nobody picks for them, and the clock drafts and builds: their runs play, each with a deck of forty.
        JsonElement mine = await Until(1001, "Alice", mine => mine.GetProperty("current").GetProperty("stage").GetString() == "Playing");
        Assert.Equal(40, mine.GetProperty("current").GetProperty("deck").GetInt32());
        await Until(1002, "Bob", mine => mine.GetProperty("current").GetProperty("stage").GetString() == "Playing");
        string run = mine.GetProperty("current").GetProperty("id").GetString()!;

        // Her deck is hers to see.
        using (JsonDocument deck = await Json(1001, $"leagues/api/deck/{League}?run={run}", "Alice"))
        {
            Assert.Equal(40, deck.RootElement.GetProperty("value").GetProperty("count").GetInt32());
            Assert.True(deck.RootElement.GetProperty("value").GetProperty("editable").GetBoolean());
        }

        Assert.Equal(HttpStatusCode.NotFound, (await As((1002, "Bob"), HttpMethod.Get, $"leagues/api/deck/{League}?run={run}")).StatusCode);

        // Both look for a match, and are paired: Alice is given her match's table.
        Assert.Null(await Refusal(1001, "Alice", "leagues/api/search"));
        Assert.Null(await Refusal(1002, "Bob", "leagues/api/search"));
        mine = await Until(1001, "Alice", mine => mine.GetProperty("matchReady").GetBoolean());
        Assert.Equal("Bob", mine.GetProperty("opponent").GetString());
        Assert.Contains("#ticket=", await Link(1001, "Alice", "leagues/api/play"), StringComparison.Ordinal);
        Assert.Contains("finish it first", await Refusal(1001, "Alice", "leagues/api/end"), StringComparison.Ordinal);

        // Her page lists the league, and the match it wants her at.
        using (JsonDocument board = await Json(1001, "home/api/board?brief=1", "Alice"))
        {
            JsonElement value = board.RootElement.GetProperty("value");
            Assert.True(value.GetProperty("hasLeagues").GetBoolean());
            JsonElement listed = Assert.Single(value.GetProperty("leagues").EnumerateArray());
            Assert.Equal((League, "Bob"), (listed.GetProperty("id").GetString(), listed.GetProperty("mine").GetProperty("opponent").GetString()));
        }

        // Carol has not entered: she cannot look for a match, or end a run.
        Assert.Contains("enter it first", await Refusal(1003, "Carol", "leagues/api/search"), StringComparison.Ordinal);
        Assert.Contains("no run going", await Refusal(1003, "Carol", "leagues/api/end"), StringComparison.Ordinal);
    }

    [Fact]
    public async Task A_run_ended_while_drafting_calls_its_draft_off_and_counts_as_three_losses()
    {
        Assert.Contains("#ticket=", await Link(1001, "Alice", "leagues/api/enter"), StringComparison.Ordinal);
        Assert.True(_pods.InDraft(1001));
        Assert.Null(await Refusal(1001, "Alice", "leagues/api/end"));
        using JsonDocument league = await Json(1001, $"leagues/api/league/{League}", "Alice");
        JsonElement mine = league.RootElement.GetProperty("value").GetProperty("mine");
        Assert.Equal(JsonValueKind.Null, mine.GetProperty("current").ValueKind);
        JsonElement ended = Assert.Single(mine.GetProperty("previous").EnumerateArray());
        Assert.Equal((0, 3, "retired"), (ended.GetProperty("wins").GetInt32(), ended.GetProperty("losses").GetInt32(), ended.GetProperty("ending").GetString()));
        Assert.Equal(JsonValueKind.Null, mine.GetProperty("entryRefusal").ValueKind);
        Assert.False(_pods.InDraft(1001));
    }

    // --- Helpers. ---------------------------------------------------------------------------------------------------

    /// <summary>
    /// Opens a seat's link as a player's browser would - the table in its query, the ticket after its # - and plays it
    /// as nobody in particular does, until the test is over.
    /// </summary>
    private void SitDown(string link, string name)
    {
        var uri = new Uri(link);
        Uri table = new(Uri.UnescapeDataString(uri.Query.TrimStart('?').Split('&').Single(part => part.StartsWith("server=", StringComparison.Ordinal))["server=".Length..]));
        string ticket = Uri.UnescapeDataString(uri.Fragment["#ticket=".Length..]);
        CancellationToken stop = _players.Token;
        _seats.Add(Task.Run(async () =>
        {
            try
            {
                using EventSeat seat = await EventSeat.JoinAsync(table, ticket, name, cancellation: stop);
                while (!stop.IsCancellationRequested && seat.Client.IsConnected)
                {
                    if (!await seat.StepAsync(stop)) await Task.Delay(15, stop);
                }
            }
            catch (Exception error) when (error is OperationCanceledException or System.Net.WebSockets.WebSocketException or IOException)
            {
            }
        }));
    }

    private Uri Address(string path) => new($"http://127.0.0.1:{_port}/{path}");

    private static string Body() => $$"""{"league":"{{League}}"}""";

    private static Task SignInByHeader(Microsoft.AspNetCore.Http.HttpContext context, Func<Task> next)
    {
        if (context.Request.Headers.TryGetValue(AsHeader, out var who) && who.ToString().Split(':', 2) is [{ } id, { } name])
        {
            context.User = new System.Security.Claims.ClaimsPrincipal(new System.Security.Claims.ClaimsIdentity(
                [new(System.Security.Claims.ClaimTypes.NameIdentifier, id), new(System.Security.Claims.ClaimTypes.Name, name)], "test"));
        }

        return next();
    }

    private async Task<HttpResponseMessage> As((ulong Id, string Name)? player, HttpMethod method, string path, string? body = null)
    {
        using var request = new HttpRequestMessage(method, Address(path));
        if (player is { } who) request.Headers.Add(AsHeader, $"{who.Id}:{who.Name}");
        if (body is not null) request.Content = new StringContent(body, Encoding.UTF8, "application/json");
        return await _http.SendAsync(request);
    }

    private async Task<JsonDocument> Json(ulong? player, string path, string name = "")
    {
        HttpResponseMessage response = await As(player is { } id ? (id, name) : null, HttpMethod.Get, path);
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        return JsonDocument.Parse(await response.Content.ReadAsStringAsync());
    }

    /// <summary>A POST about the league, answered with a link to a seat; fails with the refusal if there is none.</summary>
    private async Task<string> Link(ulong id, string name, string path)
    {
        using JsonDocument answer = JsonDocument.Parse(await (await As((id, name), HttpMethod.Post, path, Body())).Content.ReadAsStringAsync());
        JsonElement value = answer.RootElement.GetProperty("value");
        Assert.True(value.GetProperty("link").ValueKind == JsonValueKind.String, value.GetProperty("refusal").GetString());
        return value.GetProperty("link").GetString()!;
    }

    /// <summary>A POST about the league answered with its refusal, or null for none.</summary>
    private async Task<string?> Refusal(ulong id, string name, string path)
    {
        HttpResponseMessage response = await As((id, name), HttpMethod.Post, path, Body());
        Assert.Equal(HttpStatusCode.OK, response.StatusCode);
        using JsonDocument answer = JsonDocument.Parse(await response.Content.ReadAsStringAsync());
        JsonElement refusal = answer.RootElement.GetProperty("value").GetProperty("refusal");
        return refusal.ValueKind == JsonValueKind.Null ? null : refusal.GetString();
    }

    /// <summary>The league as a player sees it, asked again until what is theirs of it is as wanted.</summary>
    private async Task<JsonElement> Until(ulong id, string name, Func<JsonElement, bool> wanted)
    {
        DateTime giveUp = DateTime.UtcNow.AddSeconds(30);
        while (true)
        {
            using JsonDocument league = await Json(id, $"leagues/api/league/{League}", name);
            JsonElement mine = league.RootElement.GetProperty("value").GetProperty("mine");
            if (wanted(mine)) return mine.Clone();
            Assert.True(DateTime.UtcNow < giveUp, $"Still waiting: {mine}");
            await Task.Delay(100);
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
