using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>Localized-at-the-edge presentation facts for a card choice; transport and renderer independent.</summary>
/// <param name="AnswersOnLastPick">
/// Whether the click that reaches the count sends the choice, with no button to confirm it: a choice of exactly so many
/// permanents on the battlefield, made the way targets are. "Up to" a number keeps its button, since any count is an
/// answer, and so does a choice with a total power to reach or cards laid out in a tray.
/// </param>
public sealed record CardChoicePresentation(string Title, string Detail, string TrayTitle, string Confirm,
    bool CanSubmit, ZoneKindView? BoardZone, bool AnswersOnLastPick = false)
{
    public static CardChoicePresentation Describe(GameClient client, ChooseCardsPromptMessage prompt,
        IReadOnlyList<int> picked, PromptPresentationContext? context = null)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(prompt);
        context ??= client.Prompt?.Presentation;
        bool legend = prompt.Purpose == CardChoicePurposeView.KeepLegend;
        bool ordered = context?.Ordered ?? prompt.Purpose is CardChoicePurposeView.PutOnTop or CardChoicePurposeView.Splice;
        bool keep = prompt.Purpose == CardChoicePurposeView.Keep;
        bool permanents = prompt.Purpose is CardChoicePurposeView.Sacrifice or CardChoicePurposeView.ReturnToHand
            or CardChoicePurposeView.Untap or CardChoicePurposeView.Keep or CardChoicePurposeView.TapToPay
            or CardChoicePurposeView.Destroy or CardChoicePurposeView.Attach or CardChoicePurposeView.Encode
            or CardChoicePurposeView.Blight;
        // A mulligan's bottoming is asked by the rules and has no source; an effect's (Valakut Awakening) names its own.
        string source = legend ? "The legend rule"
            : client.Object(context?.Source ?? prompt.Source)?.Name
                ?? (prompt.Purpose == CardChoicePurposeView.PutOnBottom ? "Mulligan" : "A spell");
        // "An opponent chooses one of them. Put that card into your hand" (Karn, Scion of Urza): whoever is choosing
        // is not the one who gets the card, so "your hand" would be a lie to them.
        bool forAnother = prompt.Purpose == CardChoicePurposeView.PutIntoHand
            && (context?.Owner ?? prompt.Owner) != (context?.Chooser ?? prompt.Player);
        string verb = forAnother ? "put into its owner's hand" : prompt.Purpose switch
        {
            // A cast says what it costs and what is cast: the cards chosen from may be in any zone, and a copy is not the card.
            CardChoicePurposeView.Cast => "cast, paying its costs",
            CardChoicePurposeView.CastFree => "cast without paying its mana cost",
            CardChoicePurposeView.CastCopyFree => "cast without paying its mana cost",
            _ => Action(context?.Action ?? ActionFor(prompt.Purpose)),
        };
        bool cost = context?.Cause == PromptCauseView.Cost || context is null
            && prompt.Purpose is CardChoicePurposeView.ExileToPay or CardChoicePurposeView.ReturnToHand or CardChoicePurposeView.TapToPay;
        // Behold and blight name the cost in their own verb ("to behold", "to blight"): "to pay for it" would add nothing.
        if (cost && prompt.Purpose is not (CardChoicePurposeView.Behold or CardChoicePurposeView.Blight)) verb += " to pay for it";
        ZoneKindView? table = ChoiceTable(client, prompt);
        bool copies = prompt.Purpose == CardChoicePurposeView.CastCopyFree;
        // Behold (CR 701.4a) is a permanent you control or a card in your hand.
        string one = prompt.Purpose == CardChoicePurposeView.Behold ? "permanent or card" : copies ? "copy" : permanents ? "permanent" : "card";
        // The type the asking card narrowed the choice to ("a creature to exile"), the word "card" kept for a card in a zone
        // that is not the battlefield; the quality a behold names and the creature a blight takes are said as they are.
        bool said = prompt.Purpose is CardChoicePurposeView.Behold or CardChoicePurposeView.Blight;
        string kind = prompt.Kind.Length > 0 ? (said || permanents || table == ZoneKindView.Battlefield ? prompt.Kind : prompt.Kind + " card") : one;
        string many = prompt.Count == 1 ? kind : kind.EndsWith('y') ? kind[..^1] + "ies" : kind + "s";
        string article = "aeiou".Contains(kind[0], StringComparison.OrdinalIgnoreCase) ? "an" : "a";
        // "Any number of": every one of several that may be chosen, as the rules text says when it does not name a count.
        bool anyNumber = prompt.UpTo && prompt.Count >= 255;
        string budget = prompt.MaximumManaValue > 0 || prompt.Purpose == CardChoicePurposeView.CastCopyFree ? $" (total mana value {prompt.MaximumManaValue} or less left to spend)" : string.Empty;
        string where = table switch
        {
            ZoneKindView.Battlefield when prompt.Count > 0 => "  ·  click them on the battlefield",
            ZoneKindView.Hand when prompt.Count > 0 => "  ·  click them in your hand",
            _ => string.Empty,
        };
        int power = Cantrip.Core.Decisions.TotalPower.Of(prompt.Eligible, prompt.Powers, picked.Select(h => new ViewHandle(h)));
        int manaValue = Cantrip.Core.Decisions.TotalManaValue.Of(prompt.Eligible, prompt.ManaValues, picked.Select(h => new ViewHandle(h)));
        string title = prompt.MinimumPower > 0
            ? $"{source}: choose creatures with total power {prompt.MinimumPower} or more to tap"
            : prompt.MinimumManaValue > 0
                ? $"{source}: choose cards with total mana value {prompt.MinimumManaValue} or more to {verb}"
            : prompt.Count == 0 ? $"{source}: nothing to choose"
            : prompt.Purpose == CardChoicePurposeView.Behold && prompt.Count == 1 && !prompt.UpTo && prompt.Kind.Length > 0
                ? $"{source}: choose {article} {kind} you control or reveal {article} {kind} card from your hand to behold"
            : prompt.Purpose == CardChoicePurposeView.Blight && prompt.Count == 1 && !prompt.UpTo
                ? $"{source}: choose a creature you control to blight (it gets -1/-1 counters)"
            : prompt.Count == 1 && !prompt.UpTo ? $"{source}: choose {article} {kind} to {verb}{budget}"
            : anyNumber ? $"{source}: choose any number of {many} to {verb}{budget}"
            : $"{source}: choose {(prompt.UpTo ? "up to " : "")}{prompt.Count} {many} to {verb}{budget}";
        string detail = prompt.MinimumPower > 0
            ? $"{picked.Count} chosen  ·  total power {power} of {prompt.MinimumPower}{where}"
            : prompt.MinimumManaValue > 0
                ? $"{picked.Count} chosen  ·  total mana value {manaValue} of {prompt.MinimumManaValue}{where}"
            : ordered && prompt.Count > 1
                ? prompt.Purpose == CardChoicePurposeView.Splice
                    ? $"{picked.Count} of {prompt.Count} chosen  ·  their text follows the spell in the order chosen{where}"
                    : $"{picked.Count} of {prompt.Count} chosen  ·  they go on in the order chosen, so the last one ends up on top{where}"
                : keep ? $"{picked.Count} of {prompt.Count} chosen  ·  the rest are sacrificed once every kind is chosen{where}"
                : $"{picked.Count} of {prompt.Count} chosen{where}";
        string confirm = prompt.Count == 0 ? "Continue" : prompt.Purpose switch
        {
            CardChoicePurposeView.PutOnTop => "Put them back",
            CardChoicePurposeView.KeepLegend or CardChoicePurposeView.Keep => picked.Count == 1 ? "Keep it" : "Keep selected",
            // "Sacrifice nothing" only where declining is an answer; a mandatory sacrifice counts down to its number.
            CardChoicePurposeView.Sacrifice => picked.Count == 0 && prompt.UpTo ? "Sacrifice nothing"
                : picked.Count == 1 ? "Sacrifice it" : $"Sacrifice {picked.Count}",
            CardChoicePurposeView.Exile or CardChoicePurposeView.ExileToPay => picked.Count == 0 ? "Exile nothing" : picked.Count == 1 ? "Exile it" : $"Exile {picked.Count}",
            CardChoicePurposeView.ReturnToHand => picked.Count == 1 ? "Return it" : $"Return {picked.Count}",
            CardChoicePurposeView.Untap => picked.Count == 0 ? "Untap nothing" : $"Untap {picked.Count}",
            CardChoicePurposeView.TapToPay => picked.Count == 1 ? "Tap it" : $"Tap {picked.Count}",
            CardChoicePurposeView.PutOntoBattlefield => picked.Count == 0 ? "Put nothing" : "Put onto the battlefield",
            CardChoicePurposeView.PutIntoHand => picked.Count == 0 ? "Take nothing"
                : forAnother ? "Put into their hand" : "Put into your hand",
            CardChoicePurposeView.Cast or CardChoicePurposeView.CastFree or CardChoicePurposeView.CastCopyFree => picked.Count == 0 ? "Cast nothing" : "Cast selected",
            CardChoicePurposeView.OrderSimultaneous => "Apply next",
            CardChoicePurposeView.Destroy => picked.Count == 1 ? "Destroy it" : $"Destroy {picked.Count}",
            CardChoicePurposeView.Vote => picked.Count == 1 ? "Vote for it" : "Vote",
            CardChoicePurposeView.PutIntoGraveyard => picked.Count == 1 ? "Put it into the graveyard" : $"Put {picked.Count} into the graveyard",
            CardChoicePurposeView.Play => picked.Count == 1 ? "Play it" : $"Play {picked.Count}",
            CardChoicePurposeView.Splice => picked.Count == 0 ? "Splice nothing" : $"Splice {picked.Count}",
            CardChoicePurposeView.PutOnBottom => picked.Count == 0 && prompt.UpTo ? "Put nothing on the bottom"
                : picked.Count == 1 ? "Put it on the bottom" : $"Put {picked.Count} on the bottom",
            CardChoicePurposeView.Attach => picked.Count == 0 ? "Attach nothing" : picked.Count == 1 ? "Attach it" : $"Attach {picked.Count}",
            CardChoicePurposeView.Encode => picked.Count == 0 ? "Encode on nothing" : "Encode on it",
            CardChoicePurposeView.Behold => picked.Count == 1 ? "Behold it" : $"Behold {picked.Count}",
            CardChoicePurposeView.Blight => "Blight it",
            // "Discard nothing" only where declining is an answer; a mandatory discard still counts down to its number.
            _ => picked.Count == 0 && prompt.UpTo ? "Discard nothing"
                : picked.Count == 1 ? "Discard it" : $"Discard {picked.Count}",
        };
        bool valid = picked.Distinct().Count() == picked.Count
            && picked.All(handle => prompt.Eligible.Any(eligible => eligible.Value == handle))
            && (prompt.UpTo ? picked.Count <= prompt.Count : picked.Count == prompt.Count)
            && power >= prompt.MinimumPower
            && manaValue >= prompt.MinimumManaValue;
        bool answersOnLastPick = table == ZoneKindView.Battlefield && !prompt.UpTo && prompt.MinimumPower == 0 && prompt.MinimumManaValue == 0
            && prompt.Count > 0 && !ordered;
        return new(title, detail, Origins(client, prompt.Cards, context) + $"  ·  {prompt.Cards.Count} card{(prompt.Cards.Count == 1 ? "" : "s")}",
            confirm, valid, table, answersOnLastPick);
    }

    public static ZoneKindView? ChoiceTable(GameClient client, ChooseCardsPromptMessage prompt)
    {
        if (prompt.Cards.Count == 0) return null;
        List<ObjectView?> cards = [.. prompt.Cards.Select(client.Object)];
        if (cards.All(card => card?.Zone == ZoneKindView.Battlefield)) return ZoneKindView.Battlefield;
        return cards.All(card => card?.Zone == ZoneKindView.Hand && card.ZoneOwner == client.Seat) ? ZoneKindView.Hand : null;
    }

    public static string Origins(GameClient client, IReadOnlyList<ViewHandle> cards, PromptPresentationContext? context)
    {
        List<PromptOriginView> origins = context?.Origins ?? [.. cards.Select(client.Object).Where(card => card is not null)
            .Select(card => (card!.Zone, card.ZoneOwner)).Distinct()
            .Select(origin => new PromptOriginView { Zone = origin.Zone, Owner = origin.ZoneOwner })];
        if (origins.Count == 0) return cards.Count == 0 ? "No cards" : "Cards";
        if (origins.Count > 1) return "Cards from multiple zones";
        PromptOriginView origin = origins[0];
        string name = origin.Owner == client.Seat ? "Your" : origin.Owner == 0 ? "The"
            : $"{client.View?.Players.FirstOrDefault(player => player.Seat == origin.Owner)?.Name ?? $"seat {origin.Owner}"}'s";
        string zone = origin.Zone switch
        {
            ZoneKindView.Battlefield => "battlefield",
            ZoneKindView.Graveyard => "graveyard",
            ZoneKindView.Library => "library",
            ZoneKindView.Exile => "exile",
            ZoneKindView.Stack => "stack",
            _ => "hand",
        };
        return $"{name} {zone}";
    }

    public static PromptActionView ActionFor(CardChoicePurposeView purpose) => purpose switch
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
        _ => PromptActionView.Choose,
    };

    private static string Action(PromptActionView action) => action switch
    {
        PromptActionView.Discard => "discard",
        PromptActionView.PutOnLibrary => "put on top of your library",
        PromptActionView.Keep => "keep",
        PromptActionView.Sacrifice => "sacrifice",
        PromptActionView.Exile => "exile",
        PromptActionView.ReturnToHand => "return to its owner's hand",
        PromptActionView.Untap => "untap",
        PromptActionView.PutOntoBattlefield => "put onto the battlefield",
        PromptActionView.PutIntoHand => "put into your hand",
        PromptActionView.Cast => "cast",
        PromptActionView.Tap => "tap",
        PromptActionView.OrderReplacement => "apply replacement effects to next",
        PromptActionView.Destroy => "destroy",
        PromptActionView.Vote => "vote for",
        PromptActionView.PutIntoGraveyard => "put into its owner's graveyard",
        PromptActionView.Play => "play",
        PromptActionView.Splice => "splice onto this spell",
        PromptActionView.PutOnLibraryBottom => "put on the bottom of your library",
        PromptActionView.Attach => "attach",
        PromptActionView.Encode => "encode it on",
        PromptActionView.Behold => "behold",
        PromptActionView.Blight => "blight (put -1/-1 counters on)",
        _ => "choose",
    };
}
