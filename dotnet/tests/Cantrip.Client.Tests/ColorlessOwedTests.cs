using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// A cost that still owes a colorless symbol {C} (CR 107.4c), and nothing else, as every seat reads it.
/// <para>
/// The pay prompt carries {C} in a field of its own, <see cref="PayCostsPromptMessage.RemainingColorless"/>, beside
/// generic, colored and hybrid. A seat that read only the other three would think Sowing Mycospawn's kicked cost paid
/// once all but its {C} was covered, and a bot paying for a kicker would then raise, fail and lower it for ever — the
/// same shape <c>RemainingHybrid</c> was added for. Green in the pool is the trap: it is mana, it is offered, and it
/// pays nothing a {C} is owed.
/// </para>
/// </summary>
public sealed class ColorlessOwedTests
{
    private static readonly ViewHandle Spell = new(1);
    private static readonly ViewHandle SolRing = new(2);
    private static readonly ViewHandle Forest = new(3);

    /// <summary>Sowing Mycospawn kicked for {1}{C}, everything paid but the {C}, with <paramref name="options"/> offered.</summary>
    private static PayCostsPromptMessage OwingOnlyColorless(params ActionOptionMessage[] options) => new()
    {
        Spell = Spell,
        Total = "{4}{G}{C}",
        Remaining = "{C}",
        RemainingColorless = 1,
        HasX = true,
        X = 1,
        Kicker = "{1}{C}",
        Options =
        [
            .. options,
            new ActionOptionMessage { Kind = ActionKindView.DecreaseX, Source = Spell },
            new ActionOptionMessage { Kind = ActionKindView.CancelCasting, Source = Spell },
        ],
    };

    private static ActionOptionMessage Tap(ViewHandle source, params ManaTypeView[] produces) =>
        new() { Kind = ActionKindView.ActivateAbility, Source = source, Produces = [.. produces] };

    private static ActionOptionMessage Floating(ManaTypeView type) =>
        new() { Kind = ActionKindView.AssignMana, Source = Spell, Produces = [type] };

    [Fact]
    public void A_cost_owing_only_colorless_is_owing()
    {
        PayCostsPromptMessage prompt = OwingOnlyColorless();

        Assert.True(AutoPlay.Owing(prompt));

        ManaOwed owed = ManaOwed.Of(prompt)!;
        Assert.True(owed.Any);
        Assert.Equal(1, owed.Colorless);

        // Owing, so it is not "nothing owed plus one generic for X", which is how a paid X cost reads.
        Assert.Equal(0, owed.Generic);
    }

    [Fact]
    public void The_bot_taps_a_colorless_source_for_it_rather_than_green()
    {
        PayCostsPromptMessage prompt = OwingOnlyColorless(
            Floating(ManaTypeView.Green),
            Tap(Forest, ManaTypeView.Green),
            Tap(SolRing, ManaTypeView.Colorless, ManaTypeView.Colorless));

        TakeActionAnswer answer = AutoPlay.Pay(prompt);

        Assert.Equal(ActionKindView.ActivateAbility, answer.Kind);
        Assert.Equal(SolRing, answer.Source);
    }

    [Fact]
    public void Colorless_floating_in_the_pool_is_put_towards_it_and_green_is_not()
    {
        PayCostsPromptMessage prompt = OwingOnlyColorless(Floating(ManaTypeView.Green), Floating(ManaTypeView.Colorless));

        TakeActionAnswer answer = AutoPlay.Pay(prompt);

        Assert.Equal(ActionKindView.AssignMana, answer.Kind);
        Assert.Equal(ManaTypeView.Colorless, answer.Mana);
    }

    [Fact]
    public void With_nothing_that_makes_colorless_the_bot_lowers_the_kick_instead_of_spending_green_on_it()
    {
        PayCostsPromptMessage prompt = OwingOnlyColorless(Floating(ManaTypeView.Green), Tap(Forest, ManaTypeView.Green));

        TakeActionAnswer answer = AutoPlay.Pay(prompt);

        Assert.Equal(ActionKindView.DecreaseX, answer.Kind);
    }

    [Fact]
    public void Clicking_a_Sol_Ring_while_a_colorless_symbol_is_owed_taps_it_for_that()
    {
        ActionOptionMessage ring = Tap(SolRing, ManaTypeView.Colorless, ManaTypeView.Colorless);
        PayCostsPromptMessage prompt = OwingOnlyColorless(ring, Tap(Forest, ManaTypeView.Green));

        (ActionOptionMessage Option, ColorView? Color)? pick =
            AutoMana.Pick([(ring, null)], prompt.Options, ManaOwed.Of(prompt)!);

        Assert.NotNull(pick);
        Assert.Same(ring, pick.Value.Option);
    }
}
