using System.Text.RegularExpressions;
using Cantrip.Protocol;

namespace Cantrip.Client.Decks;

/// <summary>
/// Whether a deck is legal for its format, and why not: the deck construction rules (CR 100.2a, 100.4a), each card's
/// place in the format (banned, restricted, not legal), and for Commander (CR 903) the commanders - one, or two that may
/// be paired (partner, partner with, friends forever, choose a Background, Doctor's companion) - a hundred cards, one of
/// each but the basic lands, and every card within the commanders' color identity.
/// <para>
/// A deck is checked as the player edits it, in the page, from the catalog alone: nothing here refuses a deck, it says
/// what is wrong, and a deck is saved however it stands.
/// </para>
/// </summary>
public static partial class DeckRules
{
    /// <summary>Everything wrong with a deck for its format, in the order a player would fix it; empty for a legal deck.</summary>
    /// <param name="cards">The catalog by card name; a card it does not have is one the engine does not play.</param>
    public static IReadOnlyList<string> Problems(SavedDeck deck, IReadOnlyDictionary<string, CatalogCard> cards)
    {
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(cards);
        List<string> problems = [];
        if (DeckFormat.Find(deck.Format) is not { } format)
        {
            problems.Add($"\"{deck.Format}\" is not a format a deck is built for.");
            return problems;
        }

        int main = deck.Main.Sum(entry => entry.Count);
        int side = deck.Side.Sum(entry => entry.Count);
        int commanders = deck.Commanders.Sum(entry => entry.Count);

        if (format.IsCommander)
        {
            problems.AddRange(CommanderProblems(deck, cards));
            if (main + commanders != format.MainSize)
            {
                problems.Add($"A Commander deck is exactly {format.MainSize} cards with its commander: this one is {main + commanders}.");
            }

            if (side > 0)
            {
                problems.Add("A Commander deck has no sideboard.");
            }
        }
        else
        {
            if (commanders > 0)
            {
                problems.Add($"Only a Commander deck has a commander.");
            }

            if (main < format.MainSize)
            {
                problems.Add($"A {format.Title} deck is at least {format.MainSize} cards: this one is {main}.");
            }

            if (side > format.SideSize)
            {
                problems.Add($"A sideboard is at most {format.SideSize} cards: this one is {side}.");
            }
        }

        // Each card by name, across the deck, the sideboard and the command zone.
        foreach (IGrouping<string, DeckEntry> named in deck.Commanders.Concat(deck.Main).Concat(deck.Side)
            .GroupBy(entry => entry.Card, StringComparer.Ordinal).OrderBy(group => group.Key, StringComparer.Ordinal))
        {
            int copies = named.Sum(entry => entry.Count);
            if (!cards.TryGetValue(named.Key, out CatalogCard? card))
            {
                problems.Add($"{named.Key} is not a card the engine plays.");
                continue;
            }

            string legality = card.LegalityIn(format.Key);
            if (legality == Legality.Banned)
            {
                problems.Add($"{named.Key} is banned in {format.Title}.");
            }
            else if (!Legality.Allows(legality))
            {
                problems.Add($"{named.Key} is not legal in {format.Title}.");
            }

            int most = legality == Legality.Restricted ? 1 : MostCopies(card, format);
            if (copies > most)
            {
                problems.Add(legality == Legality.Restricted
                    ? $"{named.Key} is restricted in {format.Title}: one copy at most, not {copies}."
                    : $"{named.Key}: {copies} copies, more than the {most} a {format.Title} deck may have.");
            }
        }

        return problems;
    }

    /// <summary>
    /// How many copies of a card a deck may have: any number of a basic land (CR 100.2a), of a card that says a deck
    /// can have any number of it, as many as one that says how many (Seven Dwarves); otherwise the format's limit.
    /// </summary>
    public static int MostCopies(CatalogCard card, DeckFormat format)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(format);
        if (card.Card.Supertypes.Contains(SupertypeView.Basic) || card.Card.OracleText.Contains("A deck can have any number of cards named", StringComparison.Ordinal))
        {
            return int.MaxValue;
        }

        Match upTo = UpTo().Match(card.Card.OracleText);
        if (upTo.Success && Number(upTo.Groups[1].Value) is { } number)
        {
            return number;
        }

