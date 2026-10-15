using System.Text;

namespace Cantrip.Cards.Model;

/// <summary>The five colors (CR 105.1). Colorless is the absence of color, not a sixth value.</summary>
public enum ManaColor
{
    White = 0,
    Blue = 1,
    Black = 2,
    Red = 3,
    Green = 4,
}

/// <summary>
/// A printed mana cost.
/// <para>
/// Deliberately structured rather than a string, because cost is something the engine reasons
/// about: paying it is a constraint-solving problem once hybrid symbols, filter lands, and
/// CR 601.2e-f cost reduction exist. The oracle string is kept alongside for display and for
/// round-tripping imported data, but it is never the source of truth.
/// </para>
/// <para>
/// Generic, single-color, colorless, two-color hybrid, Phyrexian, and {X} symbols are modelled so far. Monocolored
/// hybrid and snow slot in as additional symbol kinds without disturbing callers that ask for
/// <see cref="ManaValue"/> or <see cref="Colors"/>.
/// </para>
/// </summary>
public sealed class ManaCost
{
    public int Generic;

    /// <summary>
    /// Colored symbols in printed order. A list rather than counts per color because printed
    /// order is part of the card's identity and round-trips through import and display.
    /// </summary>
    public List<ManaColor> Colored = [];

    /// <summary>
    /// How many colorless mana symbols {C} the cost has (CR 107.4c): a cost that can be paid only with one colorless
    /// mana each, never with generic mana of another type and never counted as a color (CR 105.2a — colorless is
    /// the absence of color, not a sixth one, so this never touches <see cref="Colors"/>). Unlike a colored symbol
    /// there is only one kind of colorless mana, so a count serves exactly as it does for <see cref="X"/>; printed
    /// order among colorless and colored symbols is not preserved, the same tradeoff <see cref="Hybrid"/> already
    /// makes against <see cref="Colored"/>.
    /// </summary>
    public int Colorless;

    /// <summary>
    /// Symbols with more than one printed way to pay them, in printed order: a two-color hybrid symbol (CR 107.4e),
    /// payable with one mana of either color, and a Phyrexian symbol (CR 107.4f), payable with one mana of its color
    /// or with life. Kept apart from <see cref="Colored"/> because paying one is a choice: which way pays is decided
    /// against the pool (and, for a Phyrexian symbol, against life), not printed, and a cost is only affordable if
    /// every such symbol can be paid one of its ways.
    /// </summary>
    public List<ManaSymbol> Hybrid = [];

    /// <summary>
    /// How many {X} symbols the cost has (CR 107.3). A count rather than a flag because {X}{X} is a real cost, and
    /// never a value: X is chosen as a spell is cast (CR 601.2b) and belongs to that spell, not to the card.
    /// </summary>
    public int X;

    /// <summary>
    /// True for an object with no mana cost at all (CR 202.1b), such as a land or a card like Living End, as opposed to
    /// a cost of {0}. The difference is real: a Mox costs nothing to cast, while a spell with no mana cost cannot be cast
    /// by paying it at all (CR 118.6), and the two render differently.
    /// </summary>
    public bool Absent;

    /// <summary>A cost of {0}: nothing to pay, but a cost.</summary>
    public static ManaCost Free() => new();

    /// <summary>No mana cost at all (CR 202.1b).</summary>
    public static ManaCost None() => new() { Absent = true };

    /// <summary>
    /// An independent copy.
    /// <para>
    /// A printed definition is shared by every object of that card, so the characteristics pipeline copies the
    /// cost rather than referencing it. Without this, the first effect to change one object's mana cost would
    /// change it for every other copy of the card and for the content set itself.
    /// </para>
    /// </summary>
    public ManaCost Copy() => new() { Generic = Generic, Colored = [.. Colored], Colorless = Colorless, Hybrid = [.. Hybrid], X = X, Absent = Absent };

    /// <summary>A cost reduced by up to <paramref name="generic"/> generic mana; colored symbols are never reduced (CR 601.2f).</summary>
    public ManaCost LessGeneric(int generic) =>
        new() { Generic = Math.Max(0, Generic - Math.Max(0, generic)), Colored = [.. Colored], Colorless = Colorless, Hybrid = [.. Hybrid], X = X, Absent = Absent };

