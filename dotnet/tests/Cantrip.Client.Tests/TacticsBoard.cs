using Cantrip.Protocol;

namespace Cantrip.Client.Tests;

/// <summary>
/// A client holding a position a test wrote, so the tactical seat can be asked about a board without a game behind it.
/// <para>
/// The seat reads nothing but the view and the prompt, which is what makes this enough: a test states the objects it
/// wants on the table and the question it wants answered, and gets the code path a real game takes.
/// </para>
/// </summary>
internal static class TacticsBoard
{
    internal const int Me = 1;
    internal const int Them = 2;

    internal static ViewHandle Handle(int value) => new(value);

    /// <summary>A client whose view holds these objects, on this seat's own precombat main phase unless told otherwise.</summary>
    internal static GameClient Seated(
        IEnumerable<ObjectView> objects,
        int myLife = 20,
        int theirLife = 20,
        int active = Me,
        TurnStepView step = TurnStepView.PrecombatMain,
        int turn = 5,
        CombatView? combat = null) => new Table(objects, myLife, theirLife, active, step, turn, combat).Client;

    /// <summary>A seated client and the means to change what it sees, for a test that follows a seat across two views.</summary>
    internal sealed class Table
    {
        private readonly Held _transport = new();

        internal Table(
            IEnumerable<ObjectView> objects,
            int myLife = 20,
            int theirLife = 20,
            int active = Me,
            TurnStepView step = TurnStepView.PrecombatMain,
            int turn = 5,
            CombatView? combat = null)
        {
            Client = new GameClient(_transport);
            _transport.Inbound.Enqueue(new WelcomeMessage { Seat = Me, PlayerNames = ["Bot 1", "Bot 2"] });
            Show(objects, myLife, theirLife, active, step, turn, combat);
        }

        internal GameClient Client { get; }

        /// <summary>What this seat has sent, oldest first.</summary>
        internal IReadOnlyList<ClientMessage> Sent => _transport.Sent;

        /// <summary>Puts a new view in front of the client, as the next state message would.</summary>
        internal void Show(
            IEnumerable<ObjectView> objects,
            int myLife = 20,
            int theirLife = 20,
            int active = Me,
            TurnStepView step = TurnStepView.PrecombatMain,
            int turn = 5,
            CombatView? combat = null)
        {
            PlayerView view = View([.. objects], myLife, theirLife, active, step, turn);
            view.Combat = combat ?? new CombatView();
            _transport.Inbound.Enqueue(new StateMessage { View = view });
            Client.Pump();
        }

        /// <summary>Asks the client something, as the server would.</summary>
        internal void Ask(int requestId, PromptPayload prompt)
        {
            _transport.Inbound.Enqueue(new PromptMessage { RequestId = requestId, Prompt = prompt });
            Client.Pump();
        }
    }

    internal static PlayerView View(List<ObjectView> objects, int myLife, int theirLife, int active, TurnStepView step, int turn)
    {
        ZoneView Visible(ZoneKindView kind, int owner, Func<ObjectView, bool> inZone) => new()
        {
            Kind = kind,
            Owner = owner,
            ContentsVisible = true,
            Contents = [.. objects.Where(inZone).Select(card => card.Handle)],
            Count = objects.Count(inZone),
        };

        return new PlayerView
        {
            Viewer = Me,
            Turn = turn,
            Step = step,
            ActivePlayer = active,
            PriorityPlayer = Me,
            Players =
            [
                new() { Seat = Me, Name = "Bot 1", Team = Me, Life = myLife, HandCount = objects.Count(card => card is { Zone: ZoneKindView.Hand, ZoneOwner: Me }), LibraryCount = 30 },
                new() { Seat = Them, Name = "Bot 2", Team = Them, Life = theirLife, HandCount = 3, LibraryCount = 30 },
            ],
            Objects = objects,
            Zones =
            [
                Visible(ZoneKindView.Battlefield, 0, card => card.Zone == ZoneKindView.Battlefield),
                Visible(ZoneKindView.Stack, 0, card => card.Zone == ZoneKindView.Stack),
                Visible(ZoneKindView.Hand, Me, card => card is { Zone: ZoneKindView.Hand, ZoneOwner: Me }),
                Visible(ZoneKindView.Graveyard, Me, card => card is { Zone: ZoneKindView.Graveyard, ZoneOwner: Me }),
                Visible(ZoneKindView.Graveyard, Them, card => card is { Zone: ZoneKindView.Graveyard, ZoneOwner: Them }),
                new() { Kind = ZoneKindView.Hand, Owner = Them, ContentsVisible = false, Count = 3 },
                new() { Kind = ZoneKindView.Library, Owner = Me, ContentsVisible = false, Count = 30 },
                new() { Kind = ZoneKindView.Library, Owner = Them, ContentsVisible = false, Count = 30 },
            ],
        };
    }

    /// <summary>A creature on the battlefield.</summary>
    internal static ObjectView Creature(int handle, string name, int power, int toughness, int controller = Me, string text = "", bool tapped = false) => new()
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
        OracleText = text,
        Tapped = tapped,
    };

    /// <summary>A basic land on the battlefield, untapped and able to make mana.</summary>
    internal static ObjectView Land(int handle, string name = "Mountain", int controller = Me, bool tapped = false) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = controller,
        Controller = controller,
        Name = name,
        CardTypes = [CardTypeView.Land],
        Supertypes = [SupertypeView.Basic],
        Subtypes = [name],
        ProducesMana = true,
        Tapped = tapped,
    };

    /// <summary>A card in this seat's hand.</summary>
    internal static ObjectView InHand(
        int handle, string name, CardTypeView type, int manaValue, string text = "", int? power = null, int? toughness = null, ColorView color = ColorView.Red) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Hand,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = name,
        ManaValue = manaValue,
        ManaCost = manaValue == 0 ? string.Empty : $"{{{manaValue}}}",
        OracleText = text,
        CardTypes = [type],
        Colors = type == CardTypeView.Land ? [] : [color],
        Power = power,
        Toughness = toughness,
        Subtypes = type == CardTypeView.Land ? [name] : [],
    };

    internal static TargetMessage At(int handle) => new() { Object = Handle(handle) };

    internal static TargetMessage AtPlayer(int seat) => new() { Player = seat };

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
