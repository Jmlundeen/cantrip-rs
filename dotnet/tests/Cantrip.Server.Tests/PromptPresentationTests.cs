using Cantrip.Protocol;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// What the server says a card choice is for, read off the wire prompt (<see cref="PromptPresentation"/>): a cost only
/// where something is paid. Words of Wind's "each player returns a permanent they control to its owner's hand instead"
/// (KI-75) and an edict's sacrifice are effects, asked with no way back; the same purposes chosen while casting or
/// activating are costs, which the player may back out of (CR 601.2h, 602.2b).
/// </summary>
public sealed class PromptPresentationTests
{
    [Theory]
    [InlineData(CardChoicePurposeView.ReturnToHand, false, PromptCauseView.Effect)]
    [InlineData(CardChoicePurposeView.ReturnToHand, true, PromptCauseView.Cost)]
    [InlineData(CardChoicePurposeView.Sacrifice, false, PromptCauseView.Effect)]
    [InlineData(CardChoicePurposeView.Sacrifice, true, PromptCauseView.Cost)]
    [InlineData(CardChoicePurposeView.ExileToPay, false, PromptCauseView.Cost)]
    [InlineData(CardChoicePurposeView.TapToPay, false, PromptCauseView.Cost)]
    public void A_choice_is_a_cost_only_where_something_is_paid(CardChoicePurposeView purpose, bool mayCancel, PromptCauseView cause)
    {
        var prompt = new ChooseCardsPromptMessage
        {
            Player = 1, Owner = 1, Source = new ViewHandle(3), Purpose = purpose, Cards = [new ViewHandle(4)], Eligible = [new ViewHandle(4)], Count = 1,
        };

        PromptPresentationContext? context = PromptPresentation.Describe(prompt, mayCancel, view: null);

        Assert.Equal(cause, context?.Cause);
    }
}