    /// <summary>
    /// The cost with another added to it (CR 601.2f, a cost increase): generic to generic, colored symbols appended.
    /// A cost that was absent becomes one once something is added to it.
    /// </summary>
    public ManaCost Plus(ManaCost other)
    {
        ArgumentNullException.ThrowIfNull(other);

        return new ManaCost
        {
            Generic = Generic + other.Generic,
            Colored = [.. Colored, .. other.Colored],
            Colorless = Colorless + other.Colorless,
            Hybrid = [.. Hybrid, .. other.Hybrid],
            X = X,
            Absent = Absent && other.IsFree,
        };
    }

    /// <summary>
    /// The cost with another taken off it (CR 601.2f, a cost reduction): generic off generic and never below zero,
    /// each colored or hybrid symbol removed once if the cost has it, and colorless off colorless and never below
    /// zero. {X} is untouched.
    /// </summary>
    public ManaCost Less(ManaCost other)
    {
        ArgumentNullException.ThrowIfNull(other);

        List<ManaColor> colored = [.. Colored];
        foreach (ManaColor color in other.Colored)
        {
            colored.Remove(color);
        }

        List<ManaSymbol> hybrid = [.. Hybrid];
        foreach (ManaSymbol symbol in other.Hybrid)
        {
            hybrid.Remove(symbol);
        }

        return new ManaCost
        {
            Generic = Math.Max(0, Generic - other.Generic),
            Colored = colored,
            Colorless = Math.Max(0, Colorless - other.Colorless),
            Hybrid = hybrid,
            X = X,
            Absent = Absent,
        };
    }

    /// <summary>
    /// The cost as it is paid when mana of any type can be spent on it (CR 118.14, 609.4b): every coloured, colorless
    /// and hybrid symbol payable with any mana, which is to say generic. For paying only; the cost itself is unchanged.
    /// <para>
    /// A colorless symbol {C} folds in with the rest, because the permission is "as though it were colorless mana or
    /// mana of any color" (CR 118.14) and colorless is one of the six types of mana (CR 106.1b). An "any color"
    /// permission alone does not reach it; that is <see cref="AnyColor"/>.
    /// </para>
    /// <para>
    /// A Phyrexian symbol is the exception: it stays a symbol, relaxed (<see cref="ManaSymbol.Relax"/>), because it can
    /// also be paid with 2 life (CR 107.4f) and the permission changes how mana pays, never the cost (CR 609.4b).
    /// Folded into generic it would have lost that way — Gitaxian Probe cast from Black Widow's exile for {1} only.
    /// </para>
    /// </summary>
    public ManaCost AnyType() => Relaxed(ManaRelaxation.AnyType, colorlessToo: true);

    /// <summary>
    /// The cost as it is paid when mana may be spent "as though it were mana of any color" (CR 609.4b): every coloured
    /// and hybrid symbol payable with any mana, which is to say generic — but a colorless symbol {C} still takes
    /// colorless mana, because colorless is not a color (CR 105.2a, 106.1a) and the permission names colors only.
    /// Robber of the Rich's, as against Black Widow, Super Spy's "mana of any type" (<see cref="AnyType"/>). A
    /// Phyrexian symbol keeps its life way here too, relaxed to any color. For paying only; the cost itself is
    /// unchanged.
    /// </summary>
    public ManaCost AnyColor() => Relaxed(ManaRelaxation.AnyColor, colorlessToo: false);

    private ManaCost Relaxed(ManaRelaxation how, bool colorlessToo) => new()
    {
        Generic = Generic + Colored.Count + (colorlessToo ? Colorless : 0) + Hybrid.Count(symbol => symbol.LifeCost is null),
        Colorless = colorlessToo ? 0 : Colorless,
        Hybrid = [.. Hybrid.Where(symbol => symbol.LifeCost is not null).Select(symbol => symbol.Relax(how))],
        X = X,
        Absent = Absent,
    };

    /// <summary>
    /// The cost with a value chosen for X: each {X} becomes that much generic mana (CR 107.3a). What is actually paid
    /// for a spell cast with X chosen, and what the pool is planned against.
    /// </summary>
    public ManaCost WithX(int value)
    {
        ArgumentOutOfRangeException.ThrowIfNegative(value);
        return new ManaCost { Generic = Generic + (X * value), Colored = [.. Colored], Colorless = Colorless, Hybrid = [.. Hybrid] };
    }

