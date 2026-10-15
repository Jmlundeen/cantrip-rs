using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>How a list of cards the player is looking through is put in order.</summary>
public enum CardSort
{
    /// <summary>The order the cards came in, which for a library is the library's.</summary>
    None = 0,
    Name = 1,
    ManaValue = 2,
}

/// <summary>How a list of cards the player is looking through is split into sections.</summary>
public enum CardGrouping
{
    None = 0,
    Color = 1,
    Type = 2,
}

/// <summary>One section of an arranged list: its heading, and its cards in order. The heading is empty when ungrouped.</summary>
public sealed record CardGroup(string Heading, IReadOnlyList<ObjectView> Cards);

/// <summary>
/// Sorts and groups cards for a player looking through them, as a library search lets them.
/// <para>
/// Only ever presentation. What the server sent is not reordered, and a sort never decides anything: the cards are the
/// same cards whichever way they are shown. Both sorts are stable, so cards the sort cannot tell apart — two with the
/// same mana value — keep the order they came in, and "None" is exactly that order.
/// </para>
/// </summary>
public static class CardArrangement
{
    /// <summary>The sections, in the order they are shown, each with its cards sorted. Empty sections are left out.</summary>
    public static IReadOnlyList<CardGroup> Arrange(IEnumerable<ObjectView> cards, CardSort sort, CardGrouping grouping)
    {
        ArgumentNullException.ThrowIfNull(cards);

        List<ObjectView> sorted = [.. Sort(cards, sort)];

        if (grouping == CardGrouping.None)
        {
            return sorted.Count == 0 ? [] : [new CardGroup(string.Empty, sorted)];
        }

        Func<ObjectView, (int Order, string Heading)> key = grouping == CardGrouping.Color ? ColorGroup : TypeGroup;

        return
        [
            .. sorted
                .GroupBy(key)
                .OrderBy(group => group.Key.Order)
                .Select(group => new CardGroup(group.Key.Heading, [.. group])),
        ];
    }

    private static IEnumerable<ObjectView> Sort(IEnumerable<ObjectView> cards, CardSort sort) => sort switch
    {
        // OrderBy is stable, which is what keeps ties in the order they came in.
        CardSort.Name => cards.OrderBy(card => card.Name, StringComparer.OrdinalIgnoreCase),
        CardSort.ManaValue => cards.OrderBy(card => card.ManaValue),
        _ => cards,
    };

    /// <summary>
    /// A card's colour section: one per colour in the order the colour pie runs (CR 105.1), then the cards of more than
    /// one colour, then the colourless, which is where lands go.
    /// </summary>
    public static (int Order, string Heading) ColorGroup(ObjectView card)
    {
        ArgumentNullException.ThrowIfNull(card);

        return card.Colors.Distinct().ToList() switch
        {
            [] => (6, "Colorless"),
            [ColorView only] => ((int)only, only.ToString()),
            _ => (5, "Multicolor"),
        };
    }

    /// <summary>
    /// A card's type section. A card with several types goes under the one a player sorting a deck would file it by: a
    /// land is a land unless it is also a creature, and otherwise creatures come first, then the other permanents that
    /// act, then spells, then what is left.
    /// </summary>
    public static (int Order, string Heading) TypeGroup(ObjectView card)
    {
        ArgumentNullException.ThrowIfNull(card);

        List<CardTypeView> types = card.CardTypes;

        if (types.Contains(CardTypeView.Land) && !types.Contains(CardTypeView.Creature))
        {
            return (7, "Lands");
        }

        return types switch
        {
            _ when types.Contains(CardTypeView.Creature) => (0, "Creatures"),
            _ when types.Contains(CardTypeView.Planeswalker) => (1, "Planeswalkers"),
            _ when types.Contains(CardTypeView.Battle) => (2, "Battles"),
            _ when types.Contains(CardTypeView.Instant) => (3, "Instants"),
            _ when types.Contains(CardTypeView.Sorcery) => (4, "Sorceries"),
            _ when types.Contains(CardTypeView.Artifact) => (5, "Artifacts"),
            _ when types.Contains(CardTypeView.Enchantment) => (6, "Enchantments"),
            _ => (8, "Other"),
        };
    }
}
