using Cantrip.Cards.Model;
using Cantrip.Datasets;
using Cantrip.Protocol;

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

    /// <summary>A deck as a seat sent it, checked against its pool.</summary>
    /// <returns>The deck, or why it cannot be.</returns>
    public static (EventDeck? Deck, string? Refusal) Built(EventDeck current, int poolSize, BuildDeckMessage build, IReadOnlyDictionary<string, CardId> basics, int minimum)
    {
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

        int size = build.Main.Count + lands.Values.Sum();
        if (build.Submit && size < minimum)
        {
            return (null, $"A deck needs at least {minimum} cards; this one has {size}.");
        }

        return (new EventDeck { Main = [.. build.Main.Order()], Basics = lands, Submitted = build.Submit, Revision = current.Revision + 1 }, null);
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

    /// <summary>The pool's cards not in the main deck.</summary>
    public static List<CardId> Sideboard(IReadOnlyList<string> pool, EventDeck deck)
    {
        ArgumentNullException.ThrowIfNull(pool);
        ArgumentNullException.ThrowIfNull(deck);
        return [.. pool.Where((_, place) => !deck.Main.Contains(place)).Select(id => new CardId(id))];
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

        return new EventDeck { Main = [.. deck.Main], Basics = basics, Submitted = true, Revision = deck.Revision + 1 };
    }
}
