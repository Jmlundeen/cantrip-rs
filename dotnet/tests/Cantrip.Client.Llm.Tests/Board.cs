using Cantrip.Protocol;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// A client holding a position a test wrote, so a decision can be asked about a board without a game behind it.
/// <para>
/// The seat reads nothing but the view and the prompt, which is what makes this enough: a test states the objects
/// it wants named and the prompt it wants answered, and gets the same code path a real game takes.
/// </para>
/// </summary>
internal static class Board
{
    internal const int Me = 1;
    internal const int Them = 2;

    internal static ViewHandle Handle(int value) => new(value);

    /// <summary>A client whose view holds these objects, with two players at twenty life.</summary>
    internal static GameClient Seated(params ObjectView[] objects)
    {
        var transport = new Held();
        var client = new GameClient(transport);

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = View(objects) });
        client.Pump();

        return client;
    }

    internal static PlayerView View(params ObjectView[] objects) => new()
    {
        Viewer = Me,
        Turn = 3,
        Step = TurnStepView.PrecombatMain,
        ActivePlayer = Me,
        PriorityPlayer = Me,
        Players =
        [
            new() { Seat = Me, Name = "Bot 1", Team = Me, Life = 20, HandCount = 3, LibraryCount = 30 },
            new() { Seat = Them, Name = "Bot 2", Team = Them, Life = 20, HandCount = 4, LibraryCount = 29 },
        ],
        Objects = [.. objects],
        Zones =
        [
            new()
            {
                Kind = ZoneKindView.Battlefield,
                Owner = 0,
                ContentsVisible = true,
                Contents = [.. objects.Where(o => o.Zone == ZoneKindView.Battlefield).Select(o => o.Handle)],
                Count = objects.Count(o => o.Zone == ZoneKindView.Battlefield),
            },
            new()
            {
                Kind = ZoneKindView.Hand,
                Owner = Me,
                ContentsVisible = true,
                Contents = [.. objects.Where(o => o is { Zone: ZoneKindView.Hand, ZoneOwner: Me }).Select(o => o.Handle)],
                Count = objects.Count(o => o is { Zone: ZoneKindView.Hand, ZoneOwner: Me }),
            },
            new() { Kind = ZoneKindView.Hand, Owner = Them, ContentsVisible = false, Count = 4 },
            new() { Kind = ZoneKindView.Library, Owner = Me, ContentsVisible = false, Count = 30 },
        ],
    };

    /// <summary>A creature on the battlefield.</summary>
    internal static ObjectView Creature(int handle, string name, int power, int toughness, int controller = Me) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = controller,
        Controller = controller,
        Name = name,
        CardTypes = [CardTypeView.Creature],
        Power = power,
        Toughness = toughness,
    };

    /// <summary>A card in this seat's hand.</summary>
    internal static ObjectView Card(int handle, string name, string cost = "{1}", string text = "") => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Hand,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = name,
        ManaCost = cost,
        OracleText = text,
        CardTypes = [CardTypeView.Instant],
    };

    /// <summary>A transport that carries only what a test puts in it.</summary>
    internal sealed class Held : IGameTransport
    {
        internal Queue<ServerMessage> Inbound { get; } = [];

        internal List<ClientMessage> Sent { get; } = [];

        public bool IsConnected => true;

        public void Send(ClientMessage message) => Sent.Add(message);

        public IReadOnlyList<ServerMessage> Receive()
        {
            List<ServerMessage> messages = [.. Inbound];
            Inbound.Clear();
            return messages;
        }

        public void Dispose()
        {
        }
    }
}
