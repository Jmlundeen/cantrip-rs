namespace Cantrip.Client.Decks;

/// <summary>
/// A constructed format a deck is built for: its key in Scryfall's legalities (<c>vintage</c>, <c>penny</c>) and how
/// it is named. Commander is the one with a command zone; every other is sixty cards and a sideboard of fifteen.
/// </summary>
public sealed record DeckFormat(string Key, string Title)
{
    public const string VintageKey = "vintage";
    public const string LegacyKey = "legacy";
    public const string ModernKey = "modern";
    public const string PioneerKey = "pioneer";
    public const string StandardKey = "standard";
    public const string CommanderKey = "commander";
    public const string PennyKey = "penny";

    /// <summary>Every format a deck can be built for, in the order a player is offered them.</summary>
    public static IReadOnlyList<DeckFormat> All { get; } =
    [
        new(StandardKey, "Standard"),
        new(PioneerKey, "Pioneer"),
        new(ModernKey, "Modern"),
        new(LegacyKey, "Legacy"),
        new(VintageKey, "Vintage"),
        new(CommanderKey, "Commander"),
        new(PennyKey, "Penny Dreadful"),
    ];

    /// <summary>A format by its key, or null for one no deck is built for.</summary>
    public static DeckFormat? Find(string? key) => All.FirstOrDefault(format => string.Equals(format.Key, key, StringComparison.Ordinal));

    public bool IsCommander => Key == CommanderKey;

    /// <summary>The fewest cards a main deck has (CR 100.2a): sixty, and for Commander exactly a hundred with its commanders (CR 903.5a).</summary>
    public int MainSize => IsCommander ? 100 : 60;

    /// <summary>The most cards a sideboard has (CR 100.4a); Commander has none.</summary>
    public int SideSize => IsCommander ? 0 : 15;

    /// <summary>The most copies of a card by its English name across deck and sideboard (CR 100.2a), or one for Commander (CR 903.5b).</summary>
    public int Copies => IsCommander ? 1 : 4;
}
