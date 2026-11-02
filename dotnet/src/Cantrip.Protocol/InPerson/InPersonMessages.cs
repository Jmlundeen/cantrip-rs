using MessagePack;

namespace Cantrip.Protocol.InPerson;

/// <summary>
/// The wire between in-person play's devices and the relay that serves them (docs/in-person.md): a phone for each
/// player's hidden information and prompts, a tablet or two for the public board. Its own unions and its own version, so
/// nothing here touches <see cref="ServerMessage"/>, <see cref="ClientMessage"/> or <see cref="PlayerView.ProtocolVersion"/>.
/// </summary>
public static class InPersonProtocol
{
    /// <summary>The in-person wire's version, sent in every <see cref="DeviceHelloMessage"/>; a mismatch is refused.</summary>
    public const int Version = 6;
}

/// <summary>What a device is at the table.</summary>
public enum DeviceRole
{
    /// <summary>One player's own device: their hand, their library searches, their private choices and every prompt.</summary>
    Phone = 0,

    /// <summary>A public board: what anyone at the table may see, and where its permanents and players are tapped.</summary>
    Tablet = 1,
}

/// <summary>
/// Anything a device sends the relay. A MessagePack union with explicit keys, as <see cref="ClientMessage"/> is.
/// </summary>
[Union(0, typeof(DeviceHelloMessage))]
[Union(1, typeof(TableTapMessage))]
[Union(2, typeof(TapResultMessage))]
[Union(3, typeof(TableLineMessage))]
[Union(8, typeof(ReentryRequestMessage))]
[Union(9, typeof(TablePressMessage))]
[Union(10, typeof(SeatStopsMessage))]
[Union(11, typeof(SeatPromptMessage))]
public abstract class DeviceClientMessage
{
    public abstract string Describe();
}

/// <summary>
/// A device's first message: who it is. A phone brings its seat's ticket, a tablet its device token; either brings the
/// resume id and proof it was welcomed with before, to take its place back after a reload.
/// </summary>
[MessagePackObject]
public sealed class DeviceHelloMessage : DeviceClientMessage
{
    [Key(0)]
    public int InPersonVersion { get; set; } = InPersonProtocol.Version;

    [Key(1)]
    public int ProtocolVersion { get; set; } = PlayerView.ProtocolVersion;

    /// <summary>A phone's seat ticket, a bearer credential: never logged or described. Null for a tablet or a resume.</summary>
    [Key(2)]
    public string? Ticket { get; set; }

    /// <summary>A tablet's device token, a bearer credential: never logged or described. Null for a phone or a resume.</summary>
    [Key(3)]
    public string? Device { get; set; }

    /// <summary>The <see cref="DeviceWelcomeMessage.ResumeId"/> this device was welcomed with before, or null.</summary>
    [Key(4)]
    public string? ResumeId { get; set; }

    /// <summary>The <see cref="DeviceWelcomeMessage.Proof"/> that goes with <see cref="ResumeId"/>, or null. Never logged.</summary>
    [Key(5)]
    public byte[]? Proof { get; set; }

    /// <summary>
    /// A phone's re-entry code (<see cref="ReentryCodeMessage"/>), brought by the home-screen app it was minted for on its
    /// first launch, to take the seat's phone over. A bearer credential: never logged or described. Null otherwise.
    /// </summary>
    [Key(6)]
    public string? Reentry { get; set; }

    public override string Describe() =>
        $"hello v{InPersonVersion}/{ProtocolVersion}{(Ticket is null ? "" : ", ticket")}{(Device is null ? "" : ", device")}{(ResumeId is null ? "" : ", resume")}{(Reentry is null ? "" : ", re-entry")}";
}

/// <summary>
/// A tablet's tap on its board: an object by its public handle, or a player. The relay hands it to the seat with the
/// earliest outstanding request, in that seat's own handle space (<see cref="SeatTapMessage"/>).
/// </summary>
[MessagePackObject]
public sealed class TableTapMessage : DeviceClientMessage
{
    /// <summary>How far through the public log the tablet had been brought when it was tapped.</summary>
    [Key(0)]
    public long Sequence { get; set; }

    /// <summary>The object tapped, in the public handle space; <see cref="ViewHandle.None"/> for a tapped player.</summary>
    [Key(1)]
    public ViewHandle Object { get; set; }