    /// <summary>
    /// Mana value (CR 202.3): generic plus one per colored, colorless or hybrid symbol (CR 202.3d). {X} counts as
    /// zero, which is right everywhere but on the stack (CR 202.3e).
    /// </summary>
    public int ManaValue => Generic + Colored.Count + Colorless + Hybrid.Count;

    public bool HasX => X > 0;

    /// <summary>
    /// Nothing to pay: true for {0} and for no cost alike. Whether there is a cost at all is <see cref="Absent"/>.
    /// </summary>
    public bool IsFree => Generic == 0 && Colored.Count == 0 && Colorless == 0 && Hybrid.Count == 0 && X == 0;

    /// <summary>
    /// The colors of a card with this cost (CR 202.2); a hybrid symbol is both of its colors (CR 107.4e), and a
    /// Phyrexian symbol is its one color whether or not life paid it (CR 107.4f) — Gitaxian Probe is blue even cast
    /// for life alone. Note this is the cost's contribution only; a color indicator or a characteristic-defining
    /// ability can make a card's colors differ, and layer 5 can change them outright, so the engine asks the
    /// characteristics pipeline rather than reading this directly.
    /// </summary>
    public IReadOnlyList<ManaColor> Colors
    {
        get
        {
            List<ManaColor> distinct = [];
            foreach (ManaColor color in Colored)
            {
                if (!distinct.Contains(color))
                {
                    distinct.Add(color);
                }
            }

            foreach (ManaSymbol symbol in Hybrid)
            {
                foreach (ManaColor color in symbol.Colors)
                {
                    if (!distinct.Contains(color))
                    {
                        distinct.Add(color);
                    }
                }
            }

            distinct.Sort();
            return distinct;
        }
    }

    /// <summary>
    /// Parses a cost in oracle notation, for example <c>{2}{G}{G}</c>. An empty or absent string is no
    /// cost at all, which is what lands have; <c>{0}</c> is a cost of zero.
    /// </summary>
    /// <exception cref="ManaCostFormatException">The text is not a well-formed cost.</exception>
    public static ManaCost Parse(string? text)
    {
        if (string.IsNullOrWhiteSpace(text))
        {
            return None();
        }

        var cost = new ManaCost();

        int index = 0;
        string input = text.Trim();

        while (index < input.Length)
        {
            if (input[index] != '{')
            {
                throw new ManaCostFormatException(input, index, "expected '{' at the start of a symbol.");
            }

            int close = input.IndexOf('}', index);
            if (close < 0)
            {
                throw new ManaCostFormatException(input, index, "unclosed mana symbol.");
            }

            string symbol = input[(index + 1)..close];
            if (symbol.Length == 0)
            {
                throw new ManaCostFormatException(input, index, "empty mana symbol.");
            }

            ApplySymbol(cost, symbol, input, index);
            index = close + 1;
        }

        return cost;
    }

    private static void ApplySymbol(ManaCost cost, string symbol, string input, int position)
    {
        if (symbol == "X")
        {
            cost.X++;
            return;
        }

        if (int.TryParse(symbol, out int generic))
        {
            if (generic < 0)
            {
                throw new ManaCostFormatException(input, position, "a generic cost cannot be negative.");
            }

            // Two generic symbols in one cost is not a thing any real card does, but summing is
            // the only sane reading if it ever appears in imported data.
            cost.Generic += generic;
            return;
        }

        // The colorless mana symbol, {C} (CR 107.4c): a cost that can be paid only with one colorless mana.
        if (symbol == "C")
        {
            cost.Colorless++;
            return;
        }

        // A Phyrexian mana symbol, {B/P} (CR 107.4f): one mana of its color, or 2 life, pays it. Monocolored hybrid
        // ({2/W}) shares the slash too but is a different cost, so it is still refused rather than read as one of these.
        if (symbol.Length == 3 && symbol[1] == '/' && symbol[2] == 'P' && ColorOf(symbol[0]) is { } phyrexian)
        {
            cost.Hybrid.Add(ManaSymbol.Phyrexian(phyrexian));
            return;
        }

        // A two-color hybrid symbol, {U/R} (CR 107.4e).
        if (symbol.Length == 3 && symbol[1] == '/' && ColorOf(symbol[0]) is { } first && ColorOf(symbol[2]) is { } second)
        {
            if (first == second)
            {
                throw new ManaCostFormatException(input, position, $"'{{{symbol}}}' names the same color twice.");
            }

            cost.Hybrid.Add(ManaSymbol.Hybrid(first, second));
            return;
        }

        cost.Colored.Add(symbol.Length == 1 && ColorOf(symbol[0]) is { } color
            ? color
            : throw new ManaCostFormatException(
                input,
                position,
                $"'{{{symbol}}}' is not a supported mana symbol. Generic, single-color, colorless, two-color hybrid, Phyrexian, and {{X}} symbols only so far."));
    }

