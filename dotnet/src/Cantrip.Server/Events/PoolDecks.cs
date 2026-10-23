using Cantrip.Cards.Model;
using Cantrip.Datasets;
using Cantrip.Decks;
using Cantrip.Protocol;
using Cantrip.Server.Decks;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// The rules of a deck built from a pool, however the pool was drafted: the main deck names cards of the pool by their
/// place in it, and adds as many basic lands as it likes. Shared by every event that builds decks, and by sideboarding
/// between the games of a match.
/// </summary>
public static class PoolDecks
{
    /// <summary>The basic lands a deck may add as many of as it likes (CR 100.2a).</summary>
    public static readonly IReadOnlyList<string> BasicLands = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

    /// <summary>The basic lands by name, each as its card.</summary>
    public static Dictionary<string, CardId> Basics(ContentLibrary content)
    {
        ArgumentNullException.ThrowIfNull(content);
        return BasicLands.ToDictionary(name => name, name => content.Cards.InOrder().FirstOrDefault(card => string.Equals(card.Name, name, StringComparison.Ordinal))?.Id
            ?? throw new InvalidOperationException($"No loaded card is the basic land {name}."), StringComparer.Ordinal);
    }

    /// <summary>
    /// A deck as a seat sent it, checked against its pool; one submitted - for the event, or for the next game of a match
    /// after sideboarding - is checked too by <paramref name="check"/>, the server's check of the deck for the format
    /// (<see cref="Refusal"/>), and refused if it finds anything wrong.
    /// </summary>
    /// <returns>The deck, or why it cannot be.</returns>
    public static (EventDeck? Deck, string? Refusal) Built(EventDeck current, int poolSize, BuildDeckMessage build, IReadOnlyDictionary<string, CardId> basics, int minimum,
        Func<EventDeck, string?> check)
    {
        ArgumentNullException.ThrowIfNull(check);
        ArgumentNullException.ThrowIfNull(current);
        ArgumentNullException.ThrowIfNull(build);
        ArgumentNullException.ThrowIfNull(basics);

        if (build.Main is null || build.Basics is null
            || build.Main.Any(place => place < 0 || place >= poolSize)
            || build.Main.Distinct().Count() != build.Main.Count)
        {
            return (null, "The main deck names cards that are not in your pool.");
        }

        Dictionary<string, int> lands = new(StringComparer.Ordinal);
        foreach (BasicLandCount land in build.Basics)
        {
            if (land is null || !basics.ContainsKey(land.Name) || land.Count is < 0 or > 99 || lands.ContainsKey(land.Name))
            {
                return (null, "Add up to 99 of each basic land: Plains, Island, Swamp, Mountain and Forest.");
            }

            if (land.Count > 0)
            {
                lands[land.Name] = land.Count;
            }
        }

        // The companion is a card of the sideboard (CR 400.11a): of the pool, and not in the main deck.
        if (build.Companion != -1 && (build.Companion < 0 || build.Companion >= poolSize || build.Main.Contains(build.Companion)))
        {
            return (null, "The companion is a card of your sideboard.");
        }

        int size = build.Main.Count + lands.Values.Sum();
        if (build.Submit && size < minimum)
        {
            return (null, $"A deck needs at least {minimum} cards; this one has {size}.");
        }

        var deck = new EventDeck { Main = [.. build.Main.Order()], Basics = lands, Companion = build.Companion, Submitted = build.Submit, Revision = current.Revision + 1 };
        return build.Submit && check(deck) is { } refusal ? (null, refusal) : (deck, null);
    }

