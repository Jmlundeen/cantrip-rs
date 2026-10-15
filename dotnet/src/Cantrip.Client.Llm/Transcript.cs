using System.Globalization;

namespace Cantrip.Client.Llm;

/// <summary>
/// Every question this seat was put and what it answered, written to a file as the game is played.
/// <para>
/// A game played in the client shows the model's moves and none of its reasons, which makes a bad move impossible
/// to account for: a move that looks like poor judgement and a move that was the only sensible answer to a badly
/// worded question read exactly the same from across the table. The question is the thing worth keeping, because
/// nearly every fault this seat can have shows up in it — a choice that was not offered, two choices that say the
/// same thing, a board rendered wrong.
/// </para>
/// <para>
/// Written as it happens rather than kept and flushed at the end, because the game this is wanted for is the one
/// that did something strange and was then closed.
/// </para>
/// </summary>
public sealed class Transcript : IDisposable
{
    /// <summary>
    /// Written to under a lock, because in the client the questions come from the thread the model is answered on
    /// and the game's own lines come from the frame loop.
    /// </summary>
    private readonly object _gate = new();

    private readonly StreamWriter _file;
    private bool _disposed;

    /// <param name="path">The file to write. Its directory is created if it is not there.</param>
    public Transcript(string path)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);

        if (Path.GetDirectoryName(path) is { Length: > 0 } directory)
        {
            Directory.CreateDirectory(directory);
        }

        _file = new StreamWriter(path, append: true) { AutoFlush = true };
        Where = path;

        Write($"--- a game began at {DateTime.Now.ToString("u", CultureInfo.InvariantCulture)} ---");
    }

    /// <summary>The file being written, so a run can say where to look.</summary>
    public string Where { get; }

    /// <summary>One question and the answer it got.</summary>
    public void Decided(string question, string answer) => Write($"{question}\n\n  => {answer}\n");

    /// <summary>Anything else worth a line: a fallback, a refusal, the end of the game.</summary>
    public void Note(string what) => Write($"[{what}]");

    private void Write(string text)
    {
        lock (_gate)
        {
            if (_disposed)
            {
                return;
            }

            _file.WriteLine(text);
        }
    }

    public void Dispose()
    {
        lock (_gate)
        {
            if (_disposed)
            {
                return;
            }

            _disposed = true;
            _file.Dispose();
        }
    }
}
