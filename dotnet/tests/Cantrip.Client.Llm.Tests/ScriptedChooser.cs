using Cantrip.Client.Llm;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// A chooser that answers from a list written in the test, and remembers every question it was put.
/// <para>
/// This is what makes the seat testable at all. A language model is slow, costs money and answers differently on
/// Tuesday; none of that says anything about whether "block with the second creature" turns into a legal
/// <c>BlockersDeclaredAnswer</c>. The model is one implementation of <see cref="IChooser"/> and this is another, so
/// every prompt the server can send can be walked through here with no key and no network.
/// </para>
/// </summary>
internal sealed class ScriptedChooser : IChooser
{
    private readonly Queue<int[]> _answers;

    /// <param name="answers">One entry per question, as indexes into that question's choices.</param>
    internal ScriptedChooser(params int[][] answers) => _answers = new Queue<int[]>(answers);

    /// <summary>Everything it was asked, in order.</summary>
    internal List<Question> Asked { get; } = [];

    /// <summary>Whether the script was used up exactly, which a test asserts to catch a question it did not expect.</summary>
    internal bool Spent => _answers.Count == 0;

    public IReadOnlyList<int> Choose(Question question)
    {
        Asked.Add(question);

        if (_answers.Count == 0)
        {
            throw new InvalidOperationException(
                $"The script ran out. It was asked, unexpectedly: {question.Heading}");
        }

        return _answers.Dequeue();
    }
}

/// <summary>A chooser that always takes the first choice, for a test that only cares that the answer is legal.</summary>
internal sealed class FirstChooser : IChooser
{
    internal int Questions { get; private set; }

    public IReadOnlyList<int> Choose(Question question)
    {
        ArgumentNullException.ThrowIfNull(question);
        Questions++;
        return [0];
    }
}

/// <summary>A chooser that never answers, so the seat has to fall back to the prompt's own.</summary>
internal sealed class SilentChooser : IChooser
{
    public IReadOnlyList<int> Choose(Question question) => throw new NoChoiceException();
}

/// <summary>A model that reads its replies off a list, for testing what <see cref="ModelChooser"/> makes of them.</summary>
internal sealed class ScriptedModel : ILanguageModel
{
    private readonly Queue<string> _replies;

    internal ScriptedModel(params string[] replies) => _replies = new Queue<string>(replies);

    internal List<string> Heard { get; } = [];

    public ModelUsage Usage { get; } = new();

    public string Continue(IReadOnlyList<ChatTurn> conversation)
    {
        ArgumentNullException.ThrowIfNull(conversation);

        Heard.Add(conversation[^1].Text);
        Usage.Record(1, 1);

        return _replies.Count > 0 ? _replies.Dequeue() : "I do not know.";
    }
}