    /// <summary>The player tapped, by seat; zero for a tapped object.</summary>
    [Key(2)]
    public int Player { get; set; }

    public override string Describe() => Object.IsNone ? $"table tap, player {Player}" : $"table tap, {Object}";
}

/// <summary>A phone's report of what it made of a <see cref="SeatTapMessage"/>: whether its prompt took the tap.</summary>
[MessagePackObject]
public sealed class TapResultMessage : DeviceClientMessage
{
    [Key(0)]
    public int TapId { get; set; }

    /// <summary>Whether the tap answered or advanced the phone's prompt; false hands it to the next request.</summary>
    [Key(1)]
    public bool Used { get; set; }

    public override string Describe() => $"tap {TapId} {(Used ? "used" : "unused")}";
}

/// <summary>
/// A welcomed phone asks for a re-entry code (<see cref="ReentryCodeMessage"/>), for the home-screen app it is about to be
/// added as: an app with storage of its own, which knows nothing of the browser tab's ticket or welcome.
/// </summary>
[MessagePackObject]
public sealed class ReentryRequestMessage : DeviceClientMessage
{
    public override string Describe() => "re-entry asked for";
}

/// <summary>
/// A pick made on a tablet's grid during a grid draft: a row (0 to 2, from the top) or a column (3 to 5, from the left),
/// for the pick <see cref="Decision"/> names. <see cref="Take"/> false only chooses it - the first tap, which lights the
/// line - and true takes it, the second. The relay hands it to the phone of the seat whose pick it is
/// (<see cref="SeatLineMessage"/>), which sends the pick to the table as its own; the table judges it.
/// </summary>
[MessagePackObject]
public sealed class TableLineMessage : DeviceClientMessage
{
    /// <summary>The pick of the draft the line is for (<see cref="PublicGridMessage.Decision"/>), so a stale tap is let go.</summary>
    [Key(0)]
    public int Decision { get; set; }

    [Key(1)]
    public int Line { get; set; }

    [Key(2)]
    public bool Take { get; set; }

    public override string Describe() => $"table line {Line} for pick {Decision}{(Take ? ", take" : ", choose")}";
}

/// <summary>
/// Anything the relay sends a device. A MessagePack union with explicit keys, as <see cref="ServerMessage"/> is.
/// </summary>
[Union(0, typeof(DeviceWelcomeMessage))]
[Union(1, typeof(PublicFeedMessage))]
[Union(2, typeof(SeatTapMessage))]
[Union(3, typeof(DevicePhaseMessage))]
[Union(4, typeof(DeviceErrorMessage))]
[Union(5, typeof(PublicGridMessage))]
[Union(6, typeof(SeatLineMessage))]
[Union(8, typeof(ReentryCodeMessage))]
[Union(9, typeof(SeatPressMessage))]
[Union(10, typeof(TableStopsMessage))]
[Union(11, typeof(TablePromptMessage))]
public abstract class DeviceServerMessage
{
    public abstract string Describe();
}

/// <summary>Sent once a device's hello is accepted: what it is, and how it takes its place back after a reload.</summary>
[MessagePackObject]
public sealed class DeviceWelcomeMessage : DeviceServerMessage
{
    [Key(0)]
    public DeviceRole Role { get; set; }

    /// <summary>A phone's seat, from one; zero for a tablet.</summary>
    [Key(1)]
    public int Seat { get; set; }

    /// <summary>Which board a tablet shows: zero for the shared board of both halves, or the seat whose half it is.</summary>
    [Key(2)]
    public int Side { get; set; }

    /// <summary>What the device sends back in a later hello (<see cref="DeviceHelloMessage.ResumeId"/>).</summary>
    [Key(3)]
    public string ResumeId { get; set; } = string.Empty;

    /// <summary>The secret that goes with <see cref="ResumeId"/>, kept for the tab alone. Never logged or described.</summary>
    [Key(4)]
    public byte[] Proof { get; set; } = [];

    /// <summary>The players' names, by seat from one.</summary>
    [Key(5)]
    public List<string> PlayerNames { get; set; } = [];

    /// <summary>
    /// For a phone welcomed by its re-entry code, a fresh seat ticket for the table's own connection, which the home-screen
    /// app keeps in its storage; null for every other welcome. A bearer credential: never logged or described.
    /// </summary>
    [Key(6)]
    public string? Ticket { get; set; }

