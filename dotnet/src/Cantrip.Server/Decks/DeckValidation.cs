using Cantrip.Decks;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Decks;

/// <summary>
/// The server's check of a deck before it deals a game with it: the format's rules (<see cref="DeckRules"/>: its minimum and
/// any exact size, the sideboard's size, each card's legality and how many copies it may have), and the deck's companion
/// (CR 702.139a), whose condition the rules engine judges against this game's starting deck. The deck editor shows the same
/// check; this one is the one that counts. A deck it finds anything wrong with is not dealt.
/// </summary>
public static class DeckValidation
{
    /// <summary>Everything wrong with a deck for a format; empty for a deck the server deals.</summary>
    /// <param name="cards">Every card the engine plays, by name, with its legality (<see cref="DeckCatalog"/>).</param>
    /// <param name="rules">The rules engine that judges the companion's condition.</param>
    public static IReadOnlyList<string> Problems(SavedDeck deck, DeckFormat format, IReadOnlyDictionary<string, CatalogCard> cards, IRulesHostFactory rules) =>
        Check(deck, format, cards, rules).Problems;

    /// <summary>
    /// Everything wrong with a deck for a format, and whether the rules engine failed to answer for its companion: a
    /// sidecar gone, a pipe broken. Such a deck is refused - its companion unjudged - with a reason saying so, and the
    /// refusal is not one to remember, for the next ask may be answered.
    /// </summary>
    public static (IReadOnlyList<string> Problems, bool Unanswered) Check(SavedDeck deck, DeckFormat format, IReadOnlyDictionary<string, CatalogCard> cards, IRulesHostFactory rules)
    {
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(format);
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(rules);
        try
        {
            return (DeckRules.Problems(deck, format, cards, CompanionFulfilled(deck, format, cards, rules)), false);
        }
        catch (Exception error) when (error is not (OutOfMemoryException or ArgumentException))
        {
            List<string> problems = [.. DeckRules.Problems(deck, format, cards)];
            problems.Add($"The deck's companion could not be checked: the rules engine did not answer ({error.Message}). Try again.");
            return (problems, true);
        }
    }

    /// <summary>
    /// Whether the deck fulfils its companion's condition, as the rules engine judges it (CR 702.139a-b), the format's minimum
    /// deck size being the one a condition names; null where there is nothing to judge - no companion, a card without
    /// companion, or a deck naming a card the engine does not play, which is a problem of its own.
    /// </summary>
    public static bool? CompanionFulfilled(SavedDeck deck, DeckFormat format, IReadOnlyDictionary<string, CatalogCard> cards, IRulesHostFactory rules)
    {
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(format);
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(rules);
        string companion = deck.Companion?.Trim() ?? string.Empty;
        IReadOnlyList<string> starting = DeckRules.StartingDeck(deck);
        if (companion.Length == 0 || !cards.TryGetValue(companion, out CatalogCard? card) || !DeckRules.HasCompanion(card) || !starting.All(cards.ContainsKey))
        {
            return null;
        }

        try
        {
            return rules.CompanionFulfilled(starting, companion, format.MinimumDeckSize);
        }
        catch (RulesRejectedException)
        {
            return null;
        }
    }

    /// <summary>A decklist file's deck as a deck of a format: its cards, its sideboard and its companion.</summary>
    public static SavedDeck Of(DeckList list, DeckFormat format)
    {
        ArgumentNullException.ThrowIfNull(list);
        ArgumentNullException.ThrowIfNull(format);
        return new SavedDeck
        {
            Name = list.Name,
            Format = format.Key,
            Main = [.. list.Entries.Select(entry => new DeckEntry(entry.Card, entry.Copies))],
            Side = [.. list.Sideboard.Select(entry => new DeckEntry(entry.Card, entry.Copies))],
            Companion = list.Companion ?? string.Empty,
        };
    }
}
