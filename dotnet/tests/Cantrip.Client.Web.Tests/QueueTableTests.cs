using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A draft table formed from a queue, in the page, as a player who pressed Join in Discord sees it: the seats, Leave
/// Table, and once every seat is taken Ready! with how long is left. The other seats are clients of the test's own,
/// and the server's clock is the test's, moved on rather than waited for. Pictured under TestResults\web.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class QueueTableTests(WebClient web) : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-web-queue-" + Guid.NewGuid().ToString("N"));
    private readonly Clock _clock = new();

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private sealed class Clock : TimeProvider
    {
        private long _ticks = new DateTimeOffset(2026, 10, 1, 0, 0, 0, TimeSpan.Zero).Ticks;
        public override DateTimeOffset GetUtcNow() => new(Interlocked.Read(ref _ticks), TimeSpan.Zero);
        public void Advance(TimeSpan duration) => Interlocked.Add(ref _ticks, duration.Ticks);
    }

    private QueueTable Table() => new(new QueueTableSetup
    {
        Name = "Winston draft: rw-aggro",
        TimeProvider = _clock,
        Start = players =>
        {
            var store = new EventStore(_data);
            CreatedEvent created = DraftPod.CreateWinston(store, ContentCache.Get(WebClient.Content), new WinstonDraftSetup
            {
                Name = "Winston draft: rw-aggro",
                Cube = "rw-aggro",
                CubeCards = CubeList.Load(Path.Combine(WebClient.Root, "decks", "rw-aggro.txt")),
                Cards = 18,
                Players = players,
            }, _clock);
            return DraftPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), _clock);
        },
    });

    private Task<GameServer> Host(QueueTable table, Uri? art = null) => GameServer.StartAsync(new GameServerOptions
    {
        Port = 0, Players = 2, ContentRoot = WebClient.Content, Event = table, TimeProvider = _clock,

        // Everyone's picture but Larry's, who keeps the stand-in.
        Portraits = TestPortraits.AllBut("late"),
        Art = art,
    });

    private static Uri Address(GameServer server) => new($"ws://localhost:{server.Port}/game");

    private static async Task<GameClient> Sit(GameServer server, string ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(Address(server)));
        client.Join("ignored", 0, ticket);
        return client;
    }

    /// <summary>Keeps a client of the test's taking in what it is sent, for as long as the test runs.</summary>
    private static async Task Pump(GameClient client, CancellationToken stop)
    {
        while (!stop.IsCancellationRequested)
        {
            client.Pump();
            await Task.Delay(20, CancellationToken.None);
        }
    }

    [Fact]
    public async Task A_table_fills_the_one_who_is_not_ready_loses_the_seat_and_the_draft_begins_when_everyone_is()
    {
        QueueTable table = Table();
        await using GameServer server = await Host(table);
        string ticket = table.Reserve("you", "You")!;

        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(Address(server).AbsoluteUri)}#ticket={ticket}", WebClient.Pictures(nameof(QueueTableTests)));

        TableState state = await page.SettledAsync(state => state.Phase == "Forming");
        Assert.Contains(state.Controls, control => control.Label == "Leave Table");
        Assert.DoesNotContain(state.Controls, control => control.Label == "Ready!");
        await page.PictureAsync("waiting-for-a-player");

        // A second player is seated by the queue, and never presses Ready!.
        using var stop = new CancellationTokenSource();
        string larry = table.Reserve("late", "Late Larry")!;

        // Every seat taken, but Larry has not opened his table: no ready check yet, however long he takes.
        await page.SettledAsync(state => state.Phase == "Forming");
        await Task.Delay(500);
        state = (await page.StateAsync())!;
        Assert.DoesNotContain(state.Controls, control => control.Label == "Ready!");
        await page.PictureAsync("waiting-for-larry-to-open-the-table");

        GameClient late = await Sit(server, larry);
        Task lateRuns = Pump(late, stop.Token);
        state = await page.SettledAsync(state => state.Controls.Any(control => control.Label == "Ready!"));
        await page.PictureAsync("ready-check");

        await page.ClickAsync("Ready!", "button");
        await page.SettledAsync(state => !state.Controls.Any(control => control.Label == "Ready!"));
        await page.PictureAsync("ready-waiting-for-the-other");

        // Out of time: Larry is gone, and the table waits for the next player.
        _clock.Advance(TimeSpan.FromSeconds(31));
        await page.SettledAsync(state => state.Phase == "Forming" && !state.Controls.Any(control => control.Label == "Ready!"));
        DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(20);
        while (late.Released is null && DateTime.UtcNow < giveUp)
        {
            await Task.Delay(50);
        }

        Assert.NotNull(late.Released);
        await page.PictureAsync("seat-given-up");

        // The next player takes the seat and is ready at once (a bot); this one presses Ready! again, and the draft begins.
        GameClient next = await Sit(server, table.Reserve("next", "Next Nell")!);
        var bot = new AutoSeat(next);
        Task nextRuns = Task.Run(async () =>
        {
            while (!stop.IsCancellationRequested)
            {
                if (!bot.Step()) await Task.Delay(20, CancellationToken.None);
            }
        });

        await page.ClickAsync("Ready!", "button");
        await page.SettledAsync(state => state.Phase == "Drafting");
        await page.PictureAsync("draft-begun");

        await stop.CancelAsync();
        await Task.WhenAll(lateRuns, nextRuns);
        late.Disconnect();
        next.Disconnect();
        Assert.Empty(page.Errors);
    }

    [Fact]
    public async Task Leave_Table_gives_the_seat_up_and_says_so()
    {
        QueueTable table = Table();
        await using GameServer server = await Host(table);
        string ticket = table.Reserve("you", "You")!;

        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(Address(server).AbsoluteUri)}#ticket={ticket}", WebClient.Pictures(nameof(QueueTableTests)));

        await page.SettledAsync(state => state.Phase == "Forming");
        await page.ClickAsync("Leave Table", "button");
        await page.SettledAsync(state => state.Phase == "Released");
        await page.PictureAsync("left-the-table");

        Assert.Equal(0, table.Taken);
        Assert.Empty(page.Errors);
    }

    [Fact]
    public async Task A_seat_moved_to_another_table_takes_the_page_there()
    {
        QueueTable older = Table();
        QueueTable newer = Table();
        // Each table has art somewhere of its own, as a booster pod's game tables are on ports of their own.
        await using GameServer olderServer = await Host(older, new Uri("http://older.example/"));
        await using GameServer newerServer = await Host(newer, new Uri("http://newer.example/"));
        string fran = older.Reserve("first", "First Fran")!;
        string ticket = newer.Reserve("you", "You")!;

        // Fran has the older table open: the ready check starts as soon as this player arrives there.
        GameClient franClient = await Sit(olderServer, fran);
        using var stop = new CancellationTokenSource();
        Task franRuns = Pump(franClient, stop.Token);

        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(Address(newerServer).AbsoluteUri)}#ticket={ticket}", WebClient.Pictures(nameof(QueueTableTests)));
        TableState state = await page.SettledAsync(state => state.Phase == "Forming");
        Assert.Equal("http://newer.example/art/{size}/{slug}.webp", state.Art);

        // The queue moves this player to the older table, which then has both its players.
        string moved = older.Reserve("you", "You")!;
        newer.Release("you", new TableMoveMessage { Server = Address(olderServer).AbsoluteUri, Ticket = moved });

        state = await page.SettledAsync(state => state.Phase == "Forming" && state.Controls.Any(control => control.Label == "Ready!"));
        await page.PictureAsync("moved-to-the-older-table");

        // Card art is looked for where the table the page is at now says, not where the page came from or what its
        // address would suggest.
        Assert.Equal("http://older.example/art/{size}/{slug}.webp", state.Art);
        Assert.Equal(2, older.Taken);
        Assert.Equal(0, newer.Taken);
        await stop.CancelAsync();
        await franRuns;
        franClient.Disconnect();
    }
}