    public override string Describe() => $"device welcome, {Role}, seat {Seat}, side {Side}";
}

/// <summary>
/// One message of the public feed, for a tablet: only a <see cref="WelcomeMessage"/> for seat zero, a
/// <see cref="StateMessage"/>, an <see cref="EventsMessage"/> or a <see cref="GameOverMessage"/>, all in the public
/// handle space. Never a prompt, a request id or a seat's handle.
/// </summary>
[MessagePackObject]
public sealed class PublicFeedMessage : DeviceServerMessage
{
    [Key(0)]
    public ServerMessage Message { get; set; } = new WelcomeMessage();

    public override string Describe() => $"public feed: {Message.Describe()}";
}

/// <summary>
/// A tap made on a tablet, for the phone whose request it was routed to, in that seat's own handle space. The phone acts
/// on it only while <see cref="RequestId"/> is its prompt, and answers with a <see cref="TapResultMessage"/>.
/// </summary>
[MessagePackObject]
public sealed class SeatTapMessage : DeviceServerMessage
{
    [Key(0)]
    public int TapId { get; set; }

    /// <summary>The outstanding request the tap was routed to.</summary>
    [Key(1)]
    public int RequestId { get; set; }

    /// <summary>The object tapped, in the seat's handle space; <see cref="ViewHandle.None"/> for a tapped player.</summary>
    [Key(2)]
    public ViewHandle Object { get; set; }

    /// <summary>The player tapped, by seat; zero for a tapped object.</summary>
    [Key(3)]
    public int Player { get; set; }

    public override string Describe() =>
        Object.IsNone ? $"seat tap {TapId} for {RequestId}, player {Player}" : $"seat tap {TapId} for {RequestId}, {Object}";
}

/// <summary>
/// Where the event stands before its game - drafting, building decks - for a tablet to show until the first public state.
/// </summary>
[MessagePackObject]
public sealed class DevicePhaseMessage : DeviceServerMessage
{
    [Key(0)]
    public TablePhase Phase { get; set; }

    /// <summary>One line saying where the event stands: the grid being drafted, whose pick it is, who has submitted.</summary>
    [Key(1)]
    public string Line { get; set; } = string.Empty;

    /// <summary>The devices connected now, by name.</summary>
    [Key(2)]
    public List<string> Connected { get; set; } = [];

    public override string Describe() => $"phase {Phase}: {Line}";
}

/// <summary>Something a device sent could not be accepted: a hello refused, or a message out of place.</summary>
[MessagePackObject]
public sealed class DeviceErrorMessage : DeviceServerMessage
{
    [Key(0)]
    public ErrorCode Code { get; set; }

    [Key(1)]
    public string Detail { get; set; } = string.Empty;

    public override string Describe() => $"device error {Code}: {Detail}";
}

/// <summary>
/// A phone's re-entry code, for the home-screen app it is being added as (docs/in-person.md, "Full screen"): good once,
/// for the seat's phone, until it is used, a newer one is asked for, the seat's ticket takes the phone's place anew or the
/// table closes. A bearer credential: never logged or described.
/// </summary>
[MessagePackObject]
public sealed class ReentryCodeMessage : DeviceServerMessage
{
    [Key(0)]
    public string Code { get; set; } = string.Empty;

    public override string Describe() => "re-entry code";
}

/// <summary>
/// A grid draft as both players see it (docs/events.md, "A grid draft"), for a tablet: the grid face up, a place whose card
/// has been taken an unknown card; which grid and which pick of it; whose pick it is; how many cards each player has
/// drafted; and the last picks in words, each naming the cards taken. Exactly what the remote client shows both seats
/// of a grid draft, less each seat's own pool: nothing private. Sent whenever the draft moves on, and once with
/// <see cref="Grid"/> zero and no places when the draft is over.
/// </summary>
[MessagePackObject]
public sealed class PublicGridMessage : DeviceServerMessage
{
    /// <summary>The pick being made, counted from zero over the whole draft: what a <see cref="TableLineMessage"/> names.</summary>
    [Key(0)]
    public int Decision { get; set; }

    /// <summary>The grid being picked from, from 1; zero once the draft is over.</summary>
    [Key(1)]
    public int Grid { get; set; }

    /// <summary>How many grids the draft lays out.</summary>
    [Key(2)]
    public int Grids { get; set; }

