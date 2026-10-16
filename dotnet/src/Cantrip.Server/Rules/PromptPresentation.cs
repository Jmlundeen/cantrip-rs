using Cantrip.Protocol;

namespace Cantrip.Server.Rules;

/// <summary>
/// Wording facts for a prompt that arrives already on the wire, from the prompt itself and the view it was asked
/// against: what <see cref="PromptPresentationProjector"/> reads off the engine's prompt and state, read instead off
/// the wire prompt and the seat's view. Presentation, not rules: nothing here decides what is legal.
/// </summary>
internal static class PromptPresentation
{
    public static PromptPresentationContext? Describe(PromptPayload prompt, bool mayCancel, PlayerView? view) => prompt switch
    {
        ChooseCardsPromptMessage choose => new()
        {
            Action = Action(choose.Purpose),
            Cause = choose.Purpose is CardChoicePurposeView.KeepLegend or CardChoicePurposeView.OrderSimultaneous
                || choose.Purpose == CardChoicePurposeView.PutOnBottom && choose.Source.IsNone ? PromptCauseView.Rule
                : IsCost(choose.Purpose) || mayCancel && choose.Purpose != CardChoicePurposeView.Splice
                    ? PromptCauseView.Cost : PromptCauseView.Effect,
            Chooser = choose.Player,
            Owner = choose.Owner,
            Source = choose.Source,
            Minimum = choose.UpTo ? 0 : choose.Count,
            Maximum = choose.Count,
            Ordered = choose.Purpose is CardChoicePurposeView.PutOnTop or CardChoicePurposeView.Splice,
            Origins = Origins(choose.Cards, view),
        },
        DistributeCardsPromptMessage distribute => new()
        {
            Action = PromptActionView.Distribute,
            Cause = PromptCauseView.Effect,
            Chooser = distribute.Player,
            Owner = distribute.Player,
            Source = distribute.Source,
            Minimum = distribute.Cards.Count,
            Maximum = distribute.Cards.Count,
            Ordered = distribute.Destinations.Any(destination => destination.Ordered),
            Origins = Origins(distribute.Cards, view),
        },
        PayCostsPromptMessage pay => new()
        {
            Action = PromptActionView.Pay,
            Cause = PromptCauseView.Cost,
            Chooser = pay.Player,
            Owner = pay.Player,
            Source = pay.Spell,
        },
        _ => null,
    };

    /// <summary>
    /// The purposes that pay something, which the engine's prompt says of itself and the wire's does not: those whose
    /// own description is a cost.
    /// </summary>
    private static bool IsCost(CardChoicePurposeView purpose) => purpose is
        CardChoicePurposeView.ExileToPay or CardChoicePurposeView.TapToPay
        or CardChoicePurposeView.Sacrifice or CardChoicePurposeView.ReturnToHand or CardChoicePurposeView.TapToReduce;

    /// <summary>The zones the cards are in, as the view the seat holds places them; a card not in it says nothing.</summary>
    private static List<PromptOriginView> Origins(IEnumerable<ViewHandle> cards, PlayerView? view)
    {
        if (view is null)
        {
            return [];
        }

        Dictionary<ViewHandle, ObjectView> objects = [];
        foreach (ObjectView item in view.Objects)
        {
            objects[item.Handle] = item;
        }

        return [.. cards
            .Select(handle => objects.GetValueOrDefault(handle))
            .Where(card => card is not null)
            .Select(card => (card!.Zone, card.ZoneOwner))
            .Distinct()
            .Select(zone => new PromptOriginView { Zone = zone.Zone, Owner = zone.ZoneOwner })];
    }

    private static PromptActionView Action(CardChoicePurposeView purpose) => purpose switch
    {
        CardChoicePurposeView.Discard => PromptActionView.Discard,
        CardChoicePurposeView.PutOnTop => PromptActionView.PutOnLibrary,
        CardChoicePurposeView.KeepLegend or CardChoicePurposeView.Keep => PromptActionView.Keep,
        CardChoicePurposeView.Sacrifice => PromptActionView.Sacrifice,
        CardChoicePurposeView.Exile or CardChoicePurposeView.ExileToPay => PromptActionView.Exile,
        CardChoicePurposeView.ReturnToHand => PromptActionView.ReturnToHand,
        CardChoicePurposeView.Untap => PromptActionView.Untap,
        CardChoicePurposeView.PutOntoBattlefield => PromptActionView.PutOntoBattlefield,
        CardChoicePurposeView.PutIntoHand => PromptActionView.PutIntoHand,
        CardChoicePurposeView.Cast or CardChoicePurposeView.CastFree or CardChoicePurposeView.CastCopyFree => PromptActionView.Cast,
        CardChoicePurposeView.TapToPay => PromptActionView.Tap,
        CardChoicePurposeView.OrderSimultaneous => PromptActionView.OrderReplacement,
        CardChoicePurposeView.Destroy => PromptActionView.Destroy,
        CardChoicePurposeView.Vote => PromptActionView.Vote,
        CardChoicePurposeView.PutIntoGraveyard => PromptActionView.PutIntoGraveyard,
        CardChoicePurposeView.Play => PromptActionView.Play,
        CardChoicePurposeView.Splice => PromptActionView.Splice,
        CardChoicePurposeView.PutOnBottom => PromptActionView.PutOnLibraryBottom,
        CardChoicePurposeView.Attach => PromptActionView.Attach,
        CardChoicePurposeView.Encode => PromptActionView.Encode,
        CardChoicePurposeView.Behold => PromptActionView.Behold,
        CardChoicePurposeView.Blight => PromptActionView.Blight,
        CardChoicePurposeView.TapToReduce => PromptActionView.TapToReduce,
        CardChoicePurposeView.PutCounters => PromptActionView.PutCounters,
        _ => PromptActionView.Choose,
    };
}
