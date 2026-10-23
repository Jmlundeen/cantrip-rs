using System.Text.Json;
using Cantrip.Art;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Microsoft.AspNetCore.Builder;

// A player-hosted server. Players choose names/decks/sets and explicitly ready over a loopback WebSocket.
//
// Usage: cantrip-server --format KEY [--min-deck N] [--port N] [--players N] [--seed N] [--content PATH] [--decks PATH]
//                       [--deck NAME]... [--auto-start] [--art-port N] [--art-cache PATH] [--art-log] [--clock MINUTES]
//
// --format names the format the table's decks are played in, which the server checks every deck against before it is
// chosen and dealt and tells the engine the minimum deck size of: freeform (the default: no deck-construction restriction),
// standard, pioneer, modern, legacy, vintage, commander, penny, or limited (with --min-deck, forty by default).
//
// Every table has chess clocks (docs/events.md, "Clocks"): each player's runs while the game waits on them, and whoever's
// runs out loses (--clock, an hour by default).
//
// Every game is dealt through the native Rust engine, in the cantrip-native-host sidecar (CANTRIP_NATIVE_HOST names it).
//
// Card art is served beside the table at http://localhost:{art-port}/art/{size}/{slug}.webp, --art-port defaulting to
// the table's port plus one; 0 serves none. See Cantrip.Art.ArtServer.
//
// --decks is the allowlisted catalog directory (decks/demo by default). Repeated --deck options choose a smaller
// catalog by decklist path/name. They never assign decks positionally to seats unless --auto-start is used by a
// local test tool, in which case seats rotate through the catalog in command-line order.
//
// An event instead of a lobby - a Winston draft, its deck building and its game - is kept in a data directory, which
// is the server's to back up: --data PATH, or the CANTRIP_DATA environment variable. Never under AppData.
//
//   cantrip-server --data PATH --winston --cube NAME|PATH [--piles 3] [--cards 90] [--min-deck 40]
//                  --player ID[=NAME] --player ID[=NAME] [--event-name TEXT] [--tickets FILE]
//                                        makes a new Winston draft and hosts it
//   cantrip-server --data PATH --grid --cube NAME|PATH [--packs 18] [--min-deck 40]
//                  --player ID[=NAME] --player ID[=NAME] [--event-name TEXT] [--tickets FILE]
//                                        makes a new grid draft for two and hosts it: each pack a grid of nine
//   cantrip-server --data PATH --booster --cube NAME|PATH [--packs 3] [--pack-size 15] [--rounds 3] [--min-deck 40]
//                  --player ID[=NAME]... (two to eight) [--event-name TEXT] [--tickets FILE]
//                                        makes a booster draft of a pod and hosts it: the pod's table on --port, and
//                                        a table for each game of its Swiss rounds on a port of its own
//   cantrip-server --data PATH --event ID [--tickets FILE]
//                                        hosts an event already made, wherever it had got to
//
// A cube is a list of card names (docs/cube-lists/LSVCube.txt, or LSVCube to find it there); --cards draws that many
// at random from it, all of them by default. Each player gets a ticket, which seats them by --ticket in the bot or
// #ticket= in the web client, from any computer; a fresh one is made for each player whenever the event is hosted.
// --tickets writes them to a file as JSON, for a tool to hand on; otherwise they are printed.

int port = IntArg(args, "--port", 7777);
int players = IntArg(args, "--players", 2);
ulong seed = UlongArg(args, "--seed", 1);
bool autoStart = args.Contains("--auto-start", StringComparer.Ordinal);
string? content = StringArg(args, "--content") ?? FindContentRoot();

if (content is null)
{
    Console.Error.WriteLine(
        "Could not find a 'content' directory. Pass --content PATH pointing at the folder that holds the datasets.");

    return 1;
}

if (args.Contains("--winston", StringComparer.Ordinal) || args.Contains("--booster", StringComparer.Ordinal) || args.Contains("--grid", StringComparer.Ordinal)
    || StringArg(args, "--event") is not null)
{
    return await HostEventAsync(args, content);
}

string? decksRoot = FindUp("decks", "demo");
string? decks = StringArg(args, "--decks") ?? (decksRoot is null ? null : Path.Combine(decksRoot, "demo"));

if (decks is null)
{
    Console.Error.WriteLine("Could not find a 'decks/demo' directory. Pass --decks PATH pointing at a folder of decklists.");
    return 1;
}

if (players is < 2 or > 4)
{
    Console.Error.WriteLine("--players must be between 2 and 4.");
    return 1;
}

List<string> named = StringArgs(args, "--deck");

List<DeckList> catalog = [];