    /// <summary>1 for the first pick from this grid, 2 for the second, from what is left; zero once the draft is over.</summary>
    [Key(3)]
    public int Pick { get; set; }

    /// <summary>The seat whose pick it is; zero once the draft is over.</summary>
    [Key(4)]
    public int Active { get; set; }

    /// <summary>The grid's nine places, row by row from the top left, each as printed; a place taken is an unknown card.</summary>
    [Key(5)]
    public List<ObjectView> Places { get; set; } = [];

    /// <summary>How many cards each seat has drafted, seat 1's first: public, as at a real table.</summary>
    [Key(6)]
    public List<int> Drafted { get; set; } = [];

    /// <summary>The last picks in words, newest last, naming each player: "Ann took row 2 of grid 3: [A], [B] and [C]."</summary>
    [Key(7)]
    public List<string> Recent { get; set; } = [];

    /// <summary>The cards each line of <see cref="Recent"/> names, in the order its brackets name them.</summary>
    [Key(8)]
    public List<List<LogReferenceView>> RecentCards { get; set; } = [];

    /// <summary>The line the last pick took, from the grid as it was then; -1 before any.</summary>
    [Key(9)]
    public int LastLine { get; set; } = -1;

    /// <summary>How long the pick has left, in milliseconds, when this was sent; zero with no clock.</summary>
    [Key(10)]
    public int PickLeftMs { get; set; }

    public override string Describe() => Grid == 0 ? "public grid: the draft is over" : $"public grid {Grid}/{Grids}, pick {Decision}, seat {Active}";
}

/// <summary>
/// A line picked on a tablet, for the phone of the seat whose pick it is: the phone chooses or takes it as a tap on the
/// remote client's grid would (<c>GameClient.ChooseLine</c>, <c>GameClient.PickLine</c>), if the pick is still its own.
/// </summary>
[MessagePackObject]
public sealed class SeatLineMessage : DeviceServerMessage
{
    [Key(0)]
    public int Decision { get; set; }

    [Key(1)]
    public int Line { get; set; }

    [Key(2)]
    public bool Take { get; set; }

    public override string Describe() => $"seat line {Line} for pick {Decision}{(Take ? ", take" : ", choose")}";
}

/// <summary>
/// A phone's prompt as the table may see it (docs/in-person.md, "The prompt on the table"): what is asked and the buttons that
/// answer it, sent whenever it changes, for the tablets to show on the player's side of the board and answer from there.
/// At a table in paper what a player is doing is in plain view - a spell revealed as it is cast, a target pointed at, a
/// stop called - so the prompt is public; only a choice among cards nobody else may see (a library searched, a scry, a
/// card named) is not, and is sent as <see cref="PromptCard.Private"/> with nothing of it in the card.
/// </summary>
[MessagePackObject]
public sealed class SeatPromptMessage : DeviceClientMessage
{
    [Key(0)]
    public PromptCard Prompt { get; set; } = new();

    public override string Describe() => $"seat prompt {Prompt.Version}{(Prompt.Private ? ", private" : "")}, {Prompt.Rows.Sum(row => row.Items.Count)} items";
}

/// <summary>A seat's prompt, as its phone last said it, for a tablet to show on that seat's side and answer from.</summary>
[MessagePackObject]
public sealed class TablePromptMessage : DeviceServerMessage
{
    [Key(0)]
    public int Seat { get; set; }

    [Key(1)]
    public PromptCard Prompt { get; set; } = new();

    public override string Describe() => $"table prompt for seat {Seat}, {Prompt.Version}";
}

/// <summary>
/// A prompt's words and its buttons, as a phone's dock shows them: a title and a detail line (written with mana symbols,
/// "{2}{R}"), an error the table gave, and rows of items - the dock's toggles, then its buttons, a stepper's "-", value and
/// "+" in a row of their own. <see cref="Version"/> grows with every change, so a press names the prompt it was made on.
/// </summary>
[MessagePackObject]
public sealed class PromptCard
{
    [Key(0)]
    public int Version { get; set; }

    /// <summary>A choice among cards only this player may see: nothing else of the prompt is sent.</summary>
    [Key(1)]
    public bool Private { get; set; }

    [Key(2)]
    public string Title { get; set; } = string.Empty;

    [Key(3)]
    public string Detail { get; set; } = string.Empty;

