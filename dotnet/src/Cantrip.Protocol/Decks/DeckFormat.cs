namespace Cantrip.Decks;

/// <summary>
/// A format a deck is built for: a constructed one by its key in Scryfall's legalities (<c>vintage</c>, <c>penny</c>) and
/// how it is named - Commander the one with a command zone, every other sixty cards and a sideboard of fifteen - or Limited
/// (CR 100.2b), a deck built from a pool, with the event's minimum and no limit to copies or the sideboard - or Freeform, a
/// casual game with no deck-construction restriction at all, the default wherever no format is given.
/// </summary>
public sealed record DeckFormat(string Key, string Title)
{
    public const string LimitedKey = "limited";
    public const string FreeformKey = "freeform";

    /// <summary>
    /// Freeform: play whatever deck you like - no minimum or maximum size, no legality list, no limit to copies or to the
    /// sideboard. Only a companion is still held to its own condition (CR 702.139a), and a card the engine does not play
    /// can't be dealt. The default for a table, a tool or a scenario that names no format.
    /// </summary>
    public static DeckFormat Freeform { get; } = new(FreeformKey, "Freeform");

    public bool IsFreeform => Key == FreeformKey;

    /// <summary>Limited (CR 100.2b, 100.4b): at least <paramref name="minimum"/> cards, forty by the rules; the rest of the pool is the sideboard.</summary>
    public static DeckFormat Limited(int minimum = 40) => new(LimitedKey, "Limited") { LimitedMinimum = minimum };

    /// <summary>A Limited deck's fewest cards: the event's, forty by the rules (CR 100.2b).</summary>
    private int LimitedMinimum { get; init; } = 40;

    public bool IsLimited => Key == LimitedKey;

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
        Freeform,
    ];

    /// <summary>A format by its key, or null for one no deck is built for.</summary>
    public static DeckFormat? Find(string? key) =>
        string.Equals(key, LimitedKey, StringComparison.Ordinal) ? Limited() : All.FirstOrDefault(format => string.Equals(format.Key, key, StringComparison.Ordinal));

    public bool IsCommander => Key == CommanderKey;

    /// <summary>
    /// The fewest cards a main deck has (CR 100.2a): sixty, for Commander exactly a hundred with its commanders (CR 903.5a), in
    /// Limited the event's (CR 100.2b), and none in Freeform, which holds a deck to nothing.
    /// </summary>
    public int MainSize => IsFreeform ? 0 : IsCommander ? 100 : IsLimited ? LimitedMinimum : 60;

    /// <summary>
    /// The minimum deck size of the game (CR 100.2a-b, 903.5a), which the engine is told and a companion's condition may name
    /// (Yorion, Sky Nomad): the format's own - and none in Freeform, which has no deck size restriction at all (a deck of no
    /// cards is a Freeform deck), so Yorion's "twenty cards more than the minimum deck size" is twenty there.
    /// </summary>
    public int MinimumDeckSize => MainSize;

    /// <summary>The most cards a sideboard has (CR 100.4a); Commander has none, in Limited it is the rest of the pool (CR 100.4b), and Freeform has no limit.</summary>
    public int SideSize => IsCommander ? 0 : IsLimited || IsFreeform ? int.MaxValue : 15;

    /// <summary>
    /// The most copies of a card by its English name across deck and sideboard (CR 100.2a), or one for Commander (CR 903.5b);
    /// no limit in Limited, whose cards are the pool's (CR 100.2b), nor in Freeform.
    /// </summary>
    public int Copies => IsCommander ? 1 : IsLimited || IsFreeform ? int.MaxValue : 4;
}
