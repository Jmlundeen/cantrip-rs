using Cantrip.Protocol;

namespace Cantrip.Client.Drafting;

/// <summary>
/// How good a card is taken by itself, as one number, from the two places a drafter can learn it.
/// <para>
/// <b>A pick order</b>, when the cube has one and the server says where a card stands in it
/// (<see cref="DraftMessage.PileRanks"/>): consensus, from what people took. A rank is read onto the scale the order
/// was fitted on - a Bradley-Terry strength, the log-odds of one card being taken over another - which across a cube
/// follows the log-odds of the rank itself closely: the middle of the order is flat, a few places apart meaning
/// little, and both ends are steep.
/// </para>
/// <para>
/// <b>The card itself</b>, for a card no order names: what it costs, what it is and what its text says
/// (<see cref="Estimate"/>), weighted by a fit of those to the pick orders of the cubes there are
/// (<c>cantrip-botlab draft fit</c>). Rough - it knows a cheap removal spell from an expensive vanilla creature, not a
/// good card from a great one - but it is what lets the drafter take a card it has never seen for about what it is.
/// </para>
/// <para>
/// The scale is that strength plus three, so that a card nobody would play is near nothing and every card is worth
/// something: about 3 for a cube's middling card, 4 for its best quarter, 5 for its best twenty, 7 for the Power Nine.
/// </para>
/// </summary>
public static class CardWorth
{
    /// <summary>What a middling card of a cube is worth.</summary>
    public const double Middling = 3.0;

    /// <summary>
    /// The worth of the card at <paramref name="rank"/> (best from 0) among <paramref name="total"/> ranked.
    /// </summary>
    public static double FromRank(int rank, int total)
    {
        total = Math.Max(total, 2);
        double place = (Math.Clamp(rank, 0, total - 1) + 0.5) / total;
        return Math.Clamp(Middling + (0.65 * Math.Log((1 - place) / place)), 0.3, 8.0);
    }

    /// <summary>What a gold card is worth in a deck of its colours beyond what it is worth as a pick.</summary>
    public const double GoldInDeck = 0.4;

    /// <summary>What a colourless card is worth in a deck less than it is worth as a pick.</summary>
    public const double ColorlessInDeck = -0.3;

    /// <summary>
    /// What a card is worth in a deck that can cast it, over its worth as a pick. People take a colourless card earlier
    /// than it is good, because every deck can play it, and a gold card later, because few can - and a pick order is
    /// what people took. The drafter works out for itself which decks a card could go in, so it takes that back out.
    /// </summary>
    public static double InDeck(DraftCard card)
    {
        ArgumentNullException.ThrowIfNull(card);
        return card.IsLand ? 0 : int.PopCount(card.Colors) switch { 0 => ColorlessInDeck, 1 => 0, _ => GoldInDeck };
    }

    /// <summary>What a card is, as numbers: what <see cref="Weights"/> are weights of.</summary>
    public static Dictionary<string, double> Features(DraftCard card, string lowerText)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(lowerText);
        ObjectView view = card.View;
        Dictionary<string, double> features = new(StringComparer.Ordinal) { ["bias"] = 1 };

        if (card.IsLand)
        {
            int colors = int.PopCount(card.Produces);
            features[colors >= 5 ? "land.any" : colors >= 2 ? "land.dual" : colors == 1 ? "land.one" : "land.colorless"] = 1;
            if (lowerText.Contains("search your library", StringComparison.Ordinal)) features["land.fetch"] = 1;
            if (view.Subtypes.Count >= 2) features["land.typed"] = 1;
            if (lowerText.Contains("enters tapped", StringComparison.Ordinal) || lowerText.Contains("enters the battlefield tapped", StringComparison.Ordinal)) features["land.tapped"] = 1;
            features["land.text"] = Math.Min(lowerText.Length, 400) / 100.0;
            return features;
        }

        int mv = view.ManaValue;
        features["mv"] = Math.Min(mv, 8);
        features[$"mv.{Math.Min(mv, 6)}"] = 1;
        features["text"] = Math.Min(lowerText.Length, 500) / 100.0;

        int colorCount = int.PopCount(card.Colors);
        if (colorCount == 0) features["colorless"] = 1;
        if (colorCount >= 2) features["gold"] = 1;
        if (colorCount >= 3) features["gold.3"] = 1;
        if (card.Costs.Count > 0 && card.Costs[0].Count >= 2 && colorCount == 1) features["pips.2"] = 1;
        if (view.ManaCost.Contains("{X}", StringComparison.Ordinal)) features["x"] = 1;