    [Key(4)]
    public string Error { get; set; } = string.Empty;

    [Key(5)]
    public List<PromptRow> Rows { get; set; } = [];

    /// <summary>
    /// Whether the table is waiting on this player: their phone has a question of the game's to answer. The tablets show
    /// the prompt of the player being asked, turned to face them.
    /// </summary>
    [Key(6)]
    public bool Asking { get; set; }
}

/// <summary>One line of a prompt's items.</summary>
[MessagePackObject]
public sealed class PromptRow
{
    [Key(0)]
    public List<PromptItem> Items { get; set; } = [];
}

/// <summary>A button, a toggle or a word of a prompt; a button's or a toggle's <see cref="Id"/> is what a press names.</summary>
[MessagePackObject]
public sealed class PromptItem
{
    /// <summary>The item's place in its prompt, counted across every row; what <see cref="TablePressMessage.Item"/> names.</summary>
    [Key(0)]
    public int Id { get; set; }

    [Key(1)]
    public PromptItemKind Kind { get; set; }

    [Key(2)]
    public string Text { get; set; } = string.Empty;

    /// <summary>The answer the prompt expects, drawn in the accent.</summary>
    [Key(3)]
    public bool Primary { get; set; }

    [Key(4)]
    public bool Enabled { get; set; } = true;

    /// <summary>A toggle that is on, or a button held down.</summary>
    [Key(5)]
    public bool On { get; set; }
}

/// <summary>What a <see cref="PromptItem"/> is.</summary>
public enum PromptItemKind
{
    Button = 0,
    Toggle = 1,
    Words = 2,
}

/// <summary>
/// A button or a toggle of a seat's prompt pressed on a tablet: the relay hands it to that seat's phone
/// (<see cref="SeatPressMessage"/>) if <see cref="Version"/> is still the prompt the phone last sent, and the phone presses
/// that item of its own dock, as a finger on it would; the table judges the answer. A player's own board presses only for
/// that player.
/// </summary>
[MessagePackObject]
public sealed class TablePressMessage : DeviceClientMessage
{
    [Key(0)]
    public int Seat { get; set; }

    [Key(1)]
    public int Version { get; set; }

    [Key(2)]
    public int Item { get; set; }

    public override string Describe() => $"table press, seat {Seat}, item {Item} of prompt {Version}";
}

/// <summary>An item of this phone's prompt pressed on a tablet, for the prompt <see cref="Version"/> names.</summary>
[MessagePackObject]
public sealed class SeatPressMessage : DeviceServerMessage
{
    [Key(0)]
    public int Version { get; set; }

    [Key(1)]
    public int Item { get; set; }

    public override string Describe() => $"seat press, item {Item} of prompt {Version}";
}

/// <summary>
/// The steps a phone's player has set stops in (docs/in-person.md, "Stops"): on their own turn and on the opponent's, as
/// <see cref="TurnStepView"/> values. Sent on the phone's welcome and whenever they change; the relay tells every device
/// (<see cref="TableStopsMessage"/>), since at an in-person table a stop is the table's: every player stops at it.
/// </summary>
[MessagePackObject]
public sealed class SeatStopsMessage : DeviceClientMessage
{
    [Key(0)]
    public List<TurnStepView> Mine { get; set; } = [];

    [Key(1)]
    public List<TurnStepView> Theirs { get; set; } = [];

    public override string Describe() => $"stops: {Mine.Count} on own turn, {Theirs.Count} on the opponent's";
}

/// <summary>One seat's stops, as its phone last said them.</summary>
[MessagePackObject]
public sealed class SeatStops
{
    [Key(0)]
    public int Seat { get; set; }

    /// <summary>The steps of this seat's own turn it stops in.</summary>
    [Key(1)]
    public List<TurnStepView> Mine { get; set; } = [];

    /// <summary>The steps of an opponent's turn it stops in.</summary>
    [Key(2)]
    public List<TurnStepView> Theirs { get; set; } = [];
}

/// <summary>Every seat's stops, as their phones last said them: for the phones to stop where anyone does, and the tablets to show.</summary>
[MessagePackObject]
public sealed class TableStopsMessage : DeviceServerMessage
{
    [Key(0)]
    public List<SeatStops> Seats { get; set; } = [];

    public override string Describe() => $"table stops for {Seats.Count} seats";
}
