using Cantrip.Discord;
using Cantrip.Server;
using Cantrip.Server.Events;

// The Discord bot. Members with a host role run /cantrip host in a channel; the bot opens a table and posts a message
// with a Join button, and each of the two players gets a private ticket that launches the client into their seat.
//
// Usage: cantrip-discord --guild ID [--host-role ID]... [--listen ADDRESS] [--public-host HOST] [--ports FIRST-LAST]
//                        [--web-port N] [--client-root PATH | --client URL [--art URL]] [--deck NAME] [--content PATH]
//
// The bot token comes from the CANTRIP_DISCORD_TOKEN environment variable and nowhere else, so it never sits in a
// shell history or a process list. Without --host-role only members with Manage Server may host.
//
// Tables listen on --listen (127.0.0.1 by default, so only this machine can play) and the tickets tell players to
// connect to --public-host. To host for other machines, listen on 0.0.0.0, name this machine's address as the
// public host, and open the port range. Traffic is plaintext WebSocket: fine on a LAN, wants a TLS proxy beyond one.
//
// Each ticket reply opens the web client in the player's browser, in their seat. The bot serves the client, and card
// art beside it, on --web-port (7779): the built client in --client-root, by default this checkout's Release publish
// (tools\play-release-build.cmd makes it) or else its Debug build. --client names the same client hosted somewhere else - a
// static host or a CDN - and --art where its card art is, and then the bot serves neither.
//
// With a data directory - --data PATH, or CANTRIP_DATA; never AppData, since it is what gets backed up - the bot also
// hosts Winston drafts: /cantrip draft [--cube NAME|PATH (LSVCube)] [--draft-cards N (90)] [--draft-piles N (3)]
// [--min-deck N (40)] [--draft-ports FIRST-LAST (7790-7799)]. Every draft is kept there as an event, and a restarted
// bot hosts every live one again, wherever it had got to.
//
// --public-url https://cantrip.example.com (or CANTRIP_PUBLIC_URL) puts everything behind the one web port, for an
// HTTPS tunnel or proxy in front of it: the client and its art at that URL, and every table at
// wss://.../table/{port}/game, passed through to the table on this machine. The tables then listen only here, so the
// tunnel to the web port is the only way in. Without it, players connect to each table's own port at --public-host.
//
// With drafts and the bot's own site, /draft is the draft queue page: every queue, every table forming and every draft
// under way, open to anyone. /constructed is constructed play: the Standard gauntlet, 1v1 games each player brings one of the
// decks under decks/standard to. Players sign in on either with Discord to take a seat, given the application's OAuth2
// client id (--oauth-client-id, or CANTRIP_DISCORD_CLIENT_ID) and its secret, from the CANTRIP_DISCORD_SECRET environment
// variable only; {site}/signin/discord is to be registered as its redirect.
//
// --bot-narration DIR (or CANTRIP_BOT_NARRATION) has every bot seat write down each decision it makes and what it
// weighed - its picks and deck, then its games - a file a seat in DIR, so a draft or a game lost to a person can be read
// afterwards. Off by default.

BotSeats.NarrationFolder = StringArg(args, "--bot-narration") ?? Environment.GetEnvironmentVariable("CANTRIP_BOT_NARRATION");

string? token = Environment.GetEnvironmentVariable("CANTRIP_DISCORD_TOKEN");
if (string.IsNullOrWhiteSpace(token))
{
    Console.Error.WriteLine("Set CANTRIP_DISCORD_TOKEN to the bot's token.");
    return 1;
}

if (!ulong.TryParse(StringArg(args, "--guild"), out ulong guild))
{
    Console.Error.WriteLine("Pass --guild ID: the Discord server the bot serves.");
    return 1;
}

List<ulong> hostRoles = [];
foreach (string role in StringArgs(args, "--host-role"))
{
    if (!ulong.TryParse(role, out ulong id))
    {
        Console.Error.WriteLine($"--host-role takes a role ID, not '{role}'.");
        return 1;
    }

    hostRoles.Add(id);
}

string listen = StringArg(args, "--listen") ?? "127.0.0.1";
string publicHost = StringArg(args, "--public-host") ?? (listen is "127.0.0.1" or "0.0.0.0" ? "localhost" : listen);
if (!TryPorts(StringArg(args, "--ports") ?? "7780-7789", out List<int> ports))
{
    Console.Error.WriteLine("--ports takes a port or a range, as 7780-7789.");
    return 1;
}