try
{
    catalog = named.Count == 0 ? [.. DeckList.LoadDirectory(decks)]
        : [.. named
            .Select(name => Path.GetFullPath(DeckList.Find(decksRoot ?? Directory.GetCurrentDirectory(), name)))
            .Distinct(StringComparer.OrdinalIgnoreCase)
            .Select(DeckList.Load)];
}
catch (Exception error) when (error is IOException or InvalidOperationException or FormatException)
{
    Console.Error.WriteLine(error.Message);
    return 1;
}

string? formatKey = StringArg(args, "--format");
Cantrip.Decks.DeckFormat? format = formatKey is null ? Cantrip.Decks.DeckFormat.Freeform
    : formatKey == Cantrip.Decks.DeckFormat.LimitedKey ? Cantrip.Decks.DeckFormat.Limited(IntArg(args, "--min-deck", 40))
    : Cantrip.Decks.DeckFormat.Find(formatKey);
if (format is null)
{
    Console.Error.WriteLine($"There is no format '{formatKey}': "
        + string.Join(", ", Cantrip.Decks.DeckFormat.All.Select(each => each.Key)) + ", or limited [--min-deck N].");
    return 1;
}

foreach (DeckList deck in catalog)
{
    Console.WriteLine($"Available deck: {deck.Name} ({deck.Count} cards).");
}

int artPort = IntArg(args, "--art-port", port + 1);
string artCache = StringArg(args, "--art-cache")
    ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Cantrip", "art-cache");

await using WebApplication? art = artPort == 0
    ? null
    : await ArtServer.StartAsync(new ArtStore(new ArtStoreOptions { ContentRoot = content, CacheDirectory = artCache }), "127.0.0.1", artPort, args.Contains("--art-log", StringComparer.Ordinal));

if (art is not null)
{
    Console.WriteLine($"Card art at: {art.Urls.First()}/art/{{size}}/{{slug}}.webp (cached in {artCache})");
}

await using GameServer server = await GameServer.StartAsync(new GameServerOptions
{
    Format = format,
    Port = port,
    Players = players,
    Seed = seed,
    ContentRoot = content,
    Decks = catalog,
    AutoStart = autoStart,
    Art = ArtAddress(art),
    PlayerClock = TimeSpan.FromMinutes(Math.Max(1, IntArg(args, "--clock", (int)MatchClocks.DefaultPlayerClock.TotalMinutes))),
});

Console.WriteLine($"Join with: ws://localhost:{server.Port}/game");
Console.WriteLine("Ctrl+C to stop.");

using var stopping = new CancellationTokenSource();
Console.CancelKeyPress += (_, eventArgs) =>
{
    eventArgs.Cancel = true;
    stopping.Cancel();
};

try
{
    await server.Finished.WaitAsync(stopping.Token);
    Console.WriteLine("The game has finished.");
}
catch (OperationCanceledException)
{
    Console.WriteLine("Stopping.");
}

return 0;