        return format.Copies;
    }

    /// <summary>Whether a card can be a deck's commander on its own merits: a legendary creature, or one that says it can be (CR 903.3).</summary>
    public static bool CanBeCommander(CatalogCard card)
    {
        ArgumentNullException.ThrowIfNull(card);
        ObjectView face = card.Card;
        return (face.Supertypes.Contains(SupertypeView.Legendary) && face.CardTypes.Contains(CardTypeView.Creature))
            || face.OracleText.Contains("can be your commander", StringComparison.Ordinal);
    }

    /// <summary>Whether a card is a Background (CR 702.124k): a legendary enchantment, the second commander of one that chooses a Background.</summary>
    public static bool IsBackground(CatalogCard card)
    {
        ArgumentNullException.ThrowIfNull(card);
        ObjectView face = card.Card;
        return face.Supertypes.Contains(SupertypeView.Legendary) && face.CardTypes.Contains(CardTypeView.Enchantment)
            && face.Subtypes.Contains("Background", StringComparer.Ordinal);
    }

    /// <summary>
    /// Whether a card can go in the command zone of a deck whose commanders are already <paramref name="others"/>: alone,
    /// one that can be a commander; beside another, one the pairing rules let the two be together.
    /// </summary>
    public static bool CanJoinCommanders(CatalogCard card, IReadOnlyList<CatalogCard> others)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(others);
        return others.Count switch
        {
            0 => CanBeCommander(card) || IsBackground(card),
            1 => CanPair(others[0], card),
            _ => false,
        };
    }

    /// <summary>
    /// Whether two cards can be a deck's commanders together (CR 702.124): both with partner, each partnered with the
    /// other by name, both with the same partner kind (Partner - Survivors) or friends forever, one that chooses a
    /// Background with a Background, or a Doctor's companion with a legendary Time Lord Doctor.
    /// </summary>
    public static bool CanPair(CatalogCard a, CatalogCard b)
    {
        ArgumentNullException.ThrowIfNull(a);
        ArgumentNullException.ThrowIfNull(b);
        if (string.Equals(a.Name, b.Name, StringComparison.Ordinal))
        {
            return false;
        }

        Pairing first = PairingOf(a), second = PairingOf(b);
        if (first.ChoosesBackground && IsBackground(b) && CanBeCommander(a)) return true;
        if (second.ChoosesBackground && IsBackground(a) && CanBeCommander(b)) return true;
        if (!CanBeCommander(a) || !CanBeCommander(b)) return false;
        if (first.Partner && second.Partner) return true;
        if (first.PartnerWith == b.Name && second.PartnerWith == a.Name) return true;
        if (first.Kind is { } kind && kind == second.Kind) return true;
        if (first.FriendsForever && second.FriendsForever) return true;
        if (first.Companion && IsDoctor(b)) return true;
        return second.Companion && IsDoctor(a);
    }

    /// <summary>The colors a deck's cards are held to: every color in its commanders' identities (CR 903.4).</summary>
    public static HashSet<ColorView> IdentityOf(IEnumerable<CatalogCard> commanders)
    {
        ArgumentNullException.ThrowIfNull(commanders);
        return [.. commanders.SelectMany(commander => commander.Identity)];
    }

    /// <summary>Whether a card fits a color identity: each of its own colors is among them (CR 903.5c).</summary>
    public static bool Within(CatalogCard card, IReadOnlySet<ColorView> identity)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(identity);
        return card.Identity.All(identity.Contains);
    }

    /// <summary>Whether a card may be put in a deck of a format, by its legality there and, for Commander with its commanders chosen, by their identity.</summary>
    public static bool Fits(CatalogCard card, DeckFormat format, IReadOnlySet<ColorView>? identity)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(format);
        return Legality.Allows(card.LegalityIn(format.Key)) && (!format.IsCommander || identity is null || Within(card, identity));
    }

    private static IEnumerable<string> CommanderProblems(SavedDeck deck, IReadOnlyDictionary<string, CatalogCard> cards)
    {
        List<CatalogCard> commanders = [.. deck.Commanders.SelectMany(entry => Enumerable.Repeat(entry.Card, entry.Count))
            .Select(name => cards.GetValueOrDefault(name)).OfType<CatalogCard>()];
        int named = deck.Commanders.Sum(entry => entry.Count);
        if (named == 0)
        {
            yield return "A Commander deck needs a commander.";
            yield break;
        }

        if (named > 2)
        {
            yield return $"A deck has one commander, or two that may be paired: this one has {named}.";
            yield break;
        }

        if (commanders.Count != named)
        {
            // A card the engine does not play is said once, with the rest of the deck's cards.
            yield break;
        }

        if (commanders.Count == 1 && !CanBeCommander(commanders[0]))
        {
            yield return IsBackground(commanders[0])
                ? $"{commanders[0].Name} is a Background: it is a commander only beside one that says \"Choose a Background\"."
                : $"{commanders[0].Name} can't be a commander: it is not a legendary creature.";
        }
        else if (commanders.Count == 2 && !CanPair(commanders[0], commanders[1]))
        {
            yield return $"{commanders[0].Name} and {commanders[1].Name} can't be commanders together: neither pairs with the other (partner, partner with, friends forever, choose a Background).";
        }

        HashSet<ColorView> identity = IdentityOf(commanders);
        foreach (string outside in deck.Main.Concat(deck.Side).Select(entry => entry.Card).Distinct(StringComparer.Ordinal)
            .Where(name => cards.TryGetValue(name, out CatalogCard? card) && !Within(card, identity)).Order(StringComparer.Ordinal))
        {
            yield return $"{outside} is outside the commander's color identity.";
        }
    }

    /// <summary>What a card's keyword lines say about who it can be a commander with.</summary>
    private sealed record Pairing(bool Partner, string? PartnerWith, string? Kind, bool FriendsForever, bool ChoosesBackground, bool Companion);

    private static Pairing PairingOf(CatalogCard card)
    {
        bool partner = false, friends = false, background = false, companion = false;
        string? with = null, kind = null;
        foreach (string raw in card.Card.OracleText.Split('\n'))
        {
            string line = Reminder().Replace(raw, string.Empty).Trim();
            if (line.StartsWith("Partner with ", StringComparison.Ordinal))
            {
                with = line["Partner with ".Length..].Trim();
                continue;
            }

            Match kinded = PartnerKind().Match(line);
            if (kinded.Success)
            {
                kind = kinded.Groups[1].Value.Trim();
                continue;
            }

            foreach (string keyword in line.Split(',', StringSplitOptions.TrimEntries | StringSplitOptions.RemoveEmptyEntries))
            {
                switch (keyword.ToLowerInvariant())
                {
                    case "partner": partner = true; break;
                    case "friends forever": friends = true; break;
                    case "choose a background": background = true; break;
                    case "doctor's companion": companion = true; break;
                }
            }
        }

        return new Pairing(partner, with, kind, friends, background, companion);
    }

    /// <summary>A legendary Time Lord Doctor creature with no other creature types: who a Doctor's companion may be beside (CR 702.124m).</summary>
    private static bool IsDoctor(CatalogCard card) =>
        card.Card.Supertypes.Contains(SupertypeView.Legendary) && card.Card.CardTypes.Contains(CardTypeView.Creature)
        && card.Card.Subtypes.Count == 2 && card.Card.Subtypes.Contains("Time Lord", StringComparer.Ordinal) && card.Card.Subtypes.Contains("Doctor", StringComparer.Ordinal);

    private static int? Number(string word) => int.TryParse(word, out int digits) ? digits : word.ToLowerInvariant() switch
    {
        "one" => 1, "two" => 2, "three" => 3, "four" => 4, "five" => 5, "six" => 6, "seven" => 7, "eight" => 8, "nine" => 9, "ten" => 10,
        _ => null,
    };

    [GeneratedRegex(@"A deck can have up to (\w+) cards named", RegexOptions.CultureInvariant)]
    private static partial Regex UpTo();

    [GeneratedRegex(@"\([^)]*\)", RegexOptions.CultureInvariant)]
    private static partial Regex Reminder();

    [GeneratedRegex(@"^Partner\s*[—-]\s*(.+)$", RegexOptions.CultureInvariant)]
    private static partial Regex PartnerKind();
}