if (!int.TryParse(StringArg(args, "--web-port") ?? "7779", out int webPort) || webPort is < 1 or > 65535 || ports.Contains(webPort))
{
    Console.Error.WriteLine("--web-port takes a port outside the table ports.");
    return 1;
}

Uri? publicUrl = null;
if ((StringArg(args, "--public-url") ?? Environment.GetEnvironmentVariable("CANTRIP_PUBLIC_URL")) is { Length: > 0 } publicArg
    && (!Uri.TryCreate(publicArg.EndsWith('/') ? publicArg : publicArg + "/", UriKind.Absolute, out publicUrl) || publicUrl.Scheme is not ("http" or "https")))
{
    Console.Error.WriteLine("--public-url takes an http(s) URL, as https://cantrip.example.com.");
    return 1;
}

// Behind the public URL, a table is reached through the web port rather than on its own.
Func<int, Uri>? tableAddress = publicUrl is null ? null : port => TableProxy.Address(publicUrl, port);
HashSet<int> tablePorts = [.. ports];

Uri? client = null;
if (StringArg(args, "--client") is { } clientArg
    && (!Uri.TryCreate(clientArg, UriKind.Absolute, out client) || client.Scheme is not ("http" or "https")))
{
    Console.Error.WriteLine("--client takes an http(s) URL.");
    return 1;
}

Uri? art = null;
if (StringArg(args, "--art") is { } artArg
    && (!Uri.TryCreate(artArg, UriKind.Absolute, out art) || art.Scheme is not ("http" or "https")))
{
    Console.Error.WriteLine("--art takes an http(s) URL.");
    return 1;
}

// Where card art is: as given, or beside the client when the bot serves it. Every table says so to the clients at it.
if (client is null)
{
    art ??= publicUrl ?? new Uri($"http://{publicHost}:{webPort}/");
}

// The site's own pages - its draft queues, its constructed play - which a table's players go back to once its game is
// over: only when the bot serves them, beside the client.
Uri? pages = client is null ? publicUrl ?? new Uri($"http://{publicHost}:{webPort}/") : null;
Cantrip.Server.Host.ReturnPage? draftPage = pages is null ? null : new Cantrip.Server.Host.ReturnPage(new Uri(pages, QueueSite.Route), "Draft Queues");

// A booster pod's own table goes back to the player's page, which lists the pod among their events: from the final
// standings, or once they have dropped.
Cantrip.Server.Host.ReturnPage? homePage = pages is null ? null : new Cantrip.Server.Host.ReturnPage(new Uri(pages, HomeSite.Route), "your page");

string? content = StringArg(args, "--content") ?? FindUp("content", "mtg");
string? decks = FindUp("decks");
if (content is null || decks is null)
{
    Console.Error.WriteLine("Could not find the 'content' and 'decks' directories above the executable. Pass --content PATH.");
    return 1;
}

DeckList deck;
try
{
    deck = DeckList.Load(DeckList.Find(decks, StringArg(args, "--deck") ?? "rw-aggro"));

    // Load the content and resolve the deck now, so a bad card fails here rather than on the first /cantrip host,
    // and the first game opens without the wait.
    Console.WriteLine("Loading content...");
    deck.Resolve([ContentCache.Get(content).Cards]);
}
catch (Exception error) when (error is IOException or InvalidOperationException or FormatException or KeyNotFoundException)
{
    Console.Error.WriteLine(error.Message);
    return 1;
}

// The Standard gauntlet's decks: every list under decks/standard whose cards the engine has.
List<DeckList> standard = [];
if (Directory.Exists(Path.Combine(decks, "standard")))
{
    int unplayable = 0;
    foreach (string path in Directory.GetFiles(Path.Combine(decks, "standard"), "*.txt").Order(StringComparer.Ordinal))
    {
        try
        {
            DeckList candidate = DeckList.Load(path);
            candidate.Resolve([ContentCache.Get(content).Cards]);
            standard.Add(candidate);
        }
        catch (Exception error) when (error is IOException or InvalidOperationException or FormatException)
        {
            unplayable++;
        }
    }

    Console.WriteLine($"{standard.Count} Standard deck(s) for the gauntlet{(unplayable == 0 ? string.Empty : $"; {unplayable} with cards the engine lacks left out")}.");
}