    private static ManaColor? ColorOf(char letter) => letter switch
    {
        'W' => ManaColor.White,
        'U' => ManaColor.Blue,
        'B' => ManaColor.Black,
        'R' => ManaColor.Red,
        'G' => ManaColor.Green,
        _ => null,
    };

    /// <summary>
    /// Renders back to oracle notation: {X} first, then generic, then colored in printed order, then colorless, then
    /// hybrid. No cost at all renders as nothing, and a cost of zero as <c>{0}</c>, so the two stay distinct wherever
    /// the string goes.
    /// </summary>
    public override string ToString()
    {
        if (Absent)
        {
            return string.Empty;
        }

        if (IsFree)
        {
            return "{0}";
        }

        var text = new StringBuilder();
        for (int i = 0; i < X; i++)
        {
            text.Append("{X}");
        }

        if (Generic > 0)
        {
            text.Append('{').Append(Generic).Append('}');
        }

        foreach (ManaColor color in Colored)
        {
            text.Append('{').Append(Symbol(color)).Append('}');
        }

        for (int i = 0; i < Colorless; i++)
        {
            text.Append("{C}");
        }

        foreach (ManaSymbol symbol in Hybrid)
        {
            text.Append('{').Append(symbol).Append('}');
        }

        return text.ToString();
    }

    public static char Symbol(ManaColor color) => color switch
    {
        ManaColor.White => 'W',
        ManaColor.Blue => 'U',
        ManaColor.Black => 'B',
        ManaColor.Red => 'R',
        ManaColor.Green => 'G',
        _ => throw new ArgumentOutOfRangeException(nameof(color), color, "Unknown color."),
    };
}

/// <summary>Which kind of thing pays a <see cref="ManaWay"/>.</summary>
public enum ManaWayKind
{
    /// <summary>One mana of <see cref="ManaWay.Color"/> pays it (CR 107.4e-f).</summary>
    Color,

    /// <summary><see cref="ManaWay.Amount"/> life pays it instead of mana (CR 107.4f).</summary>
    Life,
}

/// <summary>
/// One way a printed mana symbol may be paid: one mana of a color, or life instead of mana (CR 107.4e-f). A closed
/// set of concrete kinds with a discriminator, as state must be (<see cref="ManaWayKind"/>) rather than a class per
/// kind — a symbol carries only the ways it was printed with; a card that needs another way (monocolored hybrid's
/// generic mana, CR 107.4e) adds a kind here the same way this one did.
/// </summary>
public readonly record struct ManaWay(ManaWayKind Kind, ManaColor Color, int Amount)
{
    /// <summary>One mana of this color pays it (CR 107.4e-f).</summary>
    public static ManaWay Colored(ManaColor color) => new(ManaWayKind.Color, color, 0);

    /// <summary>This much life pays it instead of mana (CR 107.4f).</summary>
    public static ManaWay Life(int amount) => new(ManaWayKind.Life, default, amount);

    public override string ToString() => Kind switch
    {
        ManaWayKind.Color => ManaCost.Symbol(Color).ToString(),
        ManaWayKind.Life => "P",
        _ => "?",
    };
}

/// <summary>
/// How a cast permission relaxes what mana pays a symbol (CR 609.4b): "as though it were mana of any color" or "mana of
/// any type" (CR 118.14). See <see cref="ManaSymbol.Relaxed"/>.
/// </summary>
public enum ManaRelaxation
{
    /// <summary>As printed.</summary>
    None = 0,

    /// <summary>Mana of any color pays it; colorless still does not (CR 105.2a).</summary>
    AnyColor = 1,

    /// <summary>Mana of any type pays it, colorless included (CR 118.14).</summary>
    AnyType = 2,

