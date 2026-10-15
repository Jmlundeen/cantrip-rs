namespace Cantrip.Client.Llm;

/// <summary>Who said something in a conversation with a language model.</summary>
public enum ChatRole
{
    /// <summary>The standing instructions: who this seat is, what its deck is, how to answer.</summary>
    System = 0,

    /// <summary>A question put to the model.</summary>
    User = 1,

    /// <summary>What the model said back.</summary>
    Assistant = 2,
}

/// <param name="Role">Who is speaking.</param>
/// <param name="Text">What they said.</param>
public sealed record ChatTurn(ChatRole Role, string Text);

/// <summary>
/// Something that answers a conversation with one more turn.
/// <para>
/// An interface rather than a class so the seat can be played without a network: <c>ScriptedModel</c> in the tests
/// answers from a list, which is what lets every prompt this seat can be asked be exercised as a test with no key,
/// no money and no wait. <see cref="OpenRouterModel"/> is the one that costs something.
/// </para>
/// <para>
/// Synchronous on purpose. <c>IAutoPlayer.Act</c> is called from a loop that expects an answer or nothing, and a
/// seat that returned "not yet" would be read as an idle seat by every harness that drives one. The cost is that a
/// seat blocks its caller for as long as the model takes, which is why this bot plays one game at a time in its own
/// tool rather than a thousand at once in <c>cantrip-playtest</c>.
/// </para>
/// </summary>
public interface ILanguageModel
{
    /// <summary>The next turn of <paramref name="conversation"/>, as the model's own words.</summary>
    string Continue(IReadOnlyList<ChatTurn> conversation);

    /// <summary>What has been spent so far, for a run to report.</summary>
    ModelUsage Usage { get; }
}

/// <summary>What a model has been asked and what it cost, accumulated over a game.</summary>
public sealed class ModelUsage
{
    /// <summary>How many times the model has been asked anything.</summary>
    public int Calls { get; private set; }

    public long InputTokens { get; private set; }

    public long OutputTokens { get; private set; }

    /// <summary>
    /// How many calls had to be made again: a reply that named no choice, or a request the provider refused with
    /// something worth retrying. Worth reporting on its own — a model that has to be asked twice for half its
    /// decisions is one to stop using, and the win rate alone would not say so.
    /// </summary>
    public int Retries { get; private set; }

    public void Record(long input, long output)
    {
        Calls++;
        InputTokens += input;
        OutputTokens += output;
    }

    public void Retried() => Retries++;

    /// <summary>What this cost, at prices given per million tokens. Zero where the prices are not known.</summary>
    public decimal Cost(decimal inputPerMillion, decimal outputPerMillion) =>
        ((InputTokens * inputPerMillion) + (OutputTokens * outputPerMillion)) / 1_000_000m;

    public override string ToString() =>
        $"{Calls} call(s), {InputTokens:N0} in / {OutputTokens:N0} out"
        + (Retries > 0 ? $", {Retries} retried" : string.Empty);
}