// Makes or reopens an event and hosts its table until its game is over.
static async Task<int> HostEventAsync(string[] args, string content)
{
    if (EventStore.Configured(args) is not { } data)
    {
        Console.Error.WriteLine("An event is kept in a data directory: pass --data PATH, or set CANTRIP_DATA.");
        return 1;
    }

    var store = new EventStore(data);
    Cantrip.Datasets.ContentLibrary library = ContentCache.Get(content);
    string eventId;
    try
    {
        if (StringArg(args, "--event") is { } existing)
        {
            eventId = existing;
        }
        else
        {
            string? cubeName = StringArg(args, "--cube");
            List<(string Id, string Name)> players = [.. StringArgs(args, "--player").Select(player =>
            {
                int equals = player.IndexOf('=', StringComparison.Ordinal);
                string name = equals < 0 ? player : player[(equals + 1)..];
                return (EventStore.ToId(equals < 0 ? player : player[..equals]), name);
            })];
            if (cubeName is null || players.Count < 2)
            {
                Console.Error.WriteLine("A draft needs --cube NAME|PATH and a --player ID[=NAME] for each player.");
                return 1;
            }

            string cube = CubeList.Find(cubeName, [.. new[] { FindUp(Path.Combine("docs", "cube-lists")), FindUp("cubes") }.OfType<string>()]);
            CreatedEvent created = args.Contains("--booster", StringComparer.Ordinal)
                ? BoosterPod.Create(store, library, new BoosterDraftSetup
                {
                    Name = StringArg(args, "--event-name") ?? $"Booster draft: {Path.GetFileNameWithoutExtension(cube)}",
                    Cube = Path.GetFileNameWithoutExtension(cube),
                    CubeCards = CubeList.Load(cube),
                    PickOrder = CubeList.PickOrder(cube),
                    Packs = IntArg(args, "--packs", 3),
                    PackSize = IntArg(args, "--pack-size", 15),
                    Rounds = IntArg(args, "--rounds", 3),
                    MinimumDeck = IntArg(args, "--min-deck", 40),
                    Seed = UlongArg(args, "--seed", 0),
                    Players = players,
                })
                : args.Contains("--grid", StringComparer.Ordinal)
                ? DraftPod.CreateGrid(store, library, new GridDraftSetup
                {
                    Name = StringArg(args, "--event-name") ?? $"Grid draft: {Path.GetFileNameWithoutExtension(cube)}",
                    Cube = Path.GetFileNameWithoutExtension(cube),
                    CubeCards = CubeList.Load(cube),
                    PickOrder = CubeList.PickOrder(cube),
                    Packs = IntArg(args, "--packs", 18),
                    MinimumDeck = IntArg(args, "--min-deck", 40),
                    Seed = UlongArg(args, "--seed", 0),
                    Players = players,
                })
                : DraftPod.CreateWinston(store, library, new WinstonDraftSetup
            {
                Name = StringArg(args, "--event-name") ?? $"Winston draft: {Path.GetFileNameWithoutExtension(cube)}",
                Cube = Path.GetFileNameWithoutExtension(cube),
                CubeCards = CubeList.Load(cube),
                PickOrder = CubeList.PickOrder(cube),
                Piles = IntArg(args, "--piles", 3),
                Cards = IntArg(args, "--cards", 0),
                MinimumDeck = IntArg(args, "--min-deck", 40),
                Seed = UlongArg(args, "--seed", 0),
                Players = players,
            });
            eventId = created.Record.Id;
            Console.WriteLine($"Made event {eventId}: {created.Record.Draft!.Cards} cards from {cube}, in {store.Root}.");
            if (created.Missing.Count > 0)
            {
                Console.WriteLine($"{created.Missing.Count} of the cube's cards are not loaded and were left out.");
            }
        }
    }
    catch (Exception error) when (error is IOException or ArgumentException or InvalidOperationException or FormatException)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }

    if (store.LoadEvent(eventId) is { Kind: EventKinds.BoosterDraft })
    {
        return await HostBoosterAsync(args, content, store, library, eventId);
    }

    DraftPod pod;
    try
    {
        pod = DraftPod.Open(store, eventId, library);
    }
    catch (Exception error) when (error is IOException or ArgumentException or InvalidOperationException)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }

    if (pod.Stage is EventStage.Complete or EventStage.Cancelled)
    {
        Console.Error.WriteLine($"Event {eventId} is over.");
        return 1;
    }

    // Fresh tickets each time the event is hosted, since a ticket is kept only as its hash.
    Dictionary<string, SeatTicket> tickets = [];
    foreach ((string playerId, string name, int seat) in pod.Players)
    {
        tickets[playerId] = new SeatTicket(seat, name, pod.IssueTicket(playerId));
    }

    var output = new { eventId, tickets };
    if (StringArg(args, "--tickets") is { } ticketFile)
    {
        // Moved into place whole, so a tool waiting for the file never reads half of it.
        File.WriteAllText(ticketFile + ".tmp", JsonSerializer.Serialize(output, new JsonSerializerOptions { WriteIndented = true, PropertyNamingPolicy = JsonNamingPolicy.CamelCase }));
        File.Move(ticketFile + ".tmp", ticketFile, overwrite: true);
        Console.WriteLine($"Tickets written to {ticketFile}.");
    }
    else
    {
        foreach ((string playerId, SeatTicket seat) in tickets)
        {
            Console.WriteLine($"Seat {seat.Seat}, {seat.Name} ({playerId}): ticket {seat.Ticket}");
        }
    }

    int port = IntArg(args, "--port", 7777);
    int artPort = IntArg(args, "--art-port", port + 1);
    string artCache = StringArg(args, "--art-cache")
        ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Cantrip", "art-cache");
    await using WebApplication? art = artPort == 0
        ? null
        : await ArtServer.StartAsync(new ArtStore(new ArtStoreOptions { ContentRoot = content, CacheDirectory = artCache }), "127.0.0.1", artPort, args.Contains("--art-log", StringComparer.Ordinal));

    await using GameServer server = await GameServer.StartAsync(new GameServerOptions
    {
        Format = Cantrip.Decks.DeckFormat.Limited(),
        Port = port,
        Players = pod.Seats,
        ContentRoot = content,
        Event = pod,
        Art = ArtAddress(art),
        });

    Console.WriteLine($"Event {eventId} ({pod.Name}) at ws://localhost:{server.Port}/game. Ctrl+C to stop; it carries on from here when hosted again.");
    using var stopping = new CancellationTokenSource();
    Console.CancelKeyPress += (_, eventArgs) =>
    {
        eventArgs.Cancel = true;
        stopping.Cancel();
    };

    try
    {
        await server.Finished.WaitAsync(stopping.Token);
        Console.WriteLine($"The event's game has finished. {store.LoadEvent(eventId)?.Detail}");
    }
    catch (OperationCanceledException)
    {
        Console.WriteLine("Stopping.");
    }

    return 0;
}