    /// <summary>
    /// No mana may be spent on it at all (CR 601.2h, 106.4): "you can't spend mana to cast this spell". The cost is
    /// unchanged and the symbols are all still owed, but only what is not mana can pay them — the creatures tapped for
    /// convoke, the cards exiled for delve. A prohibition, so it outranks every permission beside it (CR 101.2).
    /// </summary>
    NoMana = 3,
}

/// <summary>
/// A printed symbol with more than one way to pay it: a two-color hybrid symbol, {U/R} (CR 107.4e), or a Phyrexian
/// symbol, {B/P} (CR 107.4f). Two-color hybrid is one instance of this shape — both its ways are
/// <see cref="ManaWayKind.Color"/> — rather than a shape of its own; Phyrexian is the same shape with one way
/// <see cref="ManaWayKind.Life"/> instead.
/// <para>
/// Plain data with value equality, because a mana cost is state (a token's or a copy's cost is stored with it) and
/// state is plain data. The two ways keep their printed order, so {U/R} and {R/U} compare as different symbols; no
/// printed card writes the same pair both ways, and paying either is the same decision.
/// </para>
/// </summary>
public readonly record struct ManaSymbol(ManaWay First, ManaWay Second)
{
    /// <summary>A two-color hybrid symbol, {U/R} (CR 107.4e): one mana of either color pays it, and it is both colors.</summary>
    public static ManaSymbol Hybrid(ManaColor first, ManaColor second) =>
        new(ManaWay.Colored(first), ManaWay.Colored(second));

    /// <summary>A Phyrexian mana symbol, {B/P} (CR 107.4f): one mana of its color, or 2 life, pays it.</summary>
    public static ManaSymbol Phyrexian(ManaColor color) =>
        new(ManaWay.Colored(color), ManaWay.Life(2));

    /// <summary>
    /// How a cast permission has relaxed what mana pays this symbol (CR 609.4b): <see cref="ManaRelaxation.None"/> as
    /// printed. Only a Phyrexian symbol is ever relaxed rather than folded into generic, because it alone has a way
    /// that is not mana — its 2 life (CR 107.4f) — which the permission leaves alone, since it changes how mana may
    /// pay a cost and never the cost itself.
    /// </summary>
    public ManaRelaxation Relaxed { get; init; }

    /// <summary>This symbol with its mana way relaxed as a cast permission says, its life way untouched.</summary>
    public ManaSymbol Relax(ManaRelaxation how) => this with { Relaxed = how };

    /// <summary>Whether one mana of this color could pay it: one of its own, or any color once relaxed.</summary>
    public bool Accepts(ManaColor color) => PaidWith.Contains(color);

    /// <summary>
    /// The colors one mana of which could pay this symbol: <see cref="Colors"/> as printed, and every color once a
    /// permission has relaxed it.
    /// </summary>
    public IEnumerable<ManaColor> PaidWith => Relaxed == ManaRelaxation.None ? Colors : Enum.GetValues<ManaColor>();

    /// <summary>
    /// Whether colorless mana could pay it — only under "mana of any type" (CR 118.14), colorless being no color
    /// (CR 105.2a).
    /// </summary>
    public bool PaidWithColorless => Relaxed == ManaRelaxation.AnyType;

    /// <summary>
    /// The colors the symbol prints (CR 107.4e-f): both halves of a hybrid symbol, or the one color of a Phyrexian
    /// symbol. What it is, which a relaxation does not change; what pays it is <see cref="PaidWith"/>.
    /// </summary>
    public IEnumerable<ManaColor> Colors
    {
        get
        {
            if (First.Kind == ManaWayKind.Color)
            {
                yield return First.Color;
            }

            if (Second.Kind == ManaWayKind.Color)
            {
                yield return Second.Color;
            }
        }
    }

    /// <summary>The life that pays this symbol instead of mana (CR 107.4f), or null when it has no such way.</summary>
    public int? LifeCost => First.Kind == ManaWayKind.Life ? First.Amount : Second.Kind == ManaWayKind.Life ? Second.Amount : null;

    public override string ToString() => $"{First}/{Second}";
}

/// <summary>A mana cost string could not be parsed.</summary>
public sealed class ManaCostFormatException : Exception
{
    public ManaCostFormatException(string input, int position, string reason)
        : base($"Bad mana cost \"{input}\" at index {position}: {reason}")
    {
        Input = input;
        Position = position;
    }

    public string Input { get; }

    public int Position { get; }
}
