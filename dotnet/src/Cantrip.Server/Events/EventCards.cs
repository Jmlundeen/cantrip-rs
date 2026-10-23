using Cantrip.Cards.Model;
using Cantrip.Datasets;
using Cantrip.Projection;
using Cantrip.Protocol;
using Cantrip.Server.Rules;

namespace Cantrip.Server.Events;

/// <summary>
/// The cards an event's pools are made of, as its tables show them and deal them: each card as the client draws it, the
/// basic lands anyone may add, and a game dealt from the players' pools and decks. Shared by every event that builds
/// decks from pools - a booster pod, a league - so each shows a deck and deals a game the same way.
/// </summary>
internal sealed class EventCards
{
    private readonly CardOracle _oracle;

    public EventCards(ContentLibrary content)
    {
        ArgumentNullException.ThrowIfNull(content);
        Catalog = content.Catalog();
        _oracle = new CardOracle(Catalog);
        Basics = PoolDecks.Basics(content);
    }

    public CardCatalog Catalog { get; }

    /// <summary>The basic lands by name, which nobody drafts and anyone may add as many of as they like.</summary>
    public IReadOnlyDictionary<string, CardId> Basics { get; }

    /// <summary>A card as the client draws it; a card the content no longer has, by its id alone.</summary>
    public ObjectView Card(string id) =>
        _oracle.Oracle(new CardId(id)) ?? new ObjectView { Known = true, CardId = id, Name = id };

    public IReadOnlyList<ColorView> ColorsOf(string id) => Card(id).Colors;

    /// <summary>
    /// A pool's card as the rules name it: its id for the engine, which is its name for every card the content has
    /// (CR 201.2); the loaded card's name, which is how the native engine knows cards, or the id as it stands
    /// for one the content no longer has, which the host then refuses by name.
    /// </summary>
    public string NameOf(CardId card) =>
        Catalog.TryGet(card, out CardDefinition? loaded) ? loaded!.Name : card.Value;

    /// <summary>
    /// Deals one game on the rules engine asked for: each seat's deck from its pool, named card by card, the rest of its
    /// pool as its sideboard, and its companion (CR 702.139a), checked as the deck was submitted. A deck the server submitted
    /// for a player whose time ran out was not checked by them: a companion whose condition it doesn't fulfil is dealt all
    /// the same, and the engine leaves it a sideboard card, unrevealed (CR 103.2b).
    /// </summary>
    public IRulesHost NewGame(IReadOnlyList<(string Name, IReadOnlyList<string> Pool, EventDeck Deck)> seats, ulong seed, int minimum, IRulesHostFactory? rules = null)
    {
        ArgumentNullException.ThrowIfNull(seats);
        rules ??= RulesHostFactory.Native;
        List<RulesSeat> dealt = [.. seats.Select(seat => new RulesSeat(
            seat.Name,
            [.. PoolDecks.Cards(seat.Pool, seat.Deck, Basics).Select(NameOf)],
            [.. PoolDecks.Sideboard(seat.Pool, seat.Deck).Select(NameOf)],
            PoolDecks.Companion(seat.Pool, seat.Deck) is { } companion ? NameOf(companion) : null))];

        // A Limited sideboard is the rest of the pool and any number of basic lands (CR 100.4b; Magic Tournament Rules).
        return rules.Create(dealt, seed, new RulesGameOptions
        {
            BasicLandsOutside = [.. PoolDecks.BasicLands.Select(name => NameOf(Basics[name]))],
            MinimumDeckSize = minimum,
        });
    }

    /// <summary>The server's check of a deck built from a pool (<see cref="PoolDecks.Refusal"/>); null for a deck it deals.</summary>
    public string? Refusal(IReadOnlyList<string> pool, EventDeck deck, int minimum, IRulesHostFactory? rules = null) =>
        PoolDecks.Refusal(pool, deck, minimum, Basics, NameOf, Card, rules ?? RulesHostFactory.Native);

    /// <summary>The deck being built from a pool, as the deck builder shows it.</summary>
    public DeckBuildingMessage Deck(string eventId, IReadOnlyList<string> pool, EventDeck deck, int minimum, int change, DateTimeOffset? deadline, DateTimeOffset now, string title, List<DeckBuilderSeatView> seats) => new()
    {
        EventId = eventId,
        Pool = [.. pool.Select(Card)],
        Main = [.. deck.Main],
        Companion = deck.Companion,
        Basics = [.. PoolDecks.BasicLands.Select(name => new BasicLandView { Name = name, Count = deck.Basics.GetValueOrDefault(name), Card = Card(Basics[name].Value) })],
        Minimum = minimum,
        Submitted = deck.Submitted,
        Change = change,
        Seats = seats,
        LeftMs = deadline is { } until ? Math.Max(1, (int)Math.Min(int.MaxValue, Math.Max(0, (until - now).TotalMilliseconds))) : 0,
        Title = title,
    };

    public static EventDeck Copy(EventDeck deck) => new() { Main = [.. deck.Main], Basics = new(deck.Basics, StringComparer.Ordinal), Companion = deck.Companion, Submitted = deck.Submitted, Revision = deck.Revision };
}