    /// <summary>
    /// What is wrong with a deck built from a pool, for Limited with the event's minimum (CR 100.2b): the server's check
    /// (<see cref="DeckValidation"/>), its companion's condition judged by the rules engine. Null for a deck it deals.
    /// </summary>
    /// <param name="nameOf">A card as the rules name it.</param>
    /// <param name="card">A card of the pool, by its id, as printed.</param>
    public static string? Refusal(IReadOnlyList<string> pool, EventDeck deck, int minimum, IReadOnlyDictionary<string, CardId> basics,
        Func<CardId, string> nameOf, Func<string, ObjectView> card, IRulesHostFactory rules)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(basics);
        ArgumentNullException.ThrowIfNull(nameOf);
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(rules);
        static List<DeckEntry> Entries(IEnumerable<string> names) =>
            [.. names.GroupBy(name => name, StringComparer.Ordinal).Select(group => new DeckEntry(group.Key, group.Count()))];
        var saved = new SavedDeck
        {
            Format = DeckFormat.LimitedKey,
            Main = Entries(Cards(pool, deck, basics).Select(nameOf)),
            Side = Entries(Sideboard(pool, deck).Select(nameOf)),
            Companion = deck.Companion >= 0 && deck.Companion < pool.Count ? nameOf(new CardId(pool[deck.Companion])) : string.Empty,
        };
        Dictionary<string, CatalogCard> cards = new(StringComparer.Ordinal);
        foreach (CardId id in pool.Select(each => new CardId(each)).Concat(basics.Values))
        {
            cards.TryAdd(nameOf(id), new CatalogCard { Card = card(id.Value) });
        }

        return DeckValidation.Problems(saved, DeckFormat.Limited(minimum), cards, rules).FirstOrDefault();
    }

    /// <summary>The cards of a deck, the pool's first and then the basic lands.</summary>
    public static List<CardId> Cards(IReadOnlyList<string> pool, EventDeck deck, IReadOnlyDictionary<string, CardId> basics)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(basics);
        return
        [
            .. deck.Main.Select(place => new CardId(pool[place])),
            .. BasicLands.SelectMany(name => Enumerable.Repeat(basics[name], deck.Basics.GetValueOrDefault(name))),
        ];
    }

    /// <summary>The pool's cards neither in the main deck nor its companion.</summary>
    public static List<CardId> Sideboard(IReadOnlyList<string> pool, EventDeck deck)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        return [.. pool.Where((_, place) => !deck.Main.Contains(place) && place != deck.Companion).Select(id => new CardId(id))];
    }

    /// <summary>The deck's companion (CR 702.139a), as a card id, or null for none.</summary>
    public static CardId? Companion(IReadOnlyList<string> pool, EventDeck deck)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        return deck.Companion >= 0 && deck.Companion < pool.Count ? new CardId(pool[deck.Companion]) : null;
    }

    /// <summary>
    /// The deck as it stands, submitted for a player whose time ran out: a deck short of the minimum is made up with
    /// basic lands of the colours of its cards, the most of the colour it has most of.
    /// </summary>
    public static EventDeck Submitted(IReadOnlyList<string> pool, EventDeck deck, int minimum, Func<string, IReadOnlyList<ColorView>> colorsOf)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        ArgumentNullException.ThrowIfNull(colorsOf);
        Dictionary<string, int> basics = new(deck.Basics, StringComparer.Ordinal);
        int short_ = minimum - deck.Main.Count - basics.Values.Sum();
        if (short_ > 0)
        {
            (ColorView Color, string Land)[] lands =
            [
                (ColorView.White, "Plains"), (ColorView.Blue, "Island"), (ColorView.Black, "Swamp"), (ColorView.Red, "Mountain"), (ColorView.Green, "Forest"),
            ];
            Dictionary<string, int> weights = lands.ToDictionary(
                land => land.Land,
                land => deck.Main.Count(place => colorsOf(pool[place]).Contains(land.Color)),
                StringComparer.Ordinal);
            if (weights.Values.Sum() == 0)
            {
                weights["Plains"] = 1;
            }

            int total = weights.Values.Sum();
            int given = 0;
            foreach ((string land, int weight) in weights.Where(pair => pair.Value > 0))
            {
                int count = short_ * weight / total;
                basics[land] = basics.GetValueOrDefault(land) + count;
                given += count;
            }

            string most = weights.OrderByDescending(pair => pair.Value).ThenBy(pair => pair.Key, StringComparer.Ordinal).First().Key;
            basics[most] = basics.GetValueOrDefault(most) + short_ - given;
        }

        return new EventDeck { Main = [.. deck.Main], Basics = basics, Companion = deck.Companion, Submitted = true, Revision = deck.Revision + 1 };
    }
}
