using Cantrip.Protocol;
using MessagePack;

namespace Cantrip.Client.Decks;

/// <summary>
/// Every card a constructed deck can be built from - every card the engine plays - as printed, with where it is legal
/// and its color identity, which the deck editor searches, filters and checks a deck against. Served once by the bot's
/// site, as MessagePack (the client's own serializer for the cards it draws), and kept by the page.
/// </summary>
[MessagePackObject]
public sealed class CardCatalogView
{
    /// <summary>When the legality data was taken from Scryfall, as it says (an ISO date), or empty when it was not.</summary>
    [Key(0)]
    public string LegalityDate { get; set; } = string.Empty;

    [Key(1)]
    public List<CatalogCard> Cards { get; set; } = [];
}

/// <summary>One card of the catalog.</summary>
[MessagePackObject]
public sealed class CatalogCard
{
    /// <summary>The card as printed, as a draft or a deck builder shows it, with its other face if it has one.</summary>
    [Key(0)]
    public ObjectView Card { get; set; } = new();

    /// <summary>Its color identity (CR 903.4), as Scryfall gives it: what a Commander deck's cards are held to.</summary>
    [Key(1)]
    public List<ColorView> Identity { get; set; } = [];

    /// <summary>
    /// Where it may be played, by format key (<see cref="DeckFormat"/>): <c>legal</c>, <c>restricted</c>,
    /// <c>banned</c> or <c>not_legal</c>. A format it has no entry for is one it is not legal in.
    /// </summary>
    [Key(2)]
    public Dictionary<string, string> Legalities { get; set; } = [];

    [IgnoreMember]
    public string Name => Card.Name;

    /// <summary>Its legality in a format: <c>not_legal</c> when the data does not say.</summary>
    public string LegalityIn(string format) => Legalities.GetValueOrDefault(format) ?? Legality.NotLegal;
}

/// <summary>The words Scryfall says a card's place in a format with.</summary>
public static class Legality
{
    public const string Legal = "legal";
    public const string Restricted = "restricted";
    public const string Banned = "banned";
    public const string NotLegal = "not_legal";

    /// <summary>Whether a card so placed may be in a deck at all.</summary>
    public static bool Allows(string legality) => legality is Legal or Restricted;
}
