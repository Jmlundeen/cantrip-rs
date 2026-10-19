using MessagePack;

namespace Cantrip.Protocol;

/// <summary>Private bearer credentials. Store only on the owning client; never log or share the proof.</summary>
[MessagePackObject]
public sealed class MembershipMessage : ServerMessage
{
    [Key(0)] public string MembershipId { get; set; } = string.Empty;
    [Key(1)] public string ReconnectProof { get; set; } = string.Empty;
    [Key(2)] public int Seat { get; set; }
    public override string Describe() => $"membership, seat {Seat}";
}

[MessagePackObject]
public sealed class ResumeMessage : ClientMessage
{
    [Key(0)] public int ProtocolVersion { get; set; } = PlayerView.ProtocolVersion;
    [Key(1)] public string MembershipId { get; set; } = string.Empty;
    [Key(2)] public string ReconnectProof { get; set; } = string.Empty;
    public override string Describe() => "resume membership";
}

/// <summary>
/// Where a table is. Setup is the lobby, or for an event's table the players taking their seats; an event's table then
/// drafts and builds decks before it plays. Pairings is an event's table between the rounds its players play at other
/// tables, and its final standings. New phases go on the end, so the numbers already sent keep their meaning.
/// </summary>
public enum TablePhase { Setup, Playing, Finished, Abandoned, Drafting, DeckBuilding, Pairings }

[MessagePackObject]
public sealed class TablePlayerView
{
    [Key(0)] public int Seat { get; set; }
    [Key(1)] public string Name { get; set; } = string.Empty;
    [Key(2)] public string Deck { get; set; } = string.Empty;
    [Key(3)] public bool Ready { get; set; }
    [Key(4)] public bool Connected { get; set; }

    /// <summary>
    /// The player's picture, as a URL relative to the client's page (or absolute); empty for none, when the client draws
    /// its own stand-in.
    /// </summary>
    [Key(5)] public string Portrait { get; set; } = string.Empty;

    /// <summary>
    /// Seated by a ticket, so the name (a Discord player's own) and the deck were set when the seat was reserved and
    /// cannot be changed from the table.
    /// </summary>
    [Key(6)] public bool Fixed { get; set; }
}

/// <summary>Public lobby state. Never contains deck contents, credentials, hands or pending choices.</summary>
[MessagePackObject]
public sealed class TableMessage : ServerMessage
{
    [Key(0)] public TablePhase Phase { get; set; }
    [Key(1)] public int Capacity { get; set; }
    [Key(2)] public List<TablePlayerView> Players { get; set; } = [];
    [Key(3)] public List<string> Decks { get; set; } = [];
    [Key(4)] public List<string> AvailableSets { get; set; } = [];
    [Key(5)] public List<string> SelectedSets { get; set; } = [];
    [Key(6)] public string Detail { get; set; } = string.Empty;
    [Key(7)] public int HostSeat { get; set; }

    /// <summary>The event this table belongs to, or empty for a table set up in its own lobby.</summary>
    [Key(8)] public string EventId { get; set; } = string.Empty;

    /// <summary>The event's name, for showing; empty with no event.</summary>
    [Key(9)] public string EventName { get; set; } = string.Empty;

    /// <summary>
    /// Whether the table is still being formed from a queue: seats are taken and given up, and when every seat is
    /// taken each player has to say they are ready (<see cref="ReadyMessage"/>) before the event begins. An empty
    /// seat is a player with no name.
    /// </summary>
    [Key(10)] public bool Forming { get; set; }

    /// <summary>How long is left to say ready, in milliseconds, while a forming table waits on it; zero otherwise.</summary>
    [Key(11)] public int ReadyLeftMs { get; set; }

    /// <summary>
    /// Where card art is for this table, as the web client's <c>?art=</c> takes it: an absolute http(s) base URL, or a
    /// URL template with <c>{size}</c> and <c>{slug}</c> in it. Empty when the table was not told. The table says so rather than the
    /// client working it out from the table's address, so a player reaches the art from whatever table they reach -
    /// one they were moved to, or one opened again on another computer.
    /// </summary>
    [Key(12)] public string Art { get; set; } = string.Empty;

    public override string Describe() => $"table {Phase}, {Players.Count}/{Capacity} players";
}

/// <summary>
/// Every player's match clock, as a chess clock: how long each has left for the whole match, and whose is running - the
/// players the game is waiting on, from the mulligans on. Sent whenever whose clock runs changes, and on taking a seat;
/// the client counts a running clock down itself in between. A player whose clock reaches zero loses the match.
/// </summary>
[MessagePackObject]
public sealed class ClockMessage : ServerMessage
{
    [Key(0)] public List<SeatClockView> Clocks { get; set; } = [];
    public override string Describe() => $"clocks: {string.Join(", ", Clocks.Select(clock => $"seat {clock.Seat} {clock.LeftMs / 1000}s{(clock.Running ? " running" : string.Empty)}"))}";
}

[MessagePackObject]
public sealed class SeatClockView
{
    [Key(0)] public int Seat { get; set; }

    /// <summary>How long the seat has left, in milliseconds, as of when the message was sent.</summary>
    [Key(1)] public long LeftMs { get; set; }

    /// <summary>Whether it is ticking: the game is waiting on this seat.</summary>
    [Key(2)] public bool Running { get; set; }
}

[MessagePackObject]
public sealed class ConfigurePlayerMessage : ClientMessage
{
    [Key(0)] public string Name { get; set; } = string.Empty;
    [Key(1)] public string Deck { get; set; } = string.Empty;
    public override string Describe() => "configure player";
}

[MessagePackObject]
public sealed class ConfigureTableMessage : ClientMessage
{
    [Key(0)] public List<string> Sets { get; set; } = [];
    public override string Describe() => "configure sets";
}

[MessagePackObject]
public sealed class ReadyMessage : ClientMessage
{
    [Key(0)] public bool Ready { get; set; }
    public override string Describe() => Ready ? "ready" : "not ready";
}

[MessagePackObject]
public sealed class LeaveMessage : ClientMessage
{
    public override string Describe() => "leave table";
}

/// <summary>
/// This seat concedes the game being played (CR 104.3a): it loses it, whatever it was being asked, and the game ends
/// for whoever is left. In an event's match it is the game that is conceded, not the match.
/// </summary>
[MessagePackObject]
public sealed class ConcedeMessage : ClientMessage
{
    public override string Describe() => "concede";
}

/// <summary>
/// This seat drops from its event: it is paired no more, and a match of its not yet begun goes to its opponent - or,
/// before its round begins, the round is paired again without it.
/// </summary>
[MessagePackObject]
public sealed class DropMessage : ClientMessage
{
    public override string Describe() => "drop";
}
