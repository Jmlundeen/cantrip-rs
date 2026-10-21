namespace Cantrip.Cards.Model;

/// <summary>
/// A card's identity, which is its English name: <c>Grizzly Bears</c>, <c>Kolaghan's Command</c>.
/// <para>
/// Magic's own rules make the name the identity: two cards with the same English name are the same card
/// mechanically, whichever set printed them (CR 201.2). So content is addressed by name and nothing else — not by
/// the dataset a card is defined in, and not by a set that prints it. A set that specialises a card changes what is
/// read under that name; it never makes a second card.
/// </para>
/// </summary>
public readonly record struct CardId(string Value) : IComparable<CardId>
{
    public static readonly CardId None = new(string.Empty);

    public bool IsNone => string.IsNullOrEmpty(Value);

    public int CompareTo(CardId other) => string.CompareOrdinal(Value, other.Value);

    public override string ToString() => Value;
}

/// <summary>
/// A card's printed characteristics (CR 109.3), as its card file prints them and nothing more.
/// <para>
/// What a card does is not here: the rules are the native engine's (<c>crates/</c>), which reads the same
/// files for itself. This is what drafting, deck building and a card list need: a card as it is printed.
/// </para>
/// </summary>
public sealed class CardDefinition
{
    public CardId Id;

    public string Name = string.Empty;

    /// <summary>The printed cost. No cost at all unless one is given, which is right for lands and tokens (CR 202.1b).</summary>
    public ManaCost ManaCost = ManaCost.None();

    public TypeLine TypeLine = new();

    /// <summary>
    /// Printed power, or null when the card has none. Null rather than zero because "no power" and "power 0" are
    /// different. A characteristic-defining power (a <c>*/*</c> creature) is also null here.
    /// </summary>
    public int? Power;

    public int? Toughness;

    /// <summary>Printed loyalty (CR 306.5a), or null for anything but a planeswalker.</summary>
    public int? Loyalty;

    /// <summary>Printed defense (CR 310.4a), or null for anything but a battle.</summary>
    public int? Defense;

    /// <summary>
    /// The power and toughness box a station card prints in its striation (CR 721.2b), "2/2" (<c>stationPt</c> in a card
    /// file). Display only: the card has no power or toughness until it has the charge counters (CR 721.2c), so
    /// <see cref="Power"/> and <see cref="Toughness"/> stay null. Empty for any other card.
    /// </summary>
    public string StationPt = string.Empty;

    /// <summary>Rules text as printed. Display only: nothing here parses it.</summary>
    public string OracleText = string.Empty;

    /// <summary>
    /// Colors the object has whatever its mana cost says (CR 202.2, 204): the back face of a double-faced card, whose
    /// color indicator is printed (<c>colorIndicator</c> in a card file). Empty for most cards, whose colors come from
    /// their cost.
    /// </summary>
    public List<ManaColor> ColorIndicator = [];

    /// <summary>
    /// The other face of a nonmodal double-faced card (CR 712.2), written as a card of its own under the same id, or
    /// null for a card with one face. A back face has no mana cost of its own: its mana value is the front face's.
    /// </summary>
    public CardDefinition? BackFace;

    /// <summary>A modal double-faced card whose land back face may be played (CR 712.4).</summary>
    public bool BackFacePlayableAsLand;

    /// <summary>The Adventure of an adventurer card (CR 715.2), written as a card of its own under this card's id, or null.</summary>
    public CardDefinition? Adventure;

    /// <summary>The Omen of an omen card (CR 720.2), written as a card of its own under this card's id, or null.</summary>
    public CardDefinition? Omen;

    /// <summary>
    /// The halves of a split card (CR 709.1), each written as a card of its own under this card's id, in printed order.
    /// Empty for every other card. This definition's own name, mana cost and type line are the halves' combined, which
    /// is what the card is in every zone but the stack (CR 709.4, 202.3b).
    /// </summary>
    public List<CardDefinition> Halves = [];

    /// <summary>The prepare spell of a preparation card (CR 722.2a), written as a card of its own under this card's id, or null.</summary>
    public CardDefinition? Prepare;

    /// <summary>
    /// What this card does while a booster pack is being drafted (CR 905.2), in printed order: abilities of the draft,
    /// which the draft carries out, and which do nothing in a game. Empty for every card but a few of Conspiracy's.
    /// </summary>
    public List<DraftAbility> DraftAbilities = [];

    public bool IsLegendary => TypeLine.Has(Supertype.Legendary);

    public bool IsCreature => TypeLine.Is(CardType.Creature);

    public bool IsLand => TypeLine.Is(CardType.Land);

    public bool IsPlaneswalker => TypeLine.Is(CardType.Planeswalker);

    /// <summary>Whether this is a battle (CR 310).</summary>
    public bool IsBattle => TypeLine.Is(CardType.Battle);

    public bool IsBasicLand => IsLand && TypeLine.Has(Supertype.Basic);

    public override string ToString() =>
        IsCreature ? $"{Name} ({TypeLine}) {Power}/{Toughness}" : $"{Name} ({TypeLine})";
}
