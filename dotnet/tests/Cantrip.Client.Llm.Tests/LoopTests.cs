using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Llm.Tests.Board;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// What stops a seat that thinks about its answer from thinking the same thing for ever.
/// <para>
/// <see cref="RandomPlay"/> cannot loop: a fresh draw from the same menu eventually passes. A seat asked what it
/// wants answers the same way every time it is shown the same board, so giving up on a spell (CR 733.1) — which
/// puts the game back exactly where it was, same menu and all — is a loop unless something remembers. Here that
/// costs money by the lap, so it is guarded twice: the spell comes off the menu, and a step that has asked far more
/// than any real step needs stops asking at all.
/// </para>
/// </summary>
public sealed class LoopTests
{
    [Fact]
    public void Never_announces_a_spell_again_after_giving_up_on_it_this_step()
    {
        var chooser = new FirstChooser();
        var decisions = new Decisions(chooser);
        GameClient client = Seated(Card(2, "Phlage", "{1}{R}{W}"), Card(3, "Mountain"));

        // Offered, taken, and then unpayable: the colour it owes is one nothing left can make.
        Assert.IsType<TakeActionAnswer>(decisions.Answer(client, Offer()));

        AnswerPayload gave = decisions.Answer(client, Unpayable());
        Assert.Equal(ActionKindView.CancelCasting, Assert.IsType<TakeActionAnswer>(gave).Kind);

        // The server now offers exactly the same menu again. Taking it again is the loop.
        Assert.IsType<PassAnswer>(decisions.Answer(client, Offer()));
    }

    [Fact]
    public void Offers_it_again_once_the_step_has_moved_on()
    {
        // Held against the spell for a step, not for the game: a cost out of reach in an upkeep is well within it
        // after an untap and a land.
        var decisions = new Decisions(new FirstChooser());
        GameClient client = Seated(Card(2, "Phlage", "{1}{R}{W}"));

        decisions.Answer(client, Offer());
        decisions.Answer(client, Unpayable());
        Assert.IsType<PassAnswer>(decisions.Answer(client, Offer()));

        Assert.IsType<TakeActionAnswer>(decisions.Answer(Later(client), Offer()));
    }

    [Fact]
    public void Stops_asking_once_a_step_has_taken_more_questions_than_any_step_needs()
    {
        // The backstop, for a loop that is not a cancelled cast. Nothing about a real step takes forty decisions,
        // and without a ceiling a loop runs until the credits do.
        var chooser = new FirstChooser();
        var decisions = new Decisions(chooser);
        GameClient client = Seated(Card(2, "Lightning Bolt", "{R}"), Card(3, "Shock", "{R}"));

        var answers = new List<AnswerPayload>();

        for (int i = 0; i < 60; i++)
        {
            answers.Add(decisions.Answer(client, Offer()));
        }

        Assert.Contains(answers, answer => answer is PassAnswer);
        Assert.True(chooser.Questions < 60, $"asked {chooser.Questions} times in one step");

        // And it starts fresh when the game moves on, rather than being wedged for good.
        Assert.IsType<TakeActionAnswer>(decisions.Answer(Later(client), Offer()));
    }

    /// <summary>The same menu every time: one castable spell, and passing.</summary>
    private static PriorityPromptMessage Offer() => new()
    {
        Player = Me,
        Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(2), Affordable = true }],
    };

    /// <summary>Paying for it, with nothing on offer that makes the colour still owed.</summary>
    private static PayCostsPromptMessage Unpayable() => new()
    {
        Player = Me,
        Spell = Handle(2),
        Total = "{1}{R}{W}",
        Remaining = "{W}",
        RemainingColored = [ColorView.White],
        Options = [new() { Kind = ActionKindView.CancelCasting }],
    };

    /// <summary>The same client, one step later.</summary>
    private static GameClient Later(GameClient client)
    {
        client.View!.Step = TurnStepView.PostcombatMain;
        return client;
    }
}
