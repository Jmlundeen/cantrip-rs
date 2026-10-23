using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// A table formed from a queue: seats taken and given up, a ready check once every seat is taken that gives up the
/// seats of whoever did not say ready in time, and the draft begun at the same table when everyone did. The clock is the
/// test's own, moved on rather than waited for. Ends with the same over real sockets.
/// </summary>
public sealed class QueueTableTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-queue-table-" + Guid.NewGuid().ToString("N"));
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
            CreatedEvent created = DraftPod.CreateWinston(store, ContentCache.Get(RepoRoot.ContentRoot), new WinstonDraftSetup
            {
                Name = "Winston draft: rw-aggro",
                Cube = "rw-aggro",
                CubeCards = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
                Cards = 18,
                Players = players,
            }, _clock);
            return DraftPod.Open(store, created.Record.Id, ContentCache.Get(RepoRoot.ContentRoot), _clock);
        },
    });

    [Fact]
    public void The_ready_check_starts_when_every_seat_is_taken_and_every_player_has_the_table_open()
    {
        QueueTable table = Table();
        string alice = table.Reserve("alice", "Alice")!;
        Assert.Equal(1, table.SeatFor(alice));
        table.Connected(1, true);
        Assert.False(table.Checking);
        Assert.NotNull(table.Receive(1, new ReadyMessage { Ready = true }));

        // Bob has his seat, but the thirty seconds wait for him to open the table, however long that takes him.
        string bob = table.Reserve("bob", "Bob")!;
        Assert.Equal(2, table.SeatFor(bob));
        Assert.False(table.Checking);
        Assert.Null(table.Reserve("carol", "Carol"));
        _clock.Advance(TimeSpan.FromSeconds(25));
        table.Tick(_clock.GetUtcNow());
        table.Connected(2, true);
        Assert.True(table.Checking);
        Assert.Equal(TimeSpan.FromSeconds(30), table.ReadyLeft(_clock.GetUtcNow()));

        Assert.Null(table.Receive(1, new ReadyMessage { Ready = true }));
        Assert.True(table.IsReady(1));
        Assert.True(table.Forming);
        Assert.Null(table.Receive(2, new ReadyMessage { Ready = true }));

        Assert.False(table.Forming);
        Assert.Equal(TablePhase.Drafting, table.Phase);
        Assert.Equal(table.Pod!.EventId, table.EventId);
        Assert.Equal(1, table.SeatFor(alice));
        Assert.IsType<DraftMessage>(table.ViewFor(1));
    }

    [Fact]
    public void The_draft_formed_knows_who_is_at_the_table_and_its_game_has_a_match_clock()
    {
        QueueTable table = Table();
        table.Reserve("alice", "Alice");
        table.Reserve("bob", "Bob");
        table.Connected(1, true);
        table.Connected(2, true);
        table.Receive(1, new ReadyMessage { Ready = true });
        table.Receive(2, new ReadyMessage { Ready = true });

        // Nobody picks or builds: the clocks do it for them, and the game is dealt.
        while (table.Phase == TablePhase.Drafting)
        {
            _clock.Advance(TimeSpan.FromMinutes(1));
            table.Tick(_clock.GetUtcNow());
            table.ViewFor(1);
        }

        _clock.Advance(TimeSpan.FromMinutes(10));
        table.Tick(_clock.GetUtcNow());
        Assert.True(table.ReadyToDeal);
        table.Deal(["Alice", "Bob"], new CapturingRulesFactory());
        Assert.Equal(TimeSpan.FromHours(1), table.ClockAtDeal(1));

        // Both had the table open from before the draft began: neither is taken for a no-show.
        _clock.Advance(TimeSpan.FromMinutes(4));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeForfeits());

        table.Connected(2, false);
        _clock.Advance(TimeSpan.FromMinutes(5));
        table.Tick(_clock.GetUtcNow());
        Assert.Equal([2], table.TakeForfeits());
    }

    [Fact]
    public void Each_seat_names_its_player_by_id_while_forming_and_once_the_draft_has_begun()
    {
        QueueTable table = Table();
        table.Reserve("alice", "Alice");
        Assert.Equal("alice", table.PlayerAt(1));
        Assert.Equal(string.Empty, table.PlayerAt(2));

        table.Reserve("bob", "Bob");
        table.Connected(1, true);
        table.Connected(2, true);
        table.Receive(1, new ReadyMessage { Ready = true });
        table.Receive(2, new ReadyMessage { Ready = true });

        Assert.False(table.Forming);
        Assert.Equal(["alice", "bob"], [table.PlayerAt(1), table.PlayerAt(2)]);
    }

    [Fact]
    public void Whoever_is_not_ready_in_time_loses_their_seat_and_whoever_was_keeps_theirs()
    {
        QueueTable table = Table();
        string alice = table.Reserve("alice", "Alice")!;
        table.Reserve("bob", "Bob");
        table.Connected(1, true);
        table.Connected(2, true);
        table.Receive(2, new ReadyMessage { Ready = true });

        _clock.Advance(TimeSpan.FromSeconds(29));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());

        _clock.Advance(TimeSpan.FromSeconds(2));
        table.Tick(_clock.GetUtcNow());
        (int seat, ServerMessage farewell) = Assert.Single(table.TakeVacated());
        Assert.Equal(1, seat);
        Assert.Contains("Ready", Assert.IsType<SeatReleasedMessage>(farewell).Detail, StringComparison.Ordinal);
        Assert.Equal(0, table.SeatFor(alice));
        Assert.True(table.Holds(2));
        Assert.False(table.Checking);
        Assert.False(table.IsReady(2));

        // The next to join takes the seat, and everyone is asked again once they have the table open.
        Assert.Equal(1, table.SeatFor(table.Reserve("carol", "Carol")!));
        Assert.False(table.Checking);
        table.Connected(1, true);
        Assert.True(table.Checking);
    }

    [Fact]
    public void A_player_who_joins_and_never_opens_the_table_loses_the_seat_and_asking_again_gives_them_more_time()
    {
        QueueTable table = Table();
        table.Reserve("alice", "Alice");

        _clock.Advance(TimeSpan.FromSeconds(29));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());

        // Join pressed again for a fresh link: another thirty seconds.
        table.Reserve("alice", "Alice");
        _clock.Advance(TimeSpan.FromSeconds(29));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());

        _clock.Advance(TimeSpan.FromSeconds(2));
        table.Tick(_clock.GetUtcNow());
        Assert.Contains("wasn't open", Assert.IsType<SeatReleasedMessage>(Assert.Single(table.TakeVacated()).Farewell).Detail, StringComparison.Ordinal);
        Assert.Equal(0, table.Taken);
    }

    [Fact]
    public void A_player_who_closes_the_table_while_it_forms_has_a_while_to_come_back()
    {
        QueueTable table = Table();
        table.Reserve("alice", "Alice");
        table.Connected(1, true);

        _clock.Advance(TimeSpan.FromMinutes(5));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());

        table.Connected(1, false);
        _clock.Advance(TimeSpan.FromSeconds(20));
        table.Connected(1, true);
        _clock.Advance(TimeSpan.FromSeconds(20));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());

        table.Connected(1, false);
        _clock.Advance(TimeSpan.FromSeconds(31));
        table.Tick(_clock.GetUtcNow());
        Assert.Single(table.TakeVacated());
    }

    [Fact]
    public void A_draft_that_cannot_be_started_keeps_everyone_seated_with_a_fresh_ready_check()
    {
        var table = new QueueTable(new QueueTableSetup
        {
            Name = "Broken", TimeProvider = _clock, Start = _ => throw new KeyNotFoundException("no such cube card"),
        });
        table.Reserve("alice", "Alice");
        table.Reserve("bob", "Bob");
        table.Connected(1, true);
        table.Connected(2, true);
        _clock.Advance(TimeSpan.FromSeconds(20));
        Assert.Null(table.Receive(1, new ReadyMessage { Ready = true }));

        string? refused = table.Receive(2, new ReadyMessage { Ready = true });

        Assert.Contains("no such cube card", refused, StringComparison.Ordinal);
        Assert.True(table.Forming);
        Assert.Equal(2, table.Taken);
        Assert.Equal(TimeSpan.FromSeconds(30), table.ReadyLeft(_clock.GetUtcNow()));
        _clock.Advance(TimeSpan.FromSeconds(15));
        table.Tick(_clock.GetUtcNow());
        Assert.Empty(table.TakeVacated());
    }

    [Fact]
    public void Leaving_gives_the_seat_up_and_ends_the_ready_check()
    {
        QueueTable table = Table();
        table.Reserve("alice", "Alice");
        table.Reserve("bob", "Bob");
        table.Connected(1, true);
        table.Connected(2, true);
        table.Receive(2, new ReadyMessage { Ready = true });

        Assert.Null(table.Receive(1, new LeaveMessage()));

        Assert.Equal("You left the table.", Assert.IsType<SeatReleasedMessage>(Assert.Single(table.TakeVacated()).Farewell).Detail);
        Assert.False(table.Checking);
        Assert.False(table.IsReady(2));
        Assert.False(table.Holds(1));
        Assert.Equal(1, table.Taken);
    }

    [Fact]
    [Trait("Category", "WallClock")]
    public async Task Over_real_sockets_the_players_see_the_table_fill_and_the_one_who_did_not_ready_is_told_why()
    {
        QueueTable table = Table();
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, Event = table, TimeProvider = _clock,
        });

        GameClient alice = await Sit(server, table.Reserve("alice", "Alice")!);
        await Until(() => alice.Table is { Forming: true } shown && shown.Players.Count == 2, alice);
        Assert.Equal(["Alice", string.Empty], alice.Table!.Players.Select(player => player.Name));
        Assert.Equal(0, alice.Table.ReadyLeftMs);

        // A seat taken from outside, by the queue, is shown at once.
        GameClient bob = await Sit(server, table.Reserve("bob", "Bob")!);
        await Until(() => alice.Table!.ReadyLeftMs > 0 && bob.Table?.ReadyLeftMs > 0, alice, bob);

        alice.SetReady(true);
        await Until(() => bob.Table!.Players[0].Ready, alice, bob);

        _clock.Advance(TimeSpan.FromSeconds(31));
        await Until(() => bob.Released is not null && alice.Table!.Players[1].Name.Length == 0, alice, bob);
        Assert.False(alice.Table!.Players[0].Ready);

        GameClient carol = await Sit(server, table.Reserve("carol", "Carol")!);
        await Until(() => carol.Table?.ReadyLeftMs > 0, alice, carol);
        alice.SetReady(true);
        carol.SetReady(true);
        await Until(() => alice.Draft is not null && carol.Draft is not null, alice, carol);
        Assert.Equal(TablePhase.Drafting, alice.Table!.Phase);

        alice.Disconnect();
        carol.Disconnect();
        ServerAssert.DidNotFault(server);
    }

    private static async Task<GameClient> Sit(GameServer server, string ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{server.Port}/game")));
        client.Join("ignored", 0, ticket);
        await Until(() => client.Membership is not null || client.Errors.Count > 0, client);
        return client;
    }

    private static async Task Until(Func<bool> condition, params GameClient[] clients)
    {
        DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(30);
        while (true)
        {
            foreach (GameClient client in clients) client.Pump();
            if (condition()) return;
            Assert.True(DateTime.UtcNow < giveUp, "Gave up waiting. " + string.Join(" | ", clients.Select(client =>
                $"seat {client.Membership?.Seat}, phase {client.Table?.Phase}, forming {client.Table?.Forming}, ready left {client.Table?.ReadyLeftMs}, "
                + $"players [{string.Join(", ", client.Table?.Players.Select(player => $"{player.Name}{(player.Ready ? "*" : string.Empty)}") ?? [])}], "
                + $"released {client.Released?.Detail}, errors [{string.Join("; ", client.Errors.Select(error => error.Detail))}]")));
            await Task.Delay(1);
        }
    }
}