// Hosts a booster draft: the pod's own table at --port, where its players draft, build and see the pairings, and a
// table for every game of its rounds on a port of its own, until the server is stopped.
static async Task<int> HostBoosterAsync(string[] args, string content, EventStore store, Cantrip.Datasets.ContentLibrary library, string eventId)
{
    BoosterPod pod;
    try
    {
        pod = BoosterPod.Open(store, eventId, library);
    }
    catch (Exception error) when (error is IOException or ArgumentException or InvalidOperationException)
    {
        Console.Error.WriteLine(error.Message);
        return 1;
    }

    if (pod.Stage is EventStage.Cancelled)
    {
        Console.Error.WriteLine($"Event {eventId} was called off.");
        return 1;
    }

    Dictionary<string, SeatTicket> tickets = [];
    foreach ((string playerId, string name, int seat) in pod.Players)
    {
        tickets[playerId] = new SeatTicket(seat, name, pod.IssueTicket(playerId));
    }

    var output = new { eventId, tickets };
    if (StringArg(args, "--tickets") is { } ticketFile)
    {
        File.WriteAllText(ticketFile + ".tmp", JsonSerializer.Serialize(output, new JsonSerializerOptions { WriteIndented = true, PropertyNamingPolicy = JsonNamingPolicy.CamelCase }));
        File.Move(ticketFile + ".tmp", ticketFile, overwrite: true);
        Console.WriteLine($"Tickets written to {ticketFile}.");
    }
    else
    {
        foreach ((string playerId, SeatTicket seat) in tickets)
        {
            Console.WriteLine($"Seat {seat.Seat}, {seat.Name} ({playerId}): ticket {seat.Ticket}");
        }
    }

    int port = IntArg(args, "--port", 7777);
    int artPort = IntArg(args, "--art-port", port + 1);
    string artCache = StringArg(args, "--art-cache")
        ?? Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Cantrip", "art-cache");
    await using WebApplication? art = artPort == 0
        ? null
        : await ArtServer.StartAsync(new ArtStore(new ArtStoreOptions { ContentRoot = content, CacheDirectory = artCache }), "127.0.0.1", artPort, args.Contains("--art-log", StringComparer.Ordinal));

    // The pod's own table on the port asked for; its games' tables wherever the system has a port free.
    TableOpener anyPort = PodHost.Loopback(content, art: ArtAddress(art));
    bool podOpened = false;
    await using PodHost host = await PodHost.StartAsync(pod, async (table, seats) =>
    {
        if (podOpened) return await anyPort(table, seats);
        podOpened = true;
        GameServer server = await GameServer.StartAsync(new GameServerOptions { Format = Cantrip.Decks.DeckFormat.Limited(), Port = port, Players = seats, ContentRoot = content, Event = table, Art = ArtAddress(art) });
        return (server, new Uri($"ws://localhost:{server.Port}/game"));
    });

    Console.WriteLine($"Event {eventId} ({pod.Name}) at {host.Address}. Ctrl+C to stop; it carries on from here when hosted again.");
    using var stopping = new CancellationTokenSource();
    Console.CancelKeyPress += (_, eventArgs) =>
    {
        eventArgs.Cancel = true;
        stopping.Cancel();
    };

    EventStage last = EventStage.NotActive;
    try
    {
        while (!stopping.IsCancellationRequested)
        {
            if (pod.Stage != last)
            {
                last = pod.Stage;
                Console.WriteLine($"{last}: {pod.Detail}");
            }

            await Task.Delay(500, stopping.Token);
        }
    }
    catch (OperationCanceledException)
    {
        Console.WriteLine("Stopping.");
    }

    return 0;
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

// Where card art is for the tables' clients: the art server started beside them, or null when none was.
static Uri? ArtAddress(WebApplication? art) => art is null ? null : new Uri(art.Urls.First().TrimEnd('/') + "/");

static int IntArg(string[] args, string name, int fallback) =>
    int.TryParse(StringArg(args, name), out int value) ? value : fallback;



static ulong UlongArg(string[] args, string name, ulong fallback) =>
    ulong.TryParse(StringArg(args, name), out ulong value) ? value : fallback;

// Walks up from the executable, so running from the repository works without passing a path. An installed server
// would be given --content explicitly.
static string? FindContentRoot() => FindUp("content", "mtg");

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

/// <summary>A player's seat in an event and the ticket that takes them to it.</summary>
internal sealed record SeatTicket(int Seat, string Name, string Ticket);
