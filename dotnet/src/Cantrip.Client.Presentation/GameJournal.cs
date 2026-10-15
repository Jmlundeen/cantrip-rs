using Cantrip.Protocol;

namespace Cantrip.Client.Presentation;

public enum LogLineKind
{
    /// <summary>Whose turn it is, and which one.</summary>
    Turn = 0,

    /// <summary>A step within that turn. Only written once something has actually happened in it.</summary>
    Phase = 1,

    Entry = 2,
}

/// <param name="Kind">What sort of line it is.</param>
/// <param name="Text">The line, in oracle notation: symbols are written in braces and drawn as pips.</param>
public sealed record LogLine(LogLineKind Kind, string Text)
{
    /// <summary>The number of the server's log entry an entry line shows, or -1 for a heading.</summary>
    public long Sequence { get; init; } = -1;

    /// <summary>The turn a header line heads, or that an entry belongs to.</summary>
    public int Turn { get; init; }

    /// <summary>The step a phase header heads.</summary>
    public TurnStepView? Step { get; init; }

    /// <summary>The server's entry an entry line shows, with the objects it names; null for a heading.</summary>
    public LogEntryView? Entry { get; init; }

    /// <summary>
    /// How many entries in a row, word for word the same, this line stands for: a spell that makes two hundred tokens
    /// is one line said two hundred times. The line keeps the first of them's <see cref="Sequence"/> and <see cref="Entry"/>.
    /// </summary>
    public int Count { get; init; } = 1;
}

/// <summary>
/// The game log laid out for reading: the server's lines (<see cref="GameClient.Log"/>) under a heading for each turn
/// and each step something happened in.
/// <para>
/// The lines are the server's, written for this seat as the events happened and kept by it for the whole game, so the
/// log is complete however this client came to be here - from the start, after a reload, or after losing its
/// connection while the game went on. Nothing here words an event; it only sets the lines out.
/// </para>
/// <para>
/// Turn and step headers are held back until something happens under them. A step in which both players did nothing
/// but pass priority has no lines at all, so the step that would head an empty section never gets a heading - which
/// is what keeps the log to the things somebody did.
/// </para>
/// </summary>
public sealed class GameJournal
{
    private readonly List<LogLine> _lines = [];

    private GameClient? _client;
    private int _clientRevision = -1;
    private int _laidOut;
    private int _writtenTurn;
    private TurnStepView? _writtenStep;

    /// <summary>Every line so far, oldest first.</summary>
    public IReadOnlyList<LogLine> Lines => _lines;

    /// <summary>
    /// Bumped whenever lines are removed or replaced rather than added, so a view that appends new lines knows to start
    /// over.
    /// </summary>
    public int Revision { get; private set; }

    /// <summary>Brings the lines up to the client's log: the new entries appended, or all of it laid out again if it changed.</summary>
    public void Follow(GameClient client)
    {
        ArgumentNullException.ThrowIfNull(client);

        if (!ReferenceEquals(client, _client) || client.LogRevision != _clientRevision)
        {
            _client = client;
            _clientRevision = client.LogRevision;

            if (_lines.Count > 0)
            {
                Revision++;
            }

            _lines.Clear();
            _laidOut = 0;
            _writtenTurn = 0;
            _writtenStep = null;
        }

        for (; _laidOut < client.Log.Count; _laidOut++)
        {
            Lay(client, client.Log[_laidOut]);
        }
    }

    private void Lay(GameClient client, LogEntryView entry)
    {
        // Lines from before the first turn - the opening hands - are under no heading.
        if (entry.Turn != _writtenTurn)
        {
            _lines.Add(new LogLine(LogLineKind.Turn, $"{NameOf(client, entry.ActivePlayer)} Turn {entry.Turn}") { Turn = entry.Turn });
            _writtenTurn = entry.Turn;

            // A new turn means the step header has to be written again even if the step name has not changed.
            _writtenStep = null;
        }

        if (entry.Step != _writtenStep)
        {
            _lines.Add(new LogLine(LogLineKind.Phase, $"{Caption(entry.Step)}:") { Turn = entry.Turn, Step = entry.Step });
            _writtenStep = entry.Step;
        }

        // The same words again straight after, under the same heading, count on the line already there. The line is
        // replaced rather than changed, so a view holding the old one can tell it is not the same.
        if (_lines.Count > 0 && _lines[^1] is { Kind: LogLineKind.Entry } last && last.Text == entry.Text)
        {
            _lines[^1] = last with { Count = last.Count + 1 };
            return;
        }

        _lines.Add(new LogLine(LogLineKind.Entry, entry.Text) { Sequence = entry.Number, Turn = entry.Turn, Entry = entry });
    }

    /// <summary>Step names as they are said out loud rather than as they are spelled in the enum.</summary>
    private static string Caption(TurnStepView? step) => step switch
    {
        TurnStepView.Untap => "Untap",
        TurnStepView.Upkeep => "Upkeep",
        TurnStepView.Draw => "Draw",
        TurnStepView.PrecombatMain => "Precombat Main",
        TurnStepView.BeginCombat => "Beginning of Combat",
        TurnStepView.DeclareAttackers => "Declare Attackers",
        TurnStepView.DeclareBlockers => "Declare Blockers",
        TurnStepView.CombatDamage => "Combat Damage",
        TurnStepView.EndOfCombat => "End of Combat",
        TurnStepView.PostcombatMain => "Postcombat Main",
        TurnStepView.End => "End Step",
        TurnStepView.Cleanup => "Cleanup",
        _ => "Game",
    };

    private static string NameOf(GameClient client, int seat)
    {
        foreach (PlayerSummaryView player in client.View?.Players ?? [])
        {
            if (player.Seat == seat)
            {
                return player.Name;
            }
        }

        return seat > 0 && seat <= client.PlayerNames.Count ? client.PlayerNames[seat - 1] : $"seat {seat}";
    }
}
