using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// The server's digest of each object it sends, and the client keeping the object it already has when the digest says it
/// is unchanged - so what a client worked out from an object holds until the object changes, not for one view.
/// </summary>
public sealed class ObjectFingerprintTests
{
    [Fact]
    public void The_same_object_sent_twice_has_the_same_fingerprint_and_any_change_gives_another()
    {
        ObjectView knight = Knight(handle: 7);
        long first = ObjectFingerprint.Of(knight);

        Assert.NotEqual(0, first);
        Assert.Equal(first, ObjectFingerprint.Of(Knight(handle: 7)));

        ObjectView tapped = Knight(handle: 7);
        tapped.Tapped = true;
        ObjectView pumped = Knight(handle: 7);
        pumped.Power = 3;
        ObjectView countered = Knight(handle: 7);
        countered.Counters = [new CounterView { Name = "+1/+1", Count = 1 }];

        Assert.NotEqual(first, ObjectFingerprint.Of(tapped));
        Assert.NotEqual(first, ObjectFingerprint.Of(pumped));
        Assert.NotEqual(first, ObjectFingerprint.Of(countered));
    }

    [Fact]
    public void A_fingerprint_already_stamped_does_not_change_the_next_one()
    {
        ObjectView knight = Knight(handle: 7);
        long clean = ObjectFingerprint.Of(knight);

        knight.Fingerprint = 12345;

        Assert.Equal(clean, ObjectFingerprint.Of(knight));
    }

    [Fact]
    public void An_unchanged_object_is_the_one_the_client_already_had_and_a_changed_one_is_new()
    {
        var transport = new QueueTransport();
        var client = new GameClient(transport);
        transport.Inbox.Enqueue(new WelcomeMessage { Seat = 1, PlayerNames = ["You", "Bot 2"] });
        transport.Inbox.Enqueue(State(Knight(handle: 1), Knight(handle: 2)));
        client.Pump();
        ObjectView firstKnight = client.Object(new ViewHandle(1))!;
        ObjectView secondKnight = client.Object(new ViewHandle(2))!;

        ObjectView tapped = Knight(handle: 2);
        tapped.Tapped = true;
        transport.Inbox.Enqueue(State(Knight(handle: 1), tapped));
        client.Pump();

        Assert.Same(firstKnight, client.Object(new ViewHandle(1)));
        Assert.NotSame(secondKnight, client.Object(new ViewHandle(2)));
        Assert.True(client.Object(new ViewHandle(2))!.Tapped);
    }

    [Fact]
    public void An_object_sent_without_a_fingerprint_is_never_taken_for_the_one_before()
    {
        var transport = new QueueTransport();
        var client = new GameClient(transport);
        transport.Inbox.Enqueue(new WelcomeMessage { Seat = 1, PlayerNames = ["You", "Bot 2"] });
        transport.Inbox.Enqueue(new StateMessage { View = new PlayerView { Viewer = 1, Objects = [Knight(handle: 1)] } });
        client.Pump();
        ObjectView first = client.Object(new ViewHandle(1))!;

        transport.Inbox.Enqueue(new StateMessage { View = new PlayerView { Viewer = 1, Objects = [Knight(handle: 1)] } });
        client.Pump();

        Assert.NotSame(first, client.Object(new ViewHandle(1)));
    }

    private static StateMessage State(params ObjectView[] objects)
    {
        var view = new PlayerView { Viewer = 1, Objects = [.. objects] };
        ObjectFingerprint.Stamp(view);
        return new StateMessage { View = view };
    }

    private static ObjectView Knight(int handle) => new()
    {
        Handle = new ViewHandle(handle),
        Zone = ZoneKindView.Battlefield,
        Controller = 1,
        Owner = 1,
        Known = true,
        Name = "Human Knight",
        CardTypes = [CardTypeView.Creature],
        Subtypes = ["Human", "Knight"],
        Power = 2,
        Toughness = 2,
    };

    /// <summary>Hands the client whatever has been queued for it by the time it pumps.</summary>
    private sealed class QueueTransport : IGameTransport
    {
        public Queue<ServerMessage> Inbox { get; } = new();

        public bool IsConnected => true;

        public void Send(ClientMessage message)
        {
        }

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> taken = [.. Inbox];
            Inbox.Clear();
            return taken;
        }

        public void Dispose()
        {
        }
    }
}
