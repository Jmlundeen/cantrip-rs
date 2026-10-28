using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// Paying for a spell with assist (CR 702.132a): the steps of the player chosen to help offer no giving up, since the spell
/// is not theirs, so every seat answers each step with something it offers - the windows closed, none of the generic mana
/// announced, pool mana assigned if asked for - and the question's own default does the same, as the host's does.
/// </summary>
public sealed class AssistPaymentTests
{
    private static readonly ViewHandle Spell = new(4);

    private static PayCostsPromptMessage Step(string step, params ActionOptionMessage[] options) => new()
    {
        Player = 2,
        Spell = Spell,
        Total = "{5}",
        Remaining = "{5}",
        RemainingGeneric = 5,
        AssistStep = step,
        AnnouncesXOrKicks = step == AssistSteps.HelperAnnounces,
        MostAnnounced = step == AssistSteps.HelperAnnounces ? 2 : 0,
        MostAffordable = -1,
        Options = [.. options],
    };

    private static ActionOptionMessage Forest => new() { Kind = ActionKindView.ActivateAbility, Source = new ViewHandle(7), Produces = [ManaTypeView.Green], Affordable = true };

    private static ActionOptionMessage Option(ActionKindView kind) => new() { Kind = kind, Source = ViewHandle.None };

    private static ActionOptionMessage PoolGreen => new() { Kind = ActionKindView.AssignMana, Source = ViewHandle.None, Produces = [ManaTypeView.Green] };

    public static TheoryData<string> HelperSteps => [AssistSteps.HelperActivates, AssistSteps.HelperAnnounces, AssistSteps.HelperPays];

    private static PayCostsPromptMessage HelperStep(string step) => step switch
    {
        AssistSteps.HelperActivates => Step(step, Forest, Option(ActionKindView.ConfirmCasting)),
        AssistSteps.HelperAnnounces => Step(step, Option(ActionKindView.IncreaseX), Option(ActionKindView.ConfirmCasting)),
        _ => Step(step, PoolGreen),
    };

    [Theory]
    [MemberData(nameof(HelperSteps))]
    public void The_helpers_default_is_an_option_it_offers(string step)
    {
        PayCostsPromptMessage prompt = HelperStep(step);
        var answer = Assert.IsType<TakeActionAnswer>(prompt.Default());
        Assert.Contains(prompt.Options, option => option.Kind == answer.Kind);
        Assert.NotEqual(ActionKindView.CancelCasting, answer.Kind);
        if (answer.Kind == ActionKindView.AssignMana)
        {
            Assert.Equal(ManaTypeView.Green, answer.Mana);
        }
    }

    [Fact]
    public void A_payment_that_may_be_given_up_still_defaults_to_giving_it_up()
    {
        PayCostsPromptMessage prompt = Step(AssistSteps.CasterActivates, Forest, Option(ActionKindView.ConfirmCasting), Option(ActionKindView.CancelCasting));
        Assert.Equal(ActionKindView.CancelCasting, prompt.GiveUp().Kind);
    }

    [Theory]
    [MemberData(nameof(HelperSteps))]
    public void Every_seat_helps_with_what_the_step_offers(string step)
    {
        PayCostsPromptMessage prompt = HelperStep(step);
        List<AnswerPayload> answers = [AutoPlay.Pay(prompt), new RandomPlay(seed: 7).Decide(prompt)];
        foreach (AnswerPayload answer in answers)
        {
            var taken = Assert.IsType<TakeActionAnswer>(answer);
            Assert.Contains(prompt.Options, option => option.Kind == taken.Kind);
        }
    }

    [Fact]
    public void A_seat_closes_its_window_and_announces_no_help()
    {
        Assert.Equal(ActionKindView.ConfirmCasting, AutoPlay.Pay(HelperStep(AssistSteps.HelperActivates)).Kind);
        Assert.Equal(ActionKindView.ConfirmCasting, AutoPlay.Pay(HelperStep(AssistSteps.HelperAnnounces)).Kind);
    }

    [Fact]
    public void A_seat_chooses_nobody_to_help_pay_for_its_spell()
    {
        var may = new MayPromptMessage { Player = 1, Source = Spell, Verb = "Assist" };
        for (int seed = 0; seed < 20; seed++)
        {
            Assert.False(Assert.IsType<MayChosenAnswer>(new RandomPlay(seed).Decide(may)).Accept);
        }
    }
}
