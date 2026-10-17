using MessagePack;

namespace Cantrip.Protocol;

/// <summary>
/// The game log, as this seat reads it: new lines as they are written, and on a resync the whole of it.
/// <para>
/// Written by the server, not by the client, because the log has to be complete whatever happens to the client. A
/// client that lost power while its opponent was acting, or that joins a game already under way, is sent every line
/// it is entitled to when it comes back, worded as it would have read them at the time. Nothing about the log depends
/// on the client having been there to see the events arrive.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class LogMessage : ServerMessage
{
    /// <summary>New lines, oldest first, to add after the ones the client has.</summary>
    [Key(0)]
    public List<LogEntryView> Entries { get; set; } = [];

    /// <summary>
    /// Lines taken back, by <see cref="LogEntryView.Number"/>: what they reported was undone, as a mana ability taken
    /// back is, and the log reads as though it never happened.
    /// </summary>
    [Key(1)]
    public List<long> Removed { get; set; } = [];

    /// <summary>
    /// Whether <see cref="Entries"/> is the whole log, replacing whatever the client had: sent when a seat is joined or
    /// resumed, so a client that missed anything is brought back to the complete log rather than patched.
    /// </summary>
    [Key(2)]
    public bool Complete { get; set; }

    public override string Describe() =>
        Complete ? $"log: all {Entries.Count} line(s)" : $"log: {Entries.Count} new, {Removed.Count} removed";
}

/// <summary>What a line of the log is about, where a client does something with the line besides show it.</summary>
public enum LogEntryKind
{
    Event = 0,

    /// <summary>A player lost the game, and why: what the game-over panel says as well as the log.</summary>
    PlayerLost = 1,

    /// <summary>The game was drawn, and why (CR 104.4): what the game-over panel says as well as the log.</summary>
    GameDrawn = 2,
}

/// <summary>One line of the game log, as one seat reads it.</summary>
[MessagePackObject]
public sealed class LogEntryView
{
    /// <summary>The line's own number: unique in the game, and what a retraction names it by.</summary>
    [Key(0)]
    public long Number { get; set; }

    /// <summary>
    /// When it happened: the sequence of the game event it reports, in the server's one log of everything that has
    /// happened. Lines are in this order.
    /// </summary>
    [Key(1)]
    public long Sequence { get; set; }

    /// <summary>The turn it happened in; zero or less before the first turn.</summary>
    [Key(2)]
    public int Turn { get; set; }

    /// <summary>Whose turn that was, by seat, for the turn's heading.</summary>
    [Key(3)]
    public int ActivePlayer { get; set; }

    /// <summary>The step it happened in, for the step's heading; null before the first step.</summary>
    [Key(4)]
    public TurnStepView? Step { get; set; }

    /// <summary>
    /// The line, in oracle notation: symbols in braces, drawn as pips, and each object it names written as its name
    /// in square brackets, in the order of <see cref="References"/>.
    /// </summary>
    [Key(5)]
    public string Text { get; set; } = string.Empty;

    /// <summary>The objects the line names, in the order they appear in it.</summary>
    [Key(6)]
    public List<LogReferenceView> References { get; set; } = [];

    [Key(7)]
    public LogEntryKind Kind { get; set; }

    public override string ToString() => Text;
}

/// <summary>
/// An object a log line names: its name, and the handle it had in this seat's view when the line was written.
/// <para>
/// The handle is the id the client already finds the object on the table by, so while the object is still in view -
/// the target of a spell still on the stack, a permanent still out - a line can point at it. Once the object has left
/// view the handle simply matches nothing, and the line still has the object as the seat saw it then
/// (<see cref="Card"/>), to be read by.
/// </para>
/// </summary>
[MessagePackObject]
public sealed class LogReferenceView
{
    [Key(0)]
    public string Name { get; set; } = string.Empty;

    [Key(1)]
    public ViewHandle Object { get; set; }

    /// <summary>
    /// The object as this seat's view had it when the line was written, so a card named in the log can be read after it
    /// has gone: a creature that died, a spell that resolved. Never more than the seat was shown then; null when there
    /// is nothing to read but the name.
    /// </summary>
    [Key(2)]
    public ObjectView? Card { get; set; }
}
