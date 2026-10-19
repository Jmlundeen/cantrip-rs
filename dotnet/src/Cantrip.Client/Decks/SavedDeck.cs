using System.Text.Json.Serialization;

namespace Cantrip.Client.Decks;

/// <summary>Some copies of a card, by the name it is printed with, as a decklist names them.</summary>
public sealed record DeckEntry(string Card, int Count);

/// <summary>
/// A constructed deck a player has saved: its name, the format it is built for, its commanders (Commander only), its
/// main deck and its sideboard. Kept as it is, legal or not - saving is never refused for what is in it.
/// </summary>
public sealed class SavedDeck
{
    /// <summary>The deck's id among its player's decks; empty for one not saved yet, which saving gives one.</summary>
    public string Id { get; set; } = string.Empty;

    public string Name { get; set; } = string.Empty;

    /// <summary>The format's key (<see cref="DeckFormat"/>).</summary>
    public string Format { get; set; } = DeckFormat.StandardKey;

    public List<DeckEntry> Commanders { get; set; } = [];

    public List<DeckEntry> Main { get; set; } = [];

    public List<DeckEntry> Side { get; set; } = [];

    /// <summary>When it was last saved; the server's to set.</summary>
    public DateTimeOffset Updated { get; set; }
}

/// <summary>A saved deck as the list of a player's decks shows it.</summary>
public sealed record DeckSummary(string Id, string Name, string Format, int Cards, DateTimeOffset Updated);

/// <summary>What saving a deck answers: the deck as kept (with its id), or why it was not.</summary>
public sealed record DeckSaved(SavedDeck? Deck, string? Refusal);

/// <summary>
/// The deck editor's JSON, generated rather than reflected, so trimming the web client's release build cannot strip what
/// a deck is read into; and the same on the server, so both sides read and write one shape.
/// </summary>
[JsonSourceGenerationOptions(PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase)]
[JsonSerializable(typeof(SavedDeck))]
[JsonSerializable(typeof(List<DeckSummary>))]
[JsonSerializable(typeof(DeckSaved))]
public sealed partial class DeckJson : JsonSerializerContext;