// Drafts, when there is somewhere to keep them.
DraftHost? drafts = null;
DraftQueues? queues = null;
PodHosts? pods = null;
string? cubeName = null;
PortraitStore? portraits = null;
EventStore? store = null;
if (EventStore.Configured(args) is { } data)
{
    if (!TryPorts(StringArg(args, "--draft-ports") ?? "7790-7799", out List<int> draftPorts) || draftPorts.Intersect(ports).Any() || draftPorts.Contains(webPort))
    {
        Console.Error.WriteLine("--draft-ports takes a port or a range, apart from the game ports and the web port.");
        return 1;
    }

    try
    {
        string? cubes = FindUp(Path.Combine("docs", "cube-lists"));
        string cube = CubeList.Find(StringArg(args, "--cube") ?? "LSVCube", [.. new[] { cubes, FindUp("cubes") }.OfType<string>()]);
        cubeName = Path.GetFileNameWithoutExtension(cube);
        // One store for the data directory, shared: it is what keeps two writers of the same file from meeting.
        store = new EventStore(data);

        // Players' Discord pictures, kept beside them and shown at their tables - from the bot's own site, so only when
        // it serves the client.
        portraits = new PortraitStore(data);
        Func<string, string?>? portraitPaths = client is null ? portraits.PathFor : null;
        drafts = new DraftHost(new DraftHostOptions
        {
            ContentRoot = content,
            Store = store,
            Cube = cubeName,
            CubeCards = CubeList.Load(cube),
            PickOrder = CubeList.PickOrder(cube),
            Cards = int.TryParse(StringArg(args, "--draft-cards"), out int cards) ? cards : 90,
            Piles = int.TryParse(StringArg(args, "--draft-piles"), out int piles) ? piles : 3,
            MinimumDeck = int.TryParse(StringArg(args, "--min-deck"), out int minimum) ? minimum : 40,
            Ports = draftPorts,
            ListenAddress = listen,
            PublicHost = publicHost,
            TableAddress = tableAddress,
            Portraits = portraitPaths,
            Art = art,
            Return = draftPage,
        });
        tablePorts.UnionWith(draftPorts);
        Console.WriteLine($"Drafts of {cubeName} kept in {Path.GetFullPath(data)}, on ports {draftPorts[0]}-{draftPorts[^1]}.");

        // Booster drafts' pods: the pod's own table and one for every game of a round, on the same draft ports.
        pods = new PodHosts(new PodHostsOptions
        {
            ContentRoot = content,
            Store = store,
            Ports = draftPorts,
            ListenAddress = listen,
            PublicHost = publicHost,
            TableAddress = tableAddress,
            Portraits = portraitPaths,
            Art = art,
            Return = homePage,
        });
        Console.WriteLine($"{await pods.ResumeAsync()} booster draft(s) taken up again.");

        // Draft queues, one to a channel, set up there with /cantrip queue and kept in the same directory.
        queues = new DraftQueues(new DraftQueuesOptions
        {
            ContentRoot = content,
            Store = store,
            Drafts = drafts,
            Pods = pods,
            Ports = draftPorts,
            CubeDirectories = [.. new[] { cubes, FindUp("cubes") }.OfType<string>()],
            ListenAddress = listen,
            PublicHost = publicHost,
            TableAddress = tableAddress,
            Portraits = portraitPaths,
            Art = art,
            Return = draftPage,
        });
        foreach (string problem in await queues.LoadAsync()) Console.Error.WriteLine(problem);
        Console.WriteLine($"{queues.Queues.Count} draft queue(s).");
    }
    catch (Exception error) when (error is IOException or ArgumentException or FormatException)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }
}
else
{
    Console.WriteLine("No data directory (--data PATH or CANTRIP_DATA), so no drafts: /cantrip host only.");
}

await using DraftHost? draftHost = drafts;
await using DraftQueues? draftQueues = queues;
await using PodHosts? podHosts = pods;

