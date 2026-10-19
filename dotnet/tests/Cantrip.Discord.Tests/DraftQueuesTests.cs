using System.Net;
using System.Net.Sockets;
using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The draft queues behind a Discord queue channel, with real tables on real sockets and a clock of the test's own:
/// joining fills the oldest table, a ready check gives up the seats of whoever was not ready, two tables each short of a
/// player are made one by moving the newer table's player, and a draft that begins is handed to the draft host.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class DraftQueuesTests : IDisposable
{
    private const ulong Channel = 4242;
    private static readonly DiscordPlayer Alice = new(1001, "Alice");
    private static readonly DiscordPlayer Bob = new(1002, "Bob");
    private static readonly DiscordPlayer Carol = new(1003, "Carol");

    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-queues-" + Guid.NewGuid().ToString("N"));
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

    private (DraftQueues Queues, DraftHost Drafts) Create(ReturnPage? back = null) => Create(out _, back);

    private (DraftQueues Queues, DraftHost Drafts) Create(out PodHosts pods, ReturnPage? back = null)
    {
        List<int> ports = FreePorts(6);
        var store = new EventStore(_data);
        var drafts = new DraftHost(new DraftHostOptions
        {
            ContentRoot = RepoRoot.ContentRoot, Store = store, Cube = "rw-aggro", CubeCards = [], Ports = ports, TimeProvider = _clock,
        });
        pods = new PodHosts(new PodHostsOptions { ContentRoot = RepoRoot.ContentRoot, Store = store, Ports = ports, TimeProvider = _clock });
        var queues = new DraftQueues(new DraftQueuesOptions
        {
            ContentRoot = RepoRoot.ContentRoot, Store = store, Drafts = drafts, Pods = pods, Ports = ports, TimeProvider = _clock,
            CubeDirectories = [Path.Combine(RepoRoot.Path, "decks")], Return = back,
        });
        return (queues, drafts);
    }

    [Fact]
    public async Task A_queues_table_names_the_draft_queue_page_to_go_back_to_once_its_game_is_over()
    {
        var page = new ReturnPage(new Uri("https://cantrip.example/draft"), "Draft Queues");
        (DraftQueues queues, DraftHost drafts) = Create(page);
        await using (drafts)
        await using (queues)
        {
            Assert.Null(await queues.ConfigureAsync(Settings()));
            QueueSeating seating = await queues.JoinAsync(Channel, new DiscordPlayer(2001, "Player 1"));
            GameClient client = await Sit(seating.Address!, seating.Ticket!);
            await Until(() => client.Table is not null, client);

            Assert.Equal(("https://cantrip.example/draft", "Draft Queues"), (client.Table!.Return, client.Table.ReturnName));
            client.Disconnect();
        }
    }

    [Fact]
    public async Task A_booster_queue_seats_eight_at_one_table_and_its_pod_begins_when_all_are_ready()
    {
        (DraftQueues queues, DraftHost drafts) = Create(out PodHosts pods);
        await using (pods)
        await using (drafts)
        await using (queues)
        {
            // The RW Aggro list is forty cards: one pack of five each for eight.
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "booster", Packs = 3, PackSize = 15 }));
            Assert.Null(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "booster", Packs = 1, PackSize = 5, Rounds = 1, MinimumDeck = 20 }));

            List<QueueSeating> seatings = [];
            for (ulong i = 0; i < 8; i++)
            {
                seatings.Add(await queues.JoinAsync(Channel, new DiscordPlayer(2000 + i, $"Player {i + 1}")));
            }

            Assert.Equal(Enumerable.Range(1, 8), seatings.Select(seating => seating.Seat));
            Assert.Single(seatings.Select(seating => seating.Address).Distinct());
            await Settle(() => queues.Status(Channel) is { Waiting: 8, Seats: 8 });

            List<GameClient> clients = [];
            foreach (QueueSeating seating in seatings)
            {
                clients.Add(await Sit(seating.Address!, seating.Ticket!));
            }

            GameClient[] all = [.. clients];
            await Until(() => clients.All(client => client.Table?.ReadyLeftMs > 0), all);
            foreach (GameClient client in clients) client.SetReady(true);

            // Everyone ready: the pod's draft begins at the same table, and the pod host has it.
            await Until(() => clients.All(client => client.Draft is { Format: "booster", Pile.Count: 5 }), all);
            await Settle(() => pods.Underway(Channel) == 1 && queues.Status(Channel) is { Underway: 1, Waiting: 0 });
            Assert.True(pods.InDraft(2000));
            Assert.True((await queues.JoinAsync(Channel, new DiscordPlayer(2000, "Player 1"))).Refusal is not null);

            PodSeating back = await pods.ReturnAsync(new DiscordPlayer(2003, "Player 4"));
            Assert.Equal((seatings[0].Address, 4), (back.Address, back.Seat));
            foreach (GameClient client in clients) client.Disconnect();
        }
    }

    [Fact]
    public async Task A_table_of_a_players_own_fills_only_from_the_page_and_its_draft_begins_like_a_queues()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            await queues.ConfigureAsync(Settings());
            Assert.Contains("rw-aggro", queues.Cubes());
            Assert.Contains("no cube", (await queues.CreateTableAsync(new QueueSettings { Cube = "nope", Cards = 18 }, Alice)).Refusal, StringComparison.Ordinal);

            // A grid draft of her own, though the channel's queue drafts Winston: in no channel, and she has its first seat.
            QueueSeating alice = await queues.CreateTableAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "grid", Packs = 4, MinimumDeck = 10 }, Alice);
            Assert.Equal(1, alice.Seat);
            QueueBoardTable own = Assert.Single(queues.Board().Tables);
            Assert.Equal((0UL, "grid", "rw-aggro", 2), (own.Channel, own.Settings.Format, own.Cube, own.Seats));
            Assert.Equal(["Alice"], own.Players.Select(player => player.Name));
            Assert.Equal(own.Id, queues.TableOf(Alice.Id));

            // The channel's Join opens a table of its own rather than seat anyone at hers.
            QueueSeating bob = await queues.JoinAsync(Channel, Bob);
            Assert.NotEqual(alice.Address, bob.Address);
            Assert.Equal(2, queues.Board().Tables.Count);

            // One seat at a time, wherever it was taken.
            Assert.NotNull((await queues.CreateTableAsync(new QueueSettings { Cube = "rw-aggro", Cards = 18 }, Alice)).Refusal);
            Assert.NotNull((await queues.JoinTableAsync(own.Id, Bob)).Refusal);
            Assert.Contains("on the draft queue page", (await queues.JoinAsync(Channel, Alice)).Refusal, StringComparison.Ordinal);

            // Picked on the page: Carol sits down across from her, and then it is full.
            QueueSeating carol = await queues.JoinTableAsync(own.Id, Carol);
            Assert.Equal((alice.Address, 2), (carol.Address, carol.Seat));
            Assert.Equal("That table is full.", (await queues.JoinTableAsync(own.Id, new DiscordPlayer(1004, "Dave"))).Refusal);

            GameClient aliceClient = await Sit(alice.Address!, alice.Ticket!);
            GameClient carolClient = await Sit(carol.Address!, carol.Ticket!);
            await Until(() => aliceClient.Table?.ReadyLeftMs > 0, aliceClient, carolClient);
            aliceClient.SetReady(true);
            carolClient.SetReady(true);

            // Begun at the same table and handed to the draft host as a draft from no queue; the page no longer lists it.
            await Until(() => aliceClient.Draft is { Format: "grid" }, aliceClient, carolClient);
            await Settle(() => drafts.Drafts.Count == 1 && queues.Board().Tables.Count == 1);
            HostedDraft draft = Assert.Single(drafts.Drafts);
            Assert.Equal((Alice, Carol, 0UL), (draft.Host, draft.Guest, draft.Queue));
            Assert.Null(queues.TableOf(Alice.Id));
            Assert.Contains("Back to my draft", (await queues.JoinTableAsync(queues.Board().Tables[0].Id, Alice)).Refusal, StringComparison.Ordinal);
            aliceClient.Disconnect();
            carolClient.Disconnect();
        }
    }

    private static QueueSettings Settings() => new() { Channel = Channel, Cube = "rw-aggro", Cards = 18, MinimumDeck = 20 };

    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }

    private static async Task<GameClient> Sit(Uri address, string ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(address));
        client.Join("ignored", ticket: ticket);
        await Until(() => client.Membership is not null || client.Errors.Count > 0, client);
        return client;
    }

    private static async Task Until(Func<bool> condition, params GameClient[] clients)
    {
        DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(30);
        while (!condition())
        {
            foreach (GameClient client in clients) client.Pump();
            Assert.True(DateTime.UtcNow < giveUp, "Gave up waiting. " + string.Join(" | ", clients.Select(client =>
                $"seat {client.Membership?.Seat}, phase {client.Table?.Phase}, ready left {client.Table?.ReadyLeftMs}, "
                + $"players [{string.Join(", ", client.Table?.Players.Select(player => player.Name) ?? [])}], released {client.Released?.Detail}, moved {client.Moved?.Server}")));
            await Task.Delay(5);
        }
    }

    [Fact]
    public async Task A_queue_is_kept_and_a_cube_that_cannot_be_found_or_is_too_small_is_refused()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "no-such-cube" }));
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Cards = 5000 }));
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "rochester" }));
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt") }));
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "../decks/rw-aggro" }));
            Assert.Null(await queues.ConfigureAsync(Settings()));
            await queues.NoteMessageAsync(Channel, 99);
        }

        (DraftQueues again, DraftHost more) = Create();
        await using (more)
        await using (again)
        {
            Assert.Empty(await again.LoadAsync());
            QueueSettings kept = Assert.Single(again.Queues);
            Assert.Equal(18, kept.Cards);
            Assert.Equal(99UL, kept.Message);
            Assert.Equal(new QueueStatus(kept, 0, 2, 0, 0), again.Status(Channel));
        }
    }

    [Fact]
    public async Task Players_fill_a_table_a_ready_check_gives_up_the_unready_seat_and_the_next_player_takes_it()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            await queues.ConfigureAsync(Settings());
            QueueSeating alice = await queues.JoinAsync(Channel, Alice);
            Assert.Equal(1, alice.Seat);
            await Settle(() => queues.Status(Channel)!.Waiting == 1);

            // Joining again is a fresh ticket to the same seat.
            QueueSeating again = await queues.JoinAsync(Channel, Alice);
            Assert.Equal((alice.Address, 1), (again.Address, again.Seat));
            Assert.NotEqual(alice.Ticket, again.Ticket);

            QueueSeating bob = await queues.JoinAsync(Channel, Bob);
            Assert.Equal((alice.Address, 2), (bob.Address, bob.Seat));

            // Both seats taken, but the ready check waits for both to have the table open.
            await Settle(() => queues.Status(Channel) is { Waiting: 2, Checking: 0 });
            GameClient aliceClient = await Sit(alice.Address!, alice.Ticket!);
            GameClient bobClient = await Sit(bob.Address!, bob.Ticket!);
            await Until(() => aliceClient.Table?.ReadyLeftMs > 0, aliceClient, bobClient);
            await Settle(() => queues.Status(Channel) is { Waiting: 0, Checking: 1 });
            aliceClient.SetReady(true);
            await Until(() => bobClient.Table!.Players[0].Ready, aliceClient, bobClient);

            _clock.Advance(TimeSpan.FromSeconds(31));
            await Until(() => bobClient.Released is not null, aliceClient, bobClient);
            await Settle(() => queues.Status(Channel) is { Waiting: 1, Checking: 0 });

            QueueSeating carol = await queues.JoinAsync(Channel, Carol);
            Assert.Equal((alice.Address, 2), (carol.Address, carol.Seat));
            aliceClient.Disconnect();
            bobClient.Disconnect();
        }
    }

    [Fact]
    public async Task Two_tables_each_short_a_player_become_one_and_the_moved_player_is_told_where_to_go()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            await queues.ConfigureAsync(Settings());
            QueueSeating alice = await queues.JoinAsync(Channel, Alice);
            QueueSeating bob = await queues.JoinAsync(Channel, Bob);
            QueueSeating carol = await queues.JoinAsync(Channel, Carol);
            Assert.NotEqual(alice.Address, carol.Address);

            GameClient aliceClient = await Sit(alice.Address!, alice.Ticket!);
            GameClient carolClient = await Sit(carol.Address!, carol.Ticket!);

            // Bob never opens his table, so after thirty seconds the first table loses him: it and Carol's both wait for one
            // more, until Carol is moved.
            await Until(() => aliceClient.Table?.Players.Count == 2 && carolClient.Table?.Players.Count == 2, aliceClient, carolClient);
            Assert.Equal(0, aliceClient.Table!.ReadyLeftMs);
            _clock.Advance(TimeSpan.FromSeconds(31));
            await Until(() => carolClient.Moved is not null, aliceClient, carolClient);
            Assert.Equal(alice.Address!.AbsoluteUri, carolClient.Moved!.Server);

            GameClient moved = await Sit(new Uri(carolClient.Moved.Server), carolClient.Moved.Ticket);
            Assert.Equal(2, moved.Membership!.Seat);
            await Until(() => aliceClient.Table!.Players.Select(player => player.Name).SequenceEqual(["Alice", "Carol"]) && aliceClient.Table.ReadyLeftMs > 0, aliceClient, moved);
            await Settle(() => queues.Status(Channel) is { Waiting: 0, Checking: 1 });

            // Both ready: the draft begins at that table, and the draft host has it.
            aliceClient.SetReady(true);
            moved.SetReady(true);
            await Until(() => aliceClient.Draft is not null && moved.Draft is not null, aliceClient, moved);
            await Settle(() => queues.Status(Channel) is { Waiting: 0, Checking: 0, Underway: 1 });
            HostedDraft draft = Assert.Single(drafts.Drafts);
            Assert.Equal(Channel, draft.Queue);
            Assert.Equal((Alice, Carol), (draft.Host, draft.Guest!));
            Assert.Equal(2, (await drafts.ReturnAsync(Carol)).Seat);
            Assert.NotNull((await queues.JoinAsync(Channel, Alice)).Refusal);

            aliceClient.Disconnect();
            carolClient.Disconnect();
            moved.Disconnect();
        }
    }

    [Fact]
    public async Task A_queue_with_a_bot_seats_one_player_and_the_bot_drafts_across_from_them()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            QueueSettings grid = new() { Channel = Channel, Cube = "rw-aggro", Format = "grid", Packs = 4, MinimumDeck = 10, Bots = 1 };
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "grid", Packs = 4, Bots = 2 }));
            Assert.Null(await queues.ConfigureAsync(grid));
            Assert.Equal(new QueueStatus(grid, 0, 1, 0, 0), queues.Status(Channel));

            // One player fills the table: the bot's seat is taken and ready from the start, so the ready check is hers.
            QueueSeating alice = await queues.JoinAsync(Channel, Alice);
            Assert.Equal(1, alice.Seat);
            await Settle(() => queues.Status(Channel) is { Waiting: 1, Seats: 1 });
            GameClient client = await Sit(alice.Address!, alice.Ticket!);
            await Until(() => client.Table?.ReadyLeftMs > 0, client);
            Assert.Equal(["Alice", "Bot"], client.Table!.Players.Select(player => player.Name));
            Assert.True(client.Table.Players[1] is { Ready: true, Connected: true });
            client.SetReady(true);

            // The draft begins, the draft host has it with the bot as her opponent, and the bot drafts its half.
            await Until(() => client.Draft is { Format: "grid" }, client);
            await Settle(() => drafts.Drafts.Count == 1 && queues.Status(Channel) is { Underway: 1, Waiting: 0 });
            HostedDraft draft = Assert.Single(drafts.Drafts);
            Assert.Equal((Alice, "Bot", 0UL), (draft.Host, draft.Guest!.Name, draft.Guest.Id));

            var alicePlays = new AutoSeat(client);
            DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(60);
            while (draft.Stage is EventStage.Seating or EventStage.Drafting)
            {
                if (!alicePlays.Step()) await Task.Delay(5);
                Assert.True(DateTime.UtcNow < giveUp, $"The draft never finished: {draft.Stage}, {client.Draft?.Active} to pick.");
            }

            Assert.True(draft.Stage is EventStage.DeckBuilding or EventStage.Playing, $"The draft ended as {draft.Stage}: {draft.Detail}");
            client.Disconnect();
        }
    }

    [Fact]
    public async Task A_booster_queue_with_seven_bots_drafts_its_pod_with_one_player()
    {
        (DraftQueues queues, DraftHost drafts) = Create(out PodHosts pods);
        await using (pods)
        await using (drafts)
        await using (queues)
        {
            Assert.NotNull(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "booster", Packs = 1, PackSize = 5, Bots = 8 }));
            Assert.Null(await queues.ConfigureAsync(new QueueSettings { Channel = Channel, Cube = "rw-aggro", Format = "booster", Packs = 1, PackSize = 5, Rounds = 1, MinimumDeck = 5, Bots = 7 }));

            QueueSeating alice = await queues.JoinAsync(Channel, Alice);
            GameClient client = await Sit(alice.Address!, alice.Ticket!);
            await Until(() => client.Table?.ReadyLeftMs > 0, client);
            Assert.Equal(["Alice", .. Enumerable.Range(1, 7).Select(bot => $"Bot {bot}")], client.Table!.Players.Select(player => player.Name));
            client.SetReady(true);

            // Every pack goes round the seven bots and back to her, so the draft ends only if each of them picks.
            await Until(() => client.Draft is { Format: "booster" }, client);
            await Settle(() => pods.Underway(Channel) == 1);
            var alicePlays = new AutoSeat(client);
            DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(60);
            while (client.Table?.Phase is not TablePhase.DeckBuilding)
            {
                if (!alicePlays.Step()) await Task.Delay(5);
                Assert.True(DateTime.UtcNow < giveUp, $"The pod never finished drafting: pick {client.Draft?.PickNumber} of pack {client.Draft?.PackNumber}.");
            }

            client.Disconnect();
        }
    }

    [Fact]
    public async Task A_player_has_one_seat_across_queues_and_closing_a_queue_tells_its_players()
    {
        (DraftQueues queues, DraftHost drafts) = Create();
        await using (drafts)
        await using (queues)
        {
            await queues.ConfigureAsync(Settings());
            await queues.ConfigureAsync(new QueueSettings { Channel = Channel + 1, Cube = "rw-aggro", Cards = 18 });
            QueueSeating alice = await queues.JoinAsync(Channel, Alice);
            QueueSeating elsewhere = await queues.JoinAsync(Channel + 1, Alice);
            Assert.Contains($"<#{Channel}>", elsewhere.Refusal, StringComparison.Ordinal);

            GameClient client = await Sit(alice.Address!, alice.Ticket!);
            Assert.NotNull(await queues.RemoveAsync(Channel));
            await Until(() => client.Released is not null, client);
            Assert.Contains("closed", client.Released!.Detail, StringComparison.Ordinal);
            Assert.Null(queues.Status(Channel));
            client.Disconnect();
        }
    }

    private static async Task Settle(Func<bool> condition)
    {
        DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(30);
        while (!condition())
        {
            Assert.True(DateTime.UtcNow < giveUp, "The queue never got there.");
            await Task.Delay(5);
        }
    }
}
