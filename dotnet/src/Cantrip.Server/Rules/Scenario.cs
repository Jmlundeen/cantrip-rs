using Cantrip.Protocol;

namespace Cantrip.Server.Rules;

/// <summary>
/// A <c>.cantrip</c> scenario to play at a table instead of a dealt game (docs/scenarios-in-the-browser.md): its decks,
/// order and dice are the scenario's, the <paramref name="Driven"/> seats are answered by the table's clients, and every
/// other seat by the script.
/// </summary>
/// <param name="Name">The scenario's name, for its reports.</param>
/// <param name="Text">The scenario as written.</param>
/// <param name="Driven">The seats the table's clients answer for, from one.</param>
public sealed record ScenarioSpec(string Name, string Text, IReadOnlyList<int> Driven)
{
    /// <summary>The scenario in <paramref name="path"/>, named for its file.</summary>
    public static ScenarioSpec Load(string path, IReadOnlyList<int> driven) =>
        new(Path.GetFileNameWithoutExtension(path), File.ReadAllText(path), driven);
}

/// <summary>What a driven seat is asked now, and the answer the script means it to give.</summary>
/// <param name="Answer">The answer, in the seat's handles; null when it is to take back a mana ability (<paramref name="Undo"/>).</param>
/// <param name="Line">The line it comes from, <c>14: Bot1.Hand.Island.Play</c>, or null for a default or a payment in progress.</param>
public sealed record ScenarioExpected(int Seat, int RequestId, AnswerPayload? Answer, bool Undo, string? Line);

/// <summary>A line about what a client shows (<c>Client.Bot1.Stack = Black Lotus</c>), for the driving pages to judge.</summary>
public sealed record ScenarioCheck(int Line, string Key, string Value);

/// <summary>How a scenario ended: passed, with the client lines of <c>[Expect]</c> still to judge, or failed with the runner's report.</summary>
public sealed record ScenarioVerdict(bool Passed, string? Report, IReadOnlyList<ScenarioCheck> Client);

/// <summary>A scenario's part of one step: the question to a driven seat, the client checks reached, and the verdict once over.</summary>
public sealed record ScenarioProgress(ScenarioExpected? Expected, IReadOnlyList<ScenarioCheck> Checks, ScenarioVerdict? Verdict);