await using var matchmaker = new Matchmaker(new MatchmakerOptions
{
    ContentRoot = content,
    Deck = deck,
    Formats = standard.Count == 0 ? new Dictionary<string, IReadOnlyList<DeckList>>() : new Dictionary<string, IReadOnlyList<DeckList>> { ["standard"] = standard },
    Ports = ports,
    ListenAddress = listen,
    PublicHost = publicHost,
    TableAddress = tableAddress,
    Art = art,
    // A game hosted from Discord is a constructed game as much as one from the page.
    Return = pages is null || standard.Count == 0 ? null : new Cantrip.Server.Host.ReturnPage(new Uri(pages, PlaySite.Route), "Constructed Play"),
});

// The client is static, so one serves every game; only when it is hosted elsewhere does the bot not serve it.
Microsoft.AspNetCore.Builder.WebApplication? web = null;
TableAccess? access = null;
Uri? queuePage = null;

// The Discord side, for the queue page to ask who a signed-in player is on the server; there once the bot is made.
DiscordBot? discord = null;
if (client is null)
{
    string? root = StringArg(args, "--client-root") ?? Cantrip.Server.Host.WebClientFiles.Built(Path.GetDirectoryName(content)!);
    if (root is null || !File.Exists(Path.Combine(root, "index.html")))
    {
        Console.Error.WriteLine("No built web client to serve. Run tools\\play-release-build.cmd once, pass --client-root PATH, or --client URL.");
        return 1;
    }

    if (Cantrip.Server.Host.WebClientFiles.Stale(root, Path.GetDirectoryName(content)!) is { } stale)
    {
        Console.ForegroundColor = ConsoleColor.Yellow;
        Console.Error.WriteLine("WARNING: " + stale);
        Console.ResetColor();
    }

    Uri siteAddress = pages!;

    // Everything being played and everything played, at /status: the events kept in the data directory, the queues'
    // tables still forming and the games hosted on their own. The queue page shows the drafts under way from it too.
    // An event is being played only while the bot hosts it: one left live in the directory by a draft played locally,
    // or by one stopped for good, is not.
    var status = new EventStatusBoard(store, () => [.. queues?.StatusTables() ?? [], .. matchmaker.StatusTables()],
        () => [.. drafts?.Drafts.Where(draft => draft.IsLive).Select(draft => draft.Id) ?? [], .. pods?.Hosting ?? []]);
    var signIn = new SiteSignInOptions
    {
        Site = siteAddress,
        ClientId = StringArg(args, "--oauth-client-id") ?? Environment.GetEnvironmentVariable("CANTRIP_DISCORD_CLIENT_ID"),
        ClientSecret = Environment.GetEnvironmentVariable("CANTRIP_DISCORD_SECRET"),
        Keys = store is null ? null : Path.Combine(store.Root, "site-keys"),
        Player = account => discord?.WebPlayerAsync(account) ?? Task.FromResult(new DiscordPlayer(account.Id, account.Name)),
    };
    // Behind sign-in, every table is for the Discord accounts it has a seat for: the links go by the enter page and the
    // table proxy asks who is signed in. Without sign-in set up, or the proxy, a ticket opens a seat as it always has.
    access = signIn.Enabled && publicUrl is not null ? new TableAccess() : null;
    QueueSiteOptions? queueSite = null;
    if (queues is not null && drafts is not null && store is not null)
    {
        queueSite = new QueueSiteOptions
        {
            Queues = queues,
            Board = status,
            Drafts = drafts,
            Pods = pods,
            Client = siteAddress,
            Art = art,
            SignIn = signIn,
            Access = access,
            ChannelName = channel => discord?.ChannelName(channel),
        };
        queuePage = new Uri(siteAddress, QueueSite.Route);
    }

    PlaySiteOptions? playSite = standard.Count == 0 ? null : new PlaySiteOptions { Games = matchmaker, Client = siteAddress, Art = art, SignIn = signIn, Access = access };

    // The deck editor's catalog, and each player's saved decks, kept in the data directory.
    DeckSiteOptions? deckSite = store is null ? null : new DeckSiteOptions { ContentRoot = content, Store = store, SignIn = signIn };

    string artCache = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Cantrip", "art-cache");
    web = await ClientSite.StartAsync(listen, webPort, root, new Cantrip.Art.ArtStore(new Cantrip.Art.ArtStoreOptions { ContentRoot = content, CacheDirectory = artCache }),
        publicUrl is null ? null : tablePorts, portraits, status, signIn, queueSite, playSite, access,
        new HomeSiteOptions { SignIn = signIn, Drafts = queueSite, Constructed = playSite, HasDecks = deckSite is not null, Store = store }, deckSite);
    client = siteAddress;
    Console.WriteLine($"Serving the web client from {root}; the server's status at {new Uri(client, Cantrip.Server.Host.StatusSite.Route)}.");
    if (queueSite is not null) Console.WriteLine($"The draft queue page at {queuePage}.");
    if (playSite is not null) Console.WriteLine($"Constructed play at {new Uri(siteAddress, PlaySite.Route)}.");
    if (deckSite is not null) Console.WriteLine($"The deck editor at {new Uri(siteAddress, DeckSite.Route)}.");
    Console.WriteLine(signIn.Enabled
        ? $"Players sign in with Discord, which sends them back to {SiteSignIn.Callback(siteAddress)}: register it as the application's redirect."
        : "Nobody can sign in to take a seat from the site: give --oauth-client-id and CANTRIP_DISCORD_SECRET.");
}

