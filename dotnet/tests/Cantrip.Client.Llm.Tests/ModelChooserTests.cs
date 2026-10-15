using Xunit;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// What a reply from a model is understood to mean.
/// <para>
/// The rule being proved is that an answer line has to <em>begin</em> with a key. A looser reader finds the key
/// <c>A</c> in the word "a" and the key <c>I</c> in the word "I", and would answer a question about six creatures
/// with whichever one a stray article landed on — a wrong move made silently, which is the worst kind. Anything
/// that does not begin with a key is not an answer, and a question that gets no answer is asked again and then
/// given up on.
/// </para>
/// </summary>
public sealed class ModelChooserTests
{
    private static Question Three => new()
    {
        Heading = "What do you do?",
        Choices = ["Cast Lightning Bolt.", "Attack.", "Pass."],
    };

    [Theory]
    [InlineData("B")]
    [InlineData("B.")]
    [InlineData(" b ")]
    [InlineData("**B**")]
    [InlineData("Answer: B")]
    [InlineData("Final answer: B")]
    [InlineData("B) Attack.")]
    [InlineData("I want to keep the Bolt for their turn.\n\nB")]
    public void Reads_the_key_out_of_a_reply(string reply)
    {
        var chooser = new ModelChooser(new ScriptedModel(reply), "You are playing Magic.");

        Assert.Equal([1], chooser.Choose(Three));
    }

    [Fact]
    public void Reads_several_keys_where_several_were_asked_for()
    {
        var chooser = new ModelChooser(new ScriptedModel("A, C"), "You are playing Magic.");

        Assert.Equal(
            [0, 2],
            chooser.Choose(new Question
            {
                Heading = "Which of your creatures attack?",
                Choices = ["Grizzly Bears", "Llanowar Elves", "Serra Angel"],
                Picks = 3,
                UpTo = true,
            }));
    }

    [Fact]
    public void Takes_none_for_an_answer_where_choosing_nothing_is_allowed()
    {
        var chooser = new ModelChooser(new ScriptedModel("NONE"), "You are playing Magic.");

        Assert.Empty(chooser.Choose(new Question
        {
            Heading = "Which of your creatures block?",
            Choices = ["Grizzly Bears", "Llanowar Elves"],
            Picks = 2,
            UpTo = true,
        }));
    }

    [Fact]
    public void Never_reads_a_key_out_of_the_middle_of_a_sentence()
    {
        // "I" is the ninth key and "a" the first. A reply that only talks about the choices has not made one, and
        // answering it anyway would be a move nobody chose.
        var chooser = new ModelChooser(
            new ScriptedModel("I think attacking is a mistake here.", "I still think so."),
            "You are playing Magic.");

        Assert.Throws<NoChoiceException>(() => chooser.Choose(new Question
        {
            Heading = "What do you do?",
            Choices = [.. Enumerable.Range(0, 10).Select(i => $"Option {i}")],
        }));
    }

    [Fact]
    public void Asks_again_before_giving_up()
    {
        var model = new ScriptedModel("I would rather not say.", "C");
        var chooser = new ModelChooser(model, "You are playing Magic.");

        Assert.Equal([2], chooser.Choose(Three));
        Assert.Equal(2, model.Heard.Count);
        Assert.Contains("did not name one of the letters", model.Heard[1], StringComparison.Ordinal);
    }

    [Fact]
    public void Never_names_a_choice_the_question_did_not_offer()
    {
        var model = new ScriptedModel("Z", "Q", "X");
        var chooser = new ModelChooser(model, "You are playing Magic.");

        Assert.Throws<NoChoiceException>(() => chooser.Choose(Three));
    }

    [Fact]
    public void Puts_the_position_in_front_of_the_first_question_and_not_the_next_one()
    {
        // A decision is often several questions about a board that has not moved — which spell, then which target —
        // and sending the whole position again for each would pay for the same page twice.
        var model = new ScriptedModel("A", "A");
        var chooser = new ModelChooser(model, "You are playing Magic.") { Position = "Turn 3. You are at 20 life." };

        chooser.Choose(Three);
        chooser.Choose(Three);

        Assert.Contains("Turn 3", model.Heard[0], StringComparison.Ordinal);
        Assert.DoesNotContain("Turn 3", model.Heard[1], StringComparison.Ordinal);
    }

    [Fact]
    public void Keeps_the_briefing_and_drops_the_oldest_exchanges()
    {
        // Every question carries the whole position, so an old exchange is worth only the intent in it. Without a
        // bound, a long game ends every request with a hundred thousand tokens of history.
        var model = new Counting();
        var chooser = new ModelChooser(model, "You are playing Magic.", memory: 2);

        for (int i = 0; i < 10; i++)
        {
            chooser.Choose(Three);
        }

        Assert.Equal(ChatRole.System, model.Last[0].Role);
        Assert.Contains("playing Magic", model.Last[0].Text, StringComparison.Ordinal);
        Assert.True(model.Last.Count <= 1 + (2 * 2) + 1, $"the conversation grew to {model.Last.Count} turns");
    }

    /// <summary>A model that answers anything and keeps the last conversation it was handed.</summary>
    private sealed class Counting : ILanguageModel
    {
        public List<ChatTurn> Last { get; private set; } = [];

        public ModelUsage Usage { get; } = new();

        public string Continue(IReadOnlyList<ChatTurn> conversation)
        {
            Last = [.. conversation];
            Usage.Record(1, 1);
            return "A";
        }
    }
}
