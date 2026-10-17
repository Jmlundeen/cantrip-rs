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
            or CardChoicePurposeView.Tap or CardChoicePurposeView.Destroy or CardChoicePurposeView.Attach or CardChoicePurposeView.Encode
            or CardChoicePurposeView.Blight or CardChoicePurposeView.TapToReduce or CardChoicePurposeView.PutCounters
            or CardChoicePurposeView.PreventDamageFrom;
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
            && prompt.Purpose is CardChoicePurposeView.ExileToPay or CardChoicePurposeView.ReturnToHand or CardChoicePurposeView.TapToPay
                or CardChoicePurposeView.TapToReduce;
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
        // "Up to two creature cards with total mana value 6 or less" (CR 202.3): the cards chosen together, not each one.
        if (prompt.MaximumTotalManaValue is { } within) many += $" with total mana value {within} or less";
        string article = "aeiou".Contains(kind[0], StringComparison.OrdinalIgnoreCase) ? "an" : "a";
        // "Any number of": every one of several that may be chosen, as the rules text says when it does not name a count.
        bool anyNumber = prompt.UpTo && prompt.Count >= 255;
        string budget = prompt.MaximumManaValue > 0 || prompt.Purpose == CardChoicePurposeView.CastCopyFree ? $" (total mana value {prompt.MaximumManaValue} or less left to spend)" : string.Empty;
        // Cards brought into the game from outside it (CR 400.11b: North Wind Avatar) are the chooser's sideboard, which only they
        // may look at: the caption says where they come from, since nothing on the table shows it.
        bool outside = prompt.Cards.Count > 0 && prompt.Cards.All(card => client.Object(card)?.Zone == ZoneKindView.Sideboard);
        string from = outside ? " you own from outside the game" : string.Empty;
        string where = table switch
        {
            ZoneKindView.Battlefield when prompt.Count > 0 => "  ·  click them on the battlefield",
            ZoneKindView.Hand when prompt.Count > 0 => "  ·  click them in your hand",
            _ when outside => "  ·  from your sideboard, which only you can see",
            _ => string.Empty,
        };
        int power = Cantrip.Core.Decisions.TotalPower.Of(prompt.Eligible, prompt.Powers, picked.Select(h => new ViewHandle(h)));
        int manaValue = Cantrip.Core.Decisions.TotalManaValue.Of(prompt.Eligible, prompt.ManaValues, picked.Select(h => new ViewHandle(h)));
        // Craft (CR 702.167a): the materials are permanents you control and cards in your graveyard, of the quality the ability
        // names, exiled with the crafting permanent as its cost.
        bool craft = prompt.Purpose == CardChoicePurposeView.Craft;
        // A material named as a card ("red instant and/or sorcery card") is a card in the graveyard alone (CR 702.167b).
        bool cardsOnly = prompt.Kind.EndsWith(" card", StringComparison.Ordinal);
        string materials = prompt.Kind.Length == 0 ? "other permanents you control and/or cards in your graveyard"
            : cardsOnly ? $"{Plural(prompt.Kind)} in your graveyard"
            : $"{Plural(prompt.Kind)} you control and/or {prompt.Kind} cards in your graveyard";
        string title = craft
            ? prompt.Minimum > 0
                ? $"{source}: craft - choose {prompt.Minimum} or more from among {materials} to exile"
                : prompt.Count == 1 && prompt.Kind.Length > 0 && !cardsOnly
                    ? $"{source}: craft - choose {article} {prompt.Kind} you control or {article} {prompt.Kind} card from your graveyard to exile"
                    : $"{source}: craft - choose {prompt.Count} from among {materials} to exile"
            : prompt.MinimumPower > 0
            ? $"{source}: choose creatures with total power {prompt.MinimumPower} or more to tap"
            : prompt.MinimumManaValue > 0
                ? $"{source}: choose cards with total mana value {prompt.MinimumManaValue} or more to {verb}"
            : prompt.Count == 0 ? $"{source}: nothing to choose"
            // Harmonize (CR 702.180a): up to one, and what tapping it does to the cost is said before it is chosen.
            : prompt.Purpose == CardChoicePurposeView.TapToReduce
                ? $"{source}: tap up to one untapped creature you control to reduce the cost by generic mana equal to its power"
            : prompt.Purpose == CardChoicePurposeView.Behold && prompt.Count == 1 && !prompt.UpTo && prompt.Kind.Length > 0
                ? $"{source}: choose {article} {kind} you control or reveal {article} {kind} card from your hand to behold"
            // "You may behold" as a permanent enters (CR 614.12a): choosing none is the answer that doesn't.
            : prompt.Purpose == CardChoicePurposeView.Behold && prompt.Count == 1 && prompt.UpTo && prompt.Kind.Length > 0
                ? $"{source}: you may choose {article} {kind} you control or reveal {article} {kind} card from your hand to behold"
            : prompt.Purpose == CardChoicePurposeView.Blight && prompt.Count == 1 && !prompt.UpTo
                ? $"{source}: choose a creature you control to blight (it gets -1/-1 counters)"
            // Not a target (empower Jace's token, amass's Army): the one chosen gets the counters, and nothing is kept or lost.
            : prompt.Purpose == CardChoicePurposeView.PutCounters && prompt.Count == 1 && !prompt.UpTo
                ? $"{source}: choose {article} {kind} to put {Counters(prompt)} on"
            // A prevention shield two or more sources would deal damage to at once (CR 615.7): whose damage it prevents first.
            : prompt.Purpose == CardChoicePurposeView.PreventDamageFrom
                ? $"{source}: choose the source whose damage the shield prevents first"
            : prompt.Count == 1 && !prompt.UpTo ? $"{source}: choose {article} {kind}{from} to {verb}{budget}"
            : anyNumber ? $"{source}: choose any number of {many}{from} to {verb}{budget}"
            : $"{source}: choose {(prompt.UpTo ? "up to " : "")}{prompt.Count} {many}{from} to {verb}{budget}";
        string detail = craft
            ? $"{picked.Count} chosen  ·  exiled with {source} to pay the craft cost; {source} returns transformed"
            : prompt.Purpose == CardChoicePurposeView.PreventDamageFrom
            ? $"It has {prompt.PreventLeft} damage left to prevent; what it can't prevent of the others' damage is dealt{where}"
            : prompt.Purpose == CardChoicePurposeView.TapToReduce
            ? picked.Count == 0
                ? $"None tapped: the full cost is paid{where}"
                : $"Tapping it takes {power} off the generic mana of the cost (never colored mana){where}"
            : prompt.MinimumPower > 0
            ? $"{picked.Count} chosen  ·  total power {power} of {prompt.MinimumPower}{where}"
            : prompt.MinimumManaValue > 0
                ? $"{picked.Count} chosen  ·  total mana value {manaValue} of {prompt.MinimumManaValue}{where}"
            : prompt.MaximumTotalManaValue is { } most
                ? $"{picked.Count} of {prompt.Count} chosen  ·  total mana value {manaValue} of {most} or less{where}"
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
            CardChoicePurposeView.TapToPay or CardChoicePurposeView.Tap => picked.Count == 1 ? "Tap it" : $"Tap {picked.Count}",
            CardChoicePurposeView.TapToReduce => picked.Count == 0 ? "Tap nothing" : power > 0 ? $"Tap it (-{power})" : "Tap it",
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
            CardChoicePurposeView.Behold => picked.Count == 0 && prompt.UpTo ? "Behold nothing" : picked.Count == 1 ? "Behold it" : $"Behold {picked.Count}",
            CardChoicePurposeView.Blight => "Blight it",
            CardChoicePurposeView.PutCounters => $"Put {Counters(prompt)} on it",
            CardChoicePurposeView.Craft => picked.Count == 1 ? "Exile it" : $"Exile {picked.Count}",
            CardChoicePurposeView.Reveal => picked.Count == 0 ? "Reveal nothing" : "Reveal it",
            CardChoicePurposeView.PreventDamageFrom => "Prevent its damage first",
            // "Discard nothing" only where declining is an answer; a mandatory discard still counts down to its number.
            _ => picked.Count == 0 && prompt.UpTo ? "Discard nothing"
                : picked.Count == 1 ? "Discard it" : $"Discard {picked.Count}",
        };
        bool valid = picked.Distinct().Count() == picked.Count
            && picked.All(handle => prompt.Eligible.Any(eligible => eligible.Value == handle))
            && (prompt.UpTo ? picked.Count <= prompt.Count && picked.Count >= prompt.Minimum : picked.Count == prompt.Count)
            && power >= prompt.MinimumPower
            && manaValue >= prompt.MinimumManaValue
            && (prompt.MaximumTotalManaValue is not { } limit || manaValue <= limit);
        bool answersOnLastPick = table == ZoneKindView.Battlefield && !prompt.UpTo && prompt.MinimumPower == 0 && prompt.MinimumManaValue == 0
            && prompt.Count > 0 && !ordered;
        return new(title, detail, Origins(client, prompt.Cards, context) + $"  ·  {prompt.Cards.Count} card{(prompt.Cards.Count == 1 ? "" : "s")}",
            confirm, valid, table, answersOnLastPick);
    }

    /// <summary>
    /// Whether clicking <paramref name="handle"/> keeps a choice "with total mana value N or less" (CR 202.3) within it: a card
    /// that would take the picked cards over the limit is refused, as the engine would refuse the answer. A picked card can
    /// always be clicked (to put it back), and any card can when the choice has no such limit. With one to choose, a click
    /// trades, so the card is judged alone.
    /// </summary>
    public static bool Fits(ChooseCardsPromptMessage prompt, IReadOnlyList<int> picked, int handle)
    {
        ArgumentNullException.ThrowIfNull(prompt);
        ArgumentNullException.ThrowIfNull(picked);
        if (prompt.MaximumTotalManaValue is null || picked.Contains(handle)) return true;
        IEnumerable<int> after = prompt.Count == 1 ? [handle] : picked.Append(handle);
        return Cantrip.Core.Decisions.TotalManaValue.Within(prompt.Eligible, prompt.ManaValues, after.Select(h => new ViewHandle(h)), prompt.MaximumTotalManaValue);
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
            ZoneKindView.Sideboard => "sideboard",
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
        CardChoicePurposeView.TapToPay or CardChoicePurposeView.Tap => PromptActionView.Tap,
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
        CardChoicePurposeView.Craft => PromptActionView.Craft,
        CardChoicePurposeView.Reveal => PromptActionView.Reveal,
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
        PromptActionView.TapToReduce => "tap to reduce the cost",
        PromptActionView.PutCounters => "put the counters on",
        PromptActionView.Craft => "exile to craft",
        PromptActionView.Reveal => "reveal",
        _ => "choose",
    };

    /// <summary>A material named by its quality, more than one: "artifacts", "creatures", "Caves".</summary>
    private static string Plural(string kind) => kind.EndsWith('y') ? kind[..^1] + "ies" : kind + "s";

    /// <summary>The counters a <see cref="CardChoicePurposeView.PutCounters"/> choice puts on what is chosen: "2 loyalty counters", "a +1/+1 counter".</summary>
    public static string Counters(ChooseCardsPromptMessage prompt)
    {
        ArgumentNullException.ThrowIfNull(prompt);
        string kind = prompt.Counter.Length > 0 ? prompt.Counter + " " : string.Empty;
        return prompt.CounterCount switch
        {
            <= 0 => "the counters",
            1 => $"{(kind.Length > 0 && "aeiou".Contains(kind[0], StringComparison.OrdinalIgnoreCase) ? "an" : "a")} {kind}counter",
            _ => $"{prompt.CounterCount} {kind}counters",
        };
    }
}
