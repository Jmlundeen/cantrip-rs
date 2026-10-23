using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// An event's table over real Kestrel and WebSockets: seated only by the event's tickets, a ticket that takes its seat
/// over from another computer, a pick sent twice, and a table stopped mid-draft and hosted again from the data
/// directory, where the same tickets still work and the draft goes on from the same decision.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class EventTransportTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-event-transport-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private static Cantrip.Datasets.ContentLibrary Content => ContentCache.Get(RepoRoot.ContentRoot);

    private (EventStore Store, CreatedEvent Created) Create()
    {
        var store = new EventStore(_data);
        return (store, DraftPod.CreateWinston(store, Content, new WinstonDraftSetup
        {
            Name = "Transport draft",
            Cube = "rw-aggro",
            CubeCards = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
            Cards = 18,
            Seed = 3,
            Players = [("alice", "Alice"), ("bob", "Bob")],
        }));
    }

    private static Task<GameServer> Host(EventStore store, string id) => GameServer.StartAsync(new GameServerOptions
    {
        Format = Cantrip.Decks.DeckFormat.Limited(),
        Port = 0,
        Players = 2,
        ContentRoot = RepoRoot.ContentRoot,
        Event = DraftPod.Open(store, id, Content),
    });

    private sealed class Clock : TimeProvider
    {
        private DateTimeOffset _now = new(2026, 10, 1, 18, 0, 0, TimeSpan.Zero);
        private readonly object _gate = new();
        public override DateTimeOffset GetUtcNow() { lock (_gate) return _now; }
        public void Advance(TimeSpan by) { lock (_gate) _now += by; }
    }

    private static readonly System.Runtime.CompilerServices.ConditionalWeakTable<GameClient, WebSocketTransport> Transports = new();

    private static async Task<GameClient> Sit(GameServer server, string? ticket)
    {
        WebSocketTransport transport = await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{server.Port}/game"));
        var client = new GameClient(transport);
        Transports.Add(client, transport);
        client.Join("Anyone", 0, ticket);
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
                $"seat {client.Membership?.Seat}, connected {client.IsConnected} ({(Transports.TryGetValue(client, out WebSocketTransport? socket) ? socket.LastError : null)}), phase {client.Table?.Phase}, draft {client.Draft?.Decision}, "
                + $"errors [{string.Join("; ", client.Errors.Select(error => error.Detail))}]")));
            await Task.Delay(1);
        }
    }

    [Fact]
    public async Task Nobody_waits_on_a_player_who_has_gone_from_drafting_to_deck_building_to_the_game()
    {
        (EventStore store, CreatedEvent created) = Create();
        var clock = new Clock();
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Players = 2,
            ContentRoot = RepoRoot.ContentRoot,
            TimeProvider = clock,
            Event = DraftPod.Open(store, created.Record.Id, Content, clock),
        });
        GameClient alice = await Sit(server, created.Tickets["alice"]);
        GameClient bob = await Sit(server, created.Tickets["bob"]);

        // Nobody picks: every turn's clock runs out and plays it.
        await Until(() => alice.Draft is not null, alice, bob);
        while (alice.Table?.Phase == TablePhase.Drafting)
        {
            int decision = alice.Draft!.Decision;
            clock.Advance(TimeSpan.FromSeconds(61));
            await Until(() => alice.Table?.Phase != TablePhase.Drafting || alice.Draft!.Decision != decision, alice, bob);
        }

        // Nobody builds: the decks as they stand are submitted, and the game dealt.
        await Until(() => alice.DeckBuilding is { LeftMs: > 0 }, alice, bob);
        clock.Advance(TimeSpan.FromMinutes(10));
        await Until(() => alice.Table?.Phase == TablePhase.Playing, alice, bob);

        // Bob leaves the game, and forfeits it once he has been gone five minutes.
        Assert.True(Transports.TryGetValue(bob, out WebSocketTransport? socket));
        socket!.Dispose();
        await Until(() => alice.Table?.Players.SingleOrDefault(player => player.Seat == bob.Membership!.Seat) is { Connected: false }, alice);
        clock.Advance(TimeSpan.FromMinutes(5));
        await Until(() => alice.Table?.Phase == TablePhase.Finished, alice);

        // The table says the game is over a moment before the event has it saved.
        await Until(() => store.LoadEvent(created.Record.Id)!.Stage == EventStage.Complete, alice);
        EventRecord ended = store.LoadEvent(created.Record.Id)!;
        Assert.True(ended.Matches.Single().Forfeit);
        Assert.Equal(["alice"], ended.Matches.Single().Winners);
        Assert.Contains("Bob was not at the table", alice.Table!.Detail, StringComparison.Ordinal);
    }

    [Fact]
    public async Task An_events_table_seats_only_its_players_by_their_tickets_under_their_own_names()
    {
        (EventStore store, CreatedEvent created) = Create();
        await using GameServer server = await Host(store, created.Record.Id);

        GameClient bare = await Sit(server, null);
        Assert.Null(bare.Membership);
        Assert.Equal(ErrorCode.MembershipRefused, bare.Errors.Single().Code);

        GameClient bob = await Sit(server, created.Tickets["bob"]);
        Assert.Equal(2, bob.Membership!.Seat);
        await Until(() => bob.Table?.Players.Count == 2, bob);
        Assert.Equal(created.Record.Id, bob.Table!.EventId);
        Assert.Equal(TablePhase.Setup, bob.Table.Phase);
        Assert.Equal(["Alice", "Bob"], bob.Table.Players.Select(player => player.Name));
        Assert.False(bob.Table.Players[0].Connected);

        GameClient alice = await Sit(server, created.Tickets["alice"]);
        await Until(() => alice.Draft is not null && bob.Draft is not null, alice, bob);
        Assert.Equal(TablePhase.Drafting, alice.Table!.Phase);

        bare.Disconnect();
        alice.Disconnect();
        bob.Disconnect();
        ServerAssert.DidNotFault(server);
    }

    [Fact]
    public async Task An_events_table_shows_each_player_their_picture_where_they_have_one()
    {
        (EventStore store, CreatedEvent created) = Create();
        var portraits = new PortraitStore(store.Root);
        await portraits.SaveAsync("bob", [1, 2, 3]);
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Players = 2,
            ContentRoot = RepoRoot.ContentRoot,
            Event = DraftPod.Open(store, created.Record.Id, Content),
            Portraits = portraits.PathFor,
        });

        GameClient bob = await Sit(server, created.Tickets["bob"]);
        await Until(() => bob.Table?.Players.Count == 2, bob);
        Assert.Equal(string.Empty, bob.Table!.Players[0].Portrait);
        Assert.Equal(portraits.PathFor("bob"), bob.Table.Players[1].Portrait);

        bob.Disconnect();
        ServerAssert.DidNotFault(server);
    }

    [Fact]
    public async Task A_ticket_opened_on_another_computer_takes_the_seat_over_from_the_first()
    {
        (EventStore store, CreatedEvent created) = Create();
        await using GameServer server = await Host(store, created.Record.Id);
        GameClient bob = await Sit(server, created.Tickets["bob"]);
        GameClient first = await Sit(server, created.Tickets["alice"]);
        await Until(() => first.Draft is not null, first);

        GameClient second = await Sit(server, created.Tickets["alice"]);
        // Told why before it is closed: waited for by the message itself, since a socket can read as closed before the
        // last thing sent on it has been taken in.
        await Until(() => second.Draft is not null && first.Errors.Any(error => error.Code == ErrorCode.MembershipRefused), first, second);
        await Until(() => !first.IsConnected, first);

        Assert.Equal(1, second.Membership!.Seat);
        Assert.Contains(first.Errors, error => error.Code == ErrorCode.MembershipRefused);
        Assert.Equal(first.Draft!.Decision, second.Draft!.Decision);

        bob.Disconnect();
        second.Disconnect();
        ServerAssert.DidNotFault(server);
    }

    [Fact]
    public async Task A_pick_sent_twice_is_refused_the_second_time_and_the_draft_shown_again()
    {
        (EventStore store, CreatedEvent created) = Create();
        await using GameServer server = await Host(store, created.Record.Id);
        GameClient alice = await Sit(server, created.Tickets["alice"]);
        GameClient bob = await Sit(server, created.Tickets["bob"]);
        await Until(() => alice.Draft is not null && bob.Draft is not null, alice, bob);

        GameClient active = alice.Draft!.Active == 1 ? alice : bob;
        GameClient waiting = active == alice ? bob : alice;
        int decision = active.Draft!.Decision;
        active.Pick(take: false);
        // The same click again, from a window that had not heard back: the same decision named.
        active.Pick(take: false);
        await Until(() => waiting.Draft?.Decision == decision + 1 && active.Errors.Count > 0, active, waiting);

        Assert.Equal(decision + 1, active.Draft!.Decision);
        Assert.Equal(1, active.Draft.Looking);
        Assert.Equal(ErrorCode.InvalidSetup, active.Errors.Single().Code);

        alice.Disconnect();
        bob.Disconnect();
        ServerAssert.DidNotFault(server);
    }

    [Fact]
    public async Task A_table_stopped_mid_draft_and_hosted_again_goes_on_from_the_same_decision_with_the_same_tickets()
    {
        (EventStore store, CreatedEvent created) = Create();
        int decision;
        List<string> pool;
        await using (GameServer server = await Host(store, created.Record.Id))
        {
            GameClient alice = await Sit(server, created.Tickets["alice"]);
            GameClient bob = await Sit(server, created.Tickets["bob"]);
            await Until(() => alice.Draft is not null && bob.Draft is not null, alice, bob);

            for (int i = 0; i < 4; i++)
            {
                GameClient active = alice.Draft!.Active == 1 ? alice : bob;
                int was = active.Draft!.Decision;
                active.Pick(take: i % 2 == 1);
                await Until(() => alice.Draft!.Decision == was + 1 && bob.Draft!.Decision == was + 1, alice, bob);
            }

            decision = alice.Draft!.Decision;
            pool = [.. alice.Draft.Pool.Select(card => card.CardId)];
            alice.Disconnect();
            bob.Disconnect();
        }

        await using GameServer again = await Host(store, created.Record.Id);
        GameClient back = await Sit(again, created.Tickets["alice"]);
        await Until(() => back.Draft is not null, back);

        Assert.Equal(1, back.Membership!.Seat);
        Assert.Equal(TablePhase.Drafting, back.Table!.Phase);
        Assert.Equal(decision, back.Draft!.Decision);
        Assert.Equal(pool, back.Draft.Pool.Select(card => card.CardId));
        back.Disconnect();
        ServerAssert.DidNotFault(again);
    }
}