        if (card.IsCreature)
        {
            features["creature"] = 1;
            int power = view.Power ?? 0;
            int toughness = view.Toughness ?? 0;
            features["creature.body"] = Math.Clamp(((power + toughness) / 2.0) - mv, -4, 2);
            features["creature.power"] = Math.Min(power, 8);
            if (card.Evasive) features["creature.evasive"] = 1;
            foreach (string keyword in (string[])["flying", "haste", "lifelink", "deathtouch", "first strike", "double strike", "vigilance", "flash", "hexproof", "ward", "trample", "menace"])
            {
                if (DraftCard.HasKeyword(lowerText, keyword)) features["kw." + keyword] = 1;
            }
        }

        foreach (CardTypeView type in view.CardTypes)
        {
            if (type != CardTypeView.Creature) features["type." + type] = 1;
        }

        if (view.Subtypes.Contains("Equipment")) features["equipment"] = 1;
        if (view.Subtypes.Contains("Aura")) features["aura"] = 1;
        if (view.Subtypes.Contains("Vehicle")) features["vehicle"] = 1;
        if (card.Removal) features["removal"] = 1;
        if (card.Removal && mv <= 2) features["removal.cheap"] = 1;
        if (card.Sweeper) features["sweeper"] = 1;
        if (card.Counterspell) features["counter"] = 1;
        if (card.CardAdvantage) features["advantage"] = 1;
        if (lowerText.Contains("draw", StringComparison.Ordinal)) features["draw"] = 1;
        if (card.MakesMana) features["mana"] = 1;
        if (card.MakesMana && mv <= 1) features["mana.cheap"] = 1;
        if (card.MakesMana && mv == 0) features["mana.free"] = 1;
        if (card.ManaMade >= 2) features["mana.two"] = 1;
        if (lowerText.Contains("add {", StringComparison.Ordinal) || lowerText.Contains("mana of any", StringComparison.Ordinal)) features["adds"] = 1;
        if (lowerText.Contains("search your library", StringComparison.Ordinal)) features["tutor"] = 1;
        if (lowerText.Contains("enters", StringComparison.Ordinal)) features["enters"] = 1;
        if (lowerText.Contains("create", StringComparison.Ordinal)) features["tokens"] = 1;
        if (lowerText.Contains("extra turn", StringComparison.Ordinal)) features["extra-turn"] = 1;
        if (lowerText.Contains("from your graveyard", StringComparison.Ordinal) || lowerText.Contains("from a graveyard", StringComparison.Ordinal)) features["graveyard"] = 1;
        if (lowerText.Contains("discard", StringComparison.Ordinal)) features["discard"] = 1;
        if (lowerText.Contains("without paying", StringComparison.Ordinal) || lowerText.Contains("rather than pay", StringComparison.Ordinal)) features["free"] = 1;
        if (lowerText.Contains("each opponent", StringComparison.Ordinal) || lowerText.Contains("target opponent", StringComparison.Ordinal)) features["opponent"] = 1;
        if (lowerText.Contains("gain", StringComparison.Ordinal) && lowerText.Contains("life", StringComparison.Ordinal)) features["lifegain"] = 1;
        if (lowerText.Contains("+1/+1 counter", StringComparison.Ordinal)) features["counters"] = 1;
        return features;
    }

    /// <summary>What a card is worth going by the card alone.</summary>
    public static double Estimate(DraftCard card, string lowerText)
    {
        double worth = 0;
        foreach ((string feature, double value) in Features(card, lowerText))
        {
            worth += value * Weights.GetValueOrDefault(feature);
        }

        return Math.Clamp(worth, 0.5, 6.0);
    }

    /// <summary>
    /// What each feature adds to a card's worth, before any pick order has been looked at: what a player would say of
    /// cards in general. A removal spell is better than a creature, a creature bigger than its cost better than one
    /// smaller, free mana better than nearly anything. The fit is held towards this, and it matters most where the
    /// fit has nothing to go on: a cube has no bad cards in it, so its pick order cannot say what one looks like.
    /// </summary>
    public static IReadOnlyDictionary<string, double> Prior { get; } = new Dictionary<string, double>(StringComparer.Ordinal)
    {
        ["bias"] = 2.6,
        ["mv"] = -0.05,
        ["text"] = 0.15,
        ["gold"] = -0.4,
        ["creature.body"] = 0.25,
        ["creature.evasive"] = 0.3,
        ["kw.flying"] = 0.1,
        ["kw.haste"] = 0.1,
        ["kw.lifelink"] = 0.1,
        ["kw.deathtouch"] = 0.1,
        ["kw.first strike"] = 0.1,
        ["kw.double strike"] = 0.3,
        ["kw.flash"] = 0.1,
        ["kw.hexproof"] = 0.1,
        ["type.Planeswalker"] = 0.8,
        ["removal"] = 0.6,
        ["removal.cheap"] = 0.2,
        ["sweeper"] = 0.5,
        ["counter"] = 0.3,
        ["advantage"] = 0.4,
        ["mana"] = 0.3,
        ["mana.cheap"] = 0.5,
        ["mana.free"] = 1.0,
        ["mana.two"] = 0.5,
        ["tutor"] = 0.2,
        ["enters"] = 0.2,
        ["tokens"] = 0.2,
        ["extra-turn"] = 0.8,
        ["land.fetch"] = 0.5,
        ["land.any"] = 0.3,
        ["land.typed"] = 0.5,
        ["land.tapped"] = -0.4,
        ["land.colorless"] = 0.2,
        ["land.text"] = 0.1,
    };

    /// <summary>
    /// What each feature adds to a card's worth. Fitted - a ridge regression of the cubes' pick orders on the features,
    /// held towards <see cref="Prior"/> (<c>cantrip-botlab draft fit</c>, which prints this table) - so where the pick
    /// orders say something these are what they say, and where they do not these are the prior.
    /// </summary>
    public static IReadOnlyDictionary<string, double> Weights { get; } = new Dictionary<string, double>(StringComparer.Ordinal)
    {
        ["adds"] = -0.096,
        ["advantage"] = 0.239,
        ["bias"] = 3.054,
        ["colorless"] = -0.017,
        ["counter"] = 0.459,
        ["counters"] = 0.006,
        ["creature"] = -0.023,
        ["creature.body"] = 0.250,
        ["creature.evasive"] = 0.216,
        ["creature.power"] = -0.073,
        ["discard"] = -0.017,
        ["draw"] = -0.015,
        ["enters"] = 0.216,
        ["equipment"] = -0.066,
        ["extra-turn"] = 0.891,
        ["free"] = 0.019,
        ["gold"] = -0.556,
        ["gold.3"] = -0.038,
        ["graveyard"] = -0.113,
        ["kw.deathtouch"] = 0.118,
        ["kw.double strike"] = 0.358,
        ["kw.first strike"] = 0.168,
        ["kw.flash"] = 0.189,
        ["kw.flying"] = 0.090,
        ["kw.haste"] = 0.094,
        ["kw.hexproof"] = 0.081,
        ["kw.lifelink"] = 0.315,
        ["kw.menace"] = -0.052,
        ["kw.trample"] = -0.185,
        ["kw.vigilance"] = -0.125,
        ["kw.ward"] = -0.103,
        ["land.any"] = 0.303,
        ["land.colorless"] = 0.378,
        ["land.dual"] = -0.221,
        ["land.fetch"] = 0.893,
        ["land.one"] = 0.028,
        ["land.tapped"] = -0.625,
        ["land.text"] = -0.149,
        ["land.typed"] = 0.511,
        ["lifegain"] = 0.092,
        ["mana"] = 0.155,
        ["mana.cheap"] = 0.589,
        ["mana.free"] = 1.155,
        ["mana.two"] = 0.602,
        ["mv"] = -0.009,
        ["mv.0"] = 0.263,
        ["mv.1"] = -0.118,
        ["mv.2"] = -0.177,
        ["mv.3"] = 0.006,
        ["mv.4"] = -0.053,
        ["mv.5"] = 0.025,
        ["mv.6"] = 0.066,
        ["opponent"] = 0.012,
        ["pips.2"] = -0.118,
        ["removal"] = 0.239,
        ["removal.cheap"] = -0.077,
        ["sweeper"] = 0.373,
        ["text"] = 0.000,
        ["tokens"] = 0.318,
        ["tutor"] = 0.056,
        ["type.Artifact"] = -0.054,
        ["type.Enchantment"] = 0.045,
        ["type.Instant"] = 0.058,
        ["type.Planeswalker"] = 0.669,
        ["type.Sorcery"] = 0.093,
        ["vehicle"] = -0.032,
        ["x"] = -0.064,
    };
}
