namespace Cantrip.Cards.Model;

/// <summary>
/// A collection of card definitions, keyed by card id — which is the card's name.
/// <para>
/// Content loads into one of these and catalogs compose into the registry the engine reads from. Definitions live
/// here, outside game state: state refers to a card by <see cref="CardId"/> and looks it up. The exception is a
/// token's synthetic definition, which is carried in state because no catalog contains it.
/// </para>
/// <para>
/// Not a set. A set, in the product sense, is a list of cards that already exist; a catalog is where cards exist.
/// </para>
/// </summary>
public sealed class CardCatalog
{
    public Dictionary<CardId, CardDefinition> Cards = [];

    public int Count => Cards.Count;

    public CardDefinition this[CardId id] =>
        Cards.TryGetValue(id, out CardDefinition? card)
            ? card
            : throw new KeyNotFoundException($"'{id}' is not in this catalog.");

    public bool TryGet(CardId id, out CardDefinition? card) => Cards.TryGetValue(id, out card);

    /// <summary>Definitions in card-id order, so iteration is deterministic.</summary>
    public IEnumerable<CardDefinition> InOrder()
    {
        List<CardId> ids = [.. Cards.Keys];
        ids.Sort();

        foreach (CardId id in ids)
        {
            yield return Cards[id];
        }
    }

    public override string ToString() => $"{Count} cards";
}