await using Microsoft.AspNetCore.Builder.WebApplication? site = web;
await using var bot = new DiscordBot(
    new DiscordBotOptions { Token = token, Guild = guild, Deck = deck.Name, Client = client, Art = art, Cube = cubeName, Portraits = portraits, QueuePage = queuePage, Access = access },
    matchmaker,
    new HostPolicy(hostRoles),
    drafts,
    queues,
    pods);
discord = bot;

await bot.StartAsync();
if (drafts is not null)
{
    IReadOnlyList<HostedDraft> resumed = await drafts.ResumeAsync();
    if (resumed.Count > 0) Console.WriteLine($"Hosting {resumed.Count} draft(s) again from the data directory.");
}

Console.WriteLine(publicUrl is null
    ? $"Deck {deck.Name}; tables on {listen} ports {ports[0]}-{ports[^1]}, reached at {publicHost}; players open {client}. Ctrl+C to stop."
    : $"Deck {deck.Name}; everything behind {publicUrl} (web port {webPort}, tables passed through to {listen}); players open {client}. Ctrl+C to stop.");

using var stopping = new CancellationTokenSource();
Console.CancelKeyPress += (_, eventArgs) =>
{
    eventArgs.Cancel = true;
    stopping.Cancel();
};

try
{
    using var sweep = new PeriodicTimer(TimeSpan.FromSeconds(15));
    while (await sweep.WaitForNextTickAsync(stopping.Token))
    {
        try
        {
            await matchmaker.SweepAsync();
            if (drafts is not null) await drafts.SweepAsync();
            if (pods is not null) await pods.SweepAsync();
        }
        catch (Exception error) when (error is not OperationCanceledException)
        {
            // One table that would not close must not take the bot, and every other game, down with it.
            Console.Error.WriteLine($"A sweep failed: {error}");
        }
    }
}
catch (OperationCanceledException)
{
    Console.WriteLine("Stopping.");
}

return 0;

static bool TryPorts(string text, out List<int> ports)
{
    ports = [];
    string[] parts = text.Split('-', 2);
    if (!int.TryParse(parts[0], out int first) || first is < 1 or > 65535) return false;
    int last = first;
    if (parts.Length == 2 && (!int.TryParse(parts[1], out last) || last < first || last > 65535)) return false;
    ports = [.. Enumerable.Range(first, last - first + 1)];
    return true;
}

static string? StringArg(string[] args, string name)
{
    for (int i = 0; i < args.Length - 1; i++)
    {
        if (string.Equals(args[i], name, StringComparison.Ordinal))
        {
            return args[i + 1];
        }
    }

    return null;
}

// Every value given for an argument that may be repeated, in order.
static List<string> StringArgs(string[] args, string name)
{
    List<string> values = [];
    for (int i = 0; i < args.Length - 1; i++)
    {
        if (string.Equals(args[i], name, StringComparison.Ordinal))
        {
            values.Add(args[i + 1]);
        }
    }

    return values;
}

// The nearest directory at this relative path above the executable, optionally holding a subdirectory.
static string? FindUp(string relative, string? holding = null)
{
    var directory = new DirectoryInfo(AppContext.BaseDirectory);

    while (directory is not null)
    {
        string candidate = Path.Combine(directory.FullName, relative);
        if (Directory.Exists(holding is null ? candidate : Path.Combine(candidate, holding)))
        {
            return candidate;
        }

        directory = directory.Parent;
    }

    return null;
}
