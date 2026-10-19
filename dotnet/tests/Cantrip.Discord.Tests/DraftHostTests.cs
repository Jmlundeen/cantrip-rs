using System.Net;
using System.Net.Sockets;
using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The draft host behind <c>/cantrip draft</c>, with real tables on real sockets and a data directory of the test's own:
/// a draft opened and joined becomes an event, each player's ticket is theirs to ask for again from anywhere, and a
/// restarted host takes every live draft up again from the data directory.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class DraftHostTests : IDisposable
{
    private static readonly DiscordPlayer Alice = new(1001, "Alice");
    private static readonly DiscordPlayer Bob = new(1002, "Bob");
    private static readonly DiscordPlayer Carol = new(1003, "Carol");

    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-draft-host-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private sealed class Clock : TimeProvider
    {
        private long _ticks = new DateTimeOffset(2026, 9, 30, 0, 0, 0, TimeSpan.Zero).Ticks;
        public override DateTimeOffset GetUtcNow() => new(Interlocked.Read(ref _ticks), TimeSpan.Zero);
        public void Advance(TimeSpan duration) => Interlocked.Add(ref _ticks, duration.Ticks);
    }

    private DraftHost Create(Clock? clock = null) => new(new DraftHostOptions
    {
        ContentRoot = RepoRoot.ContentRoot,
        Store = new EventStore(_data),
        Cube = "rw-aggro",
        CubeCards = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
        Cards = 18,
        Ports = FreePorts(2),
        TimeProvider = (TimeProvider?)clock ?? TimeProvider.System,
    });

    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }

    private static async Task<GameClient> Sit(HostedDraft draft, string ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(draft.Address!));
        client.Join("ignored", ticket: ticket);
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        while (client.Membership is null && client.Errors.Count == 0)
        {
            client.Pump();
            await Task.Delay(1, timeout.Token);
        }

        return client;
    }

    [Fact]
    public async Task A_draft_becomes_an_event_when_its_second_player_joins_and_each_player_is_seated_by_their_ticket()
    {
        await using DraftHost host = Create();
        DraftSeating opened = await host.OpenAsync(Alice);
        Assert.Null(opened.Ticket);
        Assert.NotNull((await host.JoinAsync(opened.Draft!.Id, Alice)).Refusal);

        DraftSeating bob = await host.JoinAsync(opened.Draft.Id, Bob);
        DraftSeating alice = await host.JoinAsync(opened.Draft.Id, Alice);
        Assert.Equal(2, bob.Seat);
        Assert.Equal(1, alice.Seat);

        EventRecord record = new EventStore(_data).LoadEvent(opened.Draft.Id)!;
        Assert.Equal(["discord-1001", "discord-1002"], record.Participants.Select(participant => participant.PlayerId));
        Assert.Equal("1001:Alice", record.HostData[DraftHost.HostKey]);
        Assert.Contains(new EventStore(_data).LoadPlayer("discord-1002")!.Events, entry => entry.EventId == record.Id);

        GameClient aliceClient = await Sit(alice.Draft!, alice.Ticket!);
        GameClient bobClient = await Sit(bob.Draft!, bob.Ticket!);
        Assert.Equal(1, aliceClient.Membership!.Seat);
        Assert.Equal(2, bobClient.Membership!.Seat);
        aliceClient.Disconnect();
        bobClient.Disconnect();

        Assert.NotNull((await host.JoinAsync(opened.Draft.Id, Carol)).Refusal);
    }

    [Fact]
    public async Task A_player_is_in_one_live_draft_at_a_time()
    {
        await using DraftHost host = Create();
        DraftSeating first = await host.OpenAsync(Alice);

        Assert.NotNull((await host.OpenAsync(Alice)).Refusal);
        await host.JoinAsync(first.Draft!.Id, Bob);
        DraftSeating other = await host.OpenAsync(Carol);
        Assert.NotNull((await host.JoinAsync(other.Draft!.Id, Bob)).Refusal);
    }

    [Fact]
    public async Task A_player_who_drops_calls_the_draft_off_for_both_and_is_free_to_join_another()
    {
        await using DraftHost host = Create();
        DraftSeating opened = await host.OpenAsync(Alice);
        await host.JoinAsync(opened.Draft!.Id, Bob);
        Assert.True(host.Hosts(opened.Draft.Id));
        Assert.True(host.InDraft(Bob.Id));

        Assert.Equal("You are not in that draft.", await host.DropAsync(opened.Draft.Id, Carol));
        Assert.Null(await host.DropAsync(opened.Draft.Id, Bob));

        // Over for both, kept so, and neither is in a draft any more.
        EventRecord record = new EventStore(_data).LoadEvent(opened.Draft.Id)!;
        Assert.Equal(EventStage.Cancelled, record.Stage);
        Assert.Equal("Bob dropped from the draft.", record.Detail);
        Assert.True(record.Participant(DraftHost.PlayerId(Bob))!.Dropped);
        Assert.False(host.Hosts(opened.Draft.Id));
        Assert.False(host.InDraft(Bob.Id));
        Assert.False(host.InDraft(Alice.Id));
        Assert.Equal("That draft is over.", await host.DropAsync(opened.Draft.Id, Alice));
        Assert.Null((await host.OpenAsync(Bob)).Refusal);

        // A draft still waiting for its second player is simply called off.
        DraftSeating waiting = await host.OpenAsync(Carol);
        Assert.Equal([waiting.Draft!.Id], host.Waiting(Carol.Id).Select(draft => draft.Id));
        Assert.Null(await host.DropAsync(waiting.Draft.Id, Carol));
        Assert.Empty(host.Waiting(Carol.Id));
        Assert.False(host.InDraft(Carol.Id));
    }

    [Fact]
    public async Task Only_the_host_or_a_moderator_cancels_and_the_event_is_kept_as_cancelled()
    {
        await using DraftHost host = Create();
        DraftSeating opened = await host.OpenAsync(Alice);
        await host.JoinAsync(opened.Draft!.Id, Bob);

        Assert.NotNull(await host.CancelAsync(opened.Draft.Id, Bob, moderator: false));
        Assert.Null(await host.CancelAsync(opened.Draft.Id, Alice, moderator: false));

        Assert.Equal(EventStage.Cancelled, new EventStore(_data).LoadEvent(opened.Draft.Id)!.Stage);
        Assert.NotNull((await host.JoinAsync(opened.Draft.Id, Alice)).Refusal);
    }

    [Fact]
    public async Task A_draft_nobody_joins_expires_and_one_whose_players_do_not_all_sit_down_is_called_off()
    {
        var clock = new Clock();
        await using DraftHost host = Create(clock);
        DraftSeating lonely = await host.OpenAsync(Alice);
        DraftSeating busy = await host.OpenAsync(Bob);
        DraftSeating joined = await host.JoinAsync(busy.Draft!.Id, Carol);
        GameClient carol = await Sit(joined.Draft!, joined.Ticket!);
        Assert.Equal(2, carol.Membership!.Seat);

        clock.Advance(TimeSpan.FromMinutes(30));
        await host.SweepAsync();
        Assert.NotNull((await host.JoinAsync(lonely.Draft!.Id, Carol)).Refusal);

        // Bob never took his seat: the draft's seating clock calls it off, and its table is closed once that has been seen.
        HostedDraft draft = joined.Draft!;
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        while (draft.IsLive)
        {
            await Task.Delay(10, timeout.Token);
        }

        Assert.Equal(EventStage.Cancelled, draft.Stage);
        Assert.Equal("Bob did not take their seat in time.", draft.Detail);
        Assert.NotNull((await host.JoinAsync(draft.Id, Bob)).Refusal);

        clock.Advance(TimeSpan.FromMinutes(2));
        await host.SweepAsync();
        Assert.DoesNotContain(host.Drafts, each => each.Id == draft.Id);
    }

    [Fact]
    public async Task A_restarted_host_takes_up_every_live_draft_and_the_old_tickets_still_seat_their_players()
    {
        string id;
        string ticket;
        await using (DraftHost first = Create())
        {
            DraftSeating opened = await first.OpenAsync(Alice);
            ticket = (await first.JoinAsync(opened.Draft!.Id, Bob)).Ticket!;
            id = opened.Draft.Id;
        }

        await using DraftHost again = Create();
        IReadOnlyList<HostedDraft> resumed = await again.ResumeAsync();

        HostedDraft draft = Assert.Single(resumed);
        Assert.Equal(id, draft.Id);
        Assert.Equal(Alice, draft.Host);
        Assert.Equal(Bob, draft.Guest);
        GameClient bob = await Sit(draft, ticket);
        Assert.Equal(2, bob.Membership!.Seat);
        bob.Disconnect();

        // And a fresh one, for a player on another computer, found from their record without the message.
        DraftSeating fresh = await again.ReturnAsync(Alice);
        Assert.Equal(id, fresh.Draft!.Id);
        Assert.Equal(1, fresh.Seat);
        Assert.NotEqual(ticket, fresh.Ticket);
        Assert.NotNull((await again.ReturnAsync(Carol)).Refusal);
        Assert.NotNull((await again.OpenAsync(Bob)).Refusal);
    }
}
