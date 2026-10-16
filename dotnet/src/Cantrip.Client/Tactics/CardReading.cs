using System.Globalization;
using System.Text.RegularExpressions;
using Cantrip.Protocol;

namespace Cantrip.Client.Tactics;

/// <summary>The keyword abilities combat arithmetic depends on, as <see cref="CardReading"/> found them on a card.</summary>
[Flags]
public enum Keywords
{
    None = 0,
    Flying = 1 << 0,
    Reach = 1 << 1,
    FirstStrike = 1 << 2,
    DoubleStrike = 1 << 3,
    Deathtouch = 1 << 4,
    Lifelink = 1 << 5,
    Trample = 1 << 6,
    Vigilance = 1 << 7,
    Menace = 1 << 8,
    Haste = 1 << 9,
    Defender = 1 << 10,
    Indestructible = 1 << 11,
    Hexproof = 1 << 12,
    Prowess = 1 << 13,
    Flash = 1 << 14,
    CantBlock = 1 << 15,
    Unblockable = 1 << 16,
}

/// <summary>What an effect does to the thing it is aimed at, as far as this seat can tell from its words.</summary>
public enum EffectKind
{
    /// <summary>
    /// Nothing recognised. Such a thing is not cast to be aimed at anything; when one has to be aimed anyway, a
    /// trigger that must have a target, it goes at an opponent's things before this seat's own.
    /// </summary>
    Unknown,

    /// <summary>Deals <see cref="EffectRead.Amount"/> damage.</summary>
    Damage,

    /// <summary>Destroys or exiles a permanent.</summary>
    Remove,

    /// <summary>Returns a permanent to its owner's hand.</summary>
    Bounce,

    /// <summary>Gives -N/-N, <see cref="EffectRead.Amount"/> being N off toughness.</summary>
    Shrink,

    /// <summary>Takes control of a permanent.</summary>
    Steal,

    /// <summary>Hurts its target some other way: taps it, stops it blocking, makes a player discard.</summary>
    Harm,

    /// <summary>Counters a spell.</summary>
    Counter,

    /// <summary>Helps its target: a pump, counters, an untap, an attachment.</summary>
    Help,

    /// <summary>Keeps its target safe: protection, hexproof, indestructible, regeneration.</summary>
    Protect,

    /// <summary>
    /// Brings a card back from a graveyard: onto the battlefield when <see cref="EffectRead.Amount"/> is 1, to a hand
    /// when it is 0.
    /// </summary>
    Raise,

    /// <summary>
    /// Makes its target a creature of a stated size and nothing else, as Oko's Elk: <see cref="EffectRead.Amount"/>
    /// its power and <see cref="EffectRead.Toughness"/> its toughness. Good for a Food and bad for a dragon, so whose
    /// it is aimed at depends on what the thing was.
    /// </summary>
    Reshape,
}

/// <summary>One effect read off a card's words.</summary>
/// <param name="Kind">What it does to its target.</param>
/// <param name="Amount">How much, for damage and -N/-N. -1 for X.</param>
/// <param name="Kicked">The amount if the spell is kicked, or zero where kicking changes nothing read here.</param>
/// <param name="OwnerGainsPower">Swords to Plowshares: the target's controller gains life equal to its power.</param>
/// <param name="Toughness">The toughness a <see cref="EffectKind.Reshape"/> leaves its target with.</param>
public readonly record struct EffectRead(EffectKind Kind, int Amount = 0, int Kicked = 0, bool OwnerGainsPower = false, int Toughness = 0)
{
    /// <summary>Whether this is something done to an opponent's things rather than to this seat's own.</summary>
    public bool Hostile => Kind is EffectKind.Damage or EffectKind.Remove or EffectKind.Bounce or EffectKind.Shrink or EffectKind.Reshape
        or EffectKind.Steal or EffectKind.Harm or EffectKind.Counter;
}

/// <summary>
/// Reads what a card does off the words a player would read: its keywords, and what its spell or ability does to a
/// target.
/// <para>
/// The protocol sends a client rules text and computed numbers and nothing structured about either, on purpose: what
/// a card does is the server's to carry out, not the client's to know. A seat that has to choose well still has to
/// understand its cards, and the text is what there is, so this reads it the way a player skims a card — keyword
/// lines, "deals 3 damage to any target", "destroy target creature". It recognises the common shapes and says
/// <see cref="EffectKind.Unknown"/> to the rest, which the seat then treats with care rather than guessing.
/// </para>
/// <para>
/// Nothing here decides a rule. A misread costs this seat a worse choice among answers the server offered; it cannot
/// make an answer legal or illegal.
/// </para>
/// </summary>
public static partial class CardReading
{
    private static readonly (string Word, Keywords Keyword)[] Words =
    [
        ("flying", Keywords.Flying),
        ("reach", Keywords.Reach),
        ("first strike", Keywords.FirstStrike),
        ("double strike", Keywords.DoubleStrike),
        ("deathtouch", Keywords.Deathtouch),
        ("lifelink", Keywords.Lifelink),
        ("trample", Keywords.Trample),
        ("vigilance", Keywords.Vigilance),
        ("menace", Keywords.Menace),
        ("haste", Keywords.Haste),
        ("defender", Keywords.Defender),
        ("indestructible", Keywords.Indestructible),
        ("hexproof", Keywords.Hexproof),
        ("prowess", Keywords.Prowess),
        ("flash", Keywords.Flash),
    ];

    [GeneratedRegex(@"\([^)]*\)")]
    private static partial Regex Reminder();

    [GeneratedRegex(@"deals? (\d+|x) damage (?:to (?:any target|target|each opponent|that)|divided)")]
    private static partial Regex DamageClause();

    [GeneratedRegex(@"deals (\d+) damage instead")]
    private static partial Regex DamageInstead();

    [GeneratedRegex(@"gets? -(\d+|x)/-(\d+|x)")]
    private static partial Regex ShrinkClause();

    [GeneratedRegex(@"\{[^}]+\}")]
    private static partial Regex ManaSymbol();

    [GeneratedRegex(@"create (a|an|one|two|three|four|five|x|\d+) (?:tapped )?(\d+)/(\d+)")]
    private static partial Regex TokenClause();

    [GeneratedRegex(@"draws? (a|an|one|two|three|four|\d+) cards?")]
    private static partial Regex DrawClause();

    [GeneratedRegex(@"you gain (\d+) life")]
    private static partial Regex LifeClause();

    [GeneratedRegex(@"if (?:this creature|it) is an? (\w+)")]
    private static partial Regex IfItIsClause();

    [GeneratedRegex(@"base power and toughness (\d+)/(\d+)")]
    private static partial Regex BaseSizeClause();

    [GeneratedRegex(@"^\s*([+\-−–]?)(\d+)\s*:")]
    private static partial Regex LoyaltyClause();

    [GeneratedRegex(@"^\s*crew (\d+)")]
    private static partial Regex CrewClause();

    [GeneratedRegex(@"target (?:player|opponent) discards (a|an|one|two|three|four|x|\d+) cards?")]
    private static partial Regex DiscardClause();

    /// <summary>A card's rules text as this reads it: lower case, with reminder text taken out.</summary>
    public static string Plain(string text) =>
        string.IsNullOrEmpty(text) ? string.Empty : Reminder().Replace(text.ToLowerInvariant(), string.Empty);

    /// <summary>
    /// The keywords an object has: those on a line of its own in its text, those an effect has granted it, and those
    /// it has a counter for (CR 122.1b).
    /// </summary>
    public static Keywords KeywordsOf(ObjectView card)
    {
        ArgumentNullException.ThrowIfNull(card);

        Keywords found = Keywords.None;

        foreach (string line in Plain(card.OracleText).Split('\n'))
        {
            found |= Line(line);
        }

        foreach (AbilityTextView granted in card.GrantedAbilities)
        {
            found |= Line(Plain(granted.Text));
        }

        foreach (CounterView counter in card.Counters)
        {
            if (counter.Count > 0)
            {
                found |= Word(counter.Name);
            }
        }

        // What an effect has taken away (Day of Black Sun) is still in the printed text it reads above, and is not had: a
        // keyword granted again after it is not on this list, since the object has it.
        Keywords lost = Keywords.None;
        foreach (string gone in card.LostAbilities)
        {
            lost |= Line(Plain(gone));
        }

        return found & ~lost;
    }

    /// <summary>The keywords on one line, when keywords are all the line is: "First strike, lifelink".</summary>
    private static Keywords Line(string line)
    {
        string trimmed = line.Trim().TrimEnd('.');
        if (trimmed.Length == 0)
        {
            return Keywords.None;
        }

        if (trimmed.Contains("can't be blocked", StringComparison.Ordinal) && !trimmed.Contains("except", StringComparison.Ordinal)
            && (trimmed.StartsWith("this creature", StringComparison.Ordinal) || trimmed.Length < 40))
        {
            return Keywords.Unblockable;
        }

        if (trimmed.EndsWith("can't block", StringComparison.Ordinal))
        {
            return Keywords.CantBlock;
        }

        Keywords found = Keywords.None;
        foreach (string part in trimmed.Split(',', ';'))
        {
            string word = part.Trim();
            Keywords known = Word(word);

            // A keyword this does not know is still a keyword — "Flying, ward {2}", "Trample, infect" — as long as it
            // is as short as one. A sentence that merely mentions a keyword, "creatures you control have flying", is
            // not this object having it.
            if (known == Keywords.None && word.Count(letter => letter == ' ') > 2 && !word.StartsWith("protection from", StringComparison.Ordinal))
            {
                return Keywords.None;
            }

            found |= known;
        }

        return found;
    }

    private static Keywords Word(string word)
    {
        foreach ((string name, Keywords keyword) in Words)
        {
            if (string.Equals(word, name, StringComparison.Ordinal))
            {
                return keyword;
            }
        }

        return Keywords.None;
    }

    /// <summary>
    /// What a piece of rules text does to its target: a spell's text, one mode of it, or an ability as it reads.
    /// </summary>
    public static EffectRead Effect(string text)
    {
        string plain = Plain(text);
        if (plain.Length == 0)
        {
            return default;
        }

        if (DamageClause().Match(plain) is { Success: true } damage)
        {
            int kicked = DamageInstead().Match(plain) is { Success: true } instead ? Number(instead.Groups[1].Value) : 0;
            return new EffectRead(EffectKind.Damage, Number(damage.Groups[1].Value), kicked);
        }

        if (plain.Contains("counter target", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Counter);
        }

        if (plain.Contains("target", StringComparison.Ordinal) && plain.Contains("graveyard", StringComparison.Ordinal)
            && (plain.Contains("onto the battlefield", StringComparison.Ordinal) || plain.Contains("to the battlefield", StringComparison.Ordinal)))
        {
            return new EffectRead(EffectKind.Raise, 1);
        }

        if ((plain.Contains("return target", StringComparison.Ordinal) || plain.Contains("return up to one target", StringComparison.Ordinal))
            && plain.Contains("graveyard to", StringComparison.Ordinal) && plain.Contains("hand", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Raise, 0);
        }

        if (plain.Contains("destroy target", StringComparison.Ordinal) || plain.Contains("exile target", StringComparison.Ordinal)
            || plain.Contains("destroy up to", StringComparison.Ordinal) || plain.Contains("exile up to", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Remove, OwnerGainsPower: plain.Contains("gains life equal to its power", StringComparison.Ordinal));
        }

        if (plain.Contains("gain control of target", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Steal);
        }

        if (plain.Contains("target", StringComparison.Ordinal) && plain.Contains("loses all abilities", StringComparison.Ordinal)
            && BaseSizeClause().Match(plain) is { Success: true } reshaped)
        {
            return new EffectRead(EffectKind.Reshape, Number(reshaped.Groups[1].Value), Toughness: Number(reshaped.Groups[2].Value));
        }

        // Onto or into its owner's library is a bounce that also costs them a draw.
        if (plain.Contains("put target", StringComparison.Ordinal) && plain.Contains("owner's library", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Bounce);
        }

        // An extra turn for a player of its choosing is for itself.
        if (plain.Contains("target player takes an extra turn", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Help);
        }

        if ((plain.Contains("return target", StringComparison.Ordinal) || plain.Contains("return up to one target", StringComparison.Ordinal))
            && plain.Contains("owner's hand", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Bounce);
        }

        if (plain.Contains("target", StringComparison.Ordinal) && ShrinkClause().Match(plain) is { Success: true } shrink)
        {
            return new EffectRead(EffectKind.Shrink, Number(shrink.Groups[2].Value));
        }

        if (plain.Contains("protection from", StringComparison.Ordinal) || plain.Contains("gains hexproof", StringComparison.Ordinal)
            || plain.Contains("gains indestructible", StringComparison.Ordinal) || plain.Contains("regenerate target", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Protect);
        }

        if (plain.Contains("you control", StringComparison.Ordinal) && plain.Contains("target", StringComparison.Ordinal)
            || plain.Contains("target player draws", StringComparison.Ordinal)
            || (plain.Contains("target", StringComparison.Ordinal) && plain.Contains(" gains ", StringComparison.Ordinal))
            || plain.Contains("gets +", StringComparison.Ordinal) || plain.Contains("+1/+1 counter", StringComparison.Ordinal)
            || plain.Contains("untap target", StringComparison.Ordinal) || plain.Contains("attach", StringComparison.Ordinal)
            || plain.StartsWith("equip", StringComparison.Ordinal) || plain.Contains(": equip", StringComparison.Ordinal)
            || plain.Contains("target attacking creature", StringComparison.Ordinal) && plain.Contains("counter", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Help);
        }

        // A player made to discard: how many cards says what it is worth, and to whom.
        if (DiscardClause().Match(plain) is { Success: true } discard)
        {
            return new EffectRead(EffectKind.Harm, Number(discard.Groups[1].Value));
        }

        if (plain.Contains("tap target", StringComparison.Ordinal) || plain.Contains("can't block", StringComparison.Ordinal)
            || plain.Contains("can't attack", StringComparison.Ordinal)
            || plain.Contains("reveals their hand", StringComparison.Ordinal)
            || plain.Contains("that much damage", StringComparison.Ordinal)
            || plain.Contains("doesn't untap", StringComparison.Ordinal) || plain.Contains("target opponent", StringComparison.Ordinal)
            || plain.Contains("target player discards", StringComparison.Ordinal) || plain.Contains("sacrifices", StringComparison.Ordinal)
            || plain.Contains("fights target", StringComparison.Ordinal) || plain.Contains("loses", StringComparison.Ordinal))
        {
            return new EffectRead(EffectKind.Harm);
        }

        return default;
    }

    /// <summary>The modes of a modal card, each as it reads, or empty for a card with none.</summary>
    public static IReadOnlyList<string> Modes(string text) =>
        [.. (text ?? string.Empty).Split('\n').Where(line => line.TrimStart().StartsWith('•')).Select(line => line.TrimStart(' ', '•'))];

    /// <summary>Whether a permanent does something whenever it, or its controller, attacks.</summary>
    public static bool AttackTrigger(string text)
    {
        string plain = Plain(text);
        return plain.Contains("whenever this creature attacks", StringComparison.Ordinal)
            || plain.Contains("whenever you attack", StringComparison.Ordinal)
            || plain.Contains("battle cry", StringComparison.Ordinal)
            || (plain.Contains("whenever ", StringComparison.Ordinal) && plain.Contains(" attacks,", StringComparison.Ordinal)
                && !plain.Contains("whenever a creature attacks you", StringComparison.Ordinal));
    }

    /// <summary>Whether a permanent pays out when it deals combat damage to a player, as Ragavan does.</summary>
    public static bool ConnectTrigger(string text) =>
        Plain(text).Contains("deals combat damage to a player", StringComparison.Ordinal);

    /// <summary>A card that is sacrificed as it enters unless it was cast some other way, as Phlage is.</summary>
    public static bool LeavesAtOnce(string text) =>
        Plain(text).Contains("sacrifice it unless it escaped", StringComparison.Ordinal);

    /// <summary>The kicker cost in mana, or zero for a card without kicker (or with one that is not simply mana).</summary>
    public static int KickerCost(string text)
    {
        foreach (string line in (text ?? string.Empty).ToLowerInvariant().Split('\n'))
        {
            if (line.StartsWith("kicker ", StringComparison.Ordinal))
            {
                int paren = line.IndexOf('(', StringComparison.Ordinal);
                return ManaValue(paren < 0 ? line : line[..paren]);
            }
        }

        return 0;
    }

    /// <summary>The mana value of every mana symbol in a piece of text: 2 for "{1}{R}", 4 for "{4}".</summary>
    public static int ManaValue(string text)
    {
        int total = 0;
        foreach (Match symbol in ManaSymbol().Matches(text ?? string.Empty))
        {
            string inner = symbol.Value[1..^1];
            if (int.TryParse(inner, NumberStyles.None, CultureInfo.InvariantCulture, out int generic))
            {
                total += generic;
            }
            else if (inner is not ("t" or "T" or "q" or "Q" or "e" or "E" or "x" or "X"))
            {
                total++;
            }
        }

        return total;
    }

    /// <summary>How many creature tokens a piece of text makes and how big, or a count of zero.</summary>
    public static (int Count, int Power, int Toughness) Tokens(string text)
    {
        if (TokenClause().Match(Plain(text)) is not { Success: true } token)
        {
            return default;
        }

        return (Math.Max(1, Number(token.Groups[1].Value)), Number(token.Groups[2].Value), Number(token.Groups[3].Value));
    }

    /// <summary>How many cards a piece of text has its controller draw, or zero.</summary>
    public static int Draws(string text)
    {
        // A line at a time: "Delve" on one line and "Draw three cards" on the next is a draw.
        foreach (string line in Plain(text).Split('\n'))
        {
            string plain = line.TrimStart();
            bool draws = plain.Contains("you draw", StringComparison.Ordinal) || plain.StartsWith("draw ", StringComparison.Ordinal)
                || plain.Contains(". draw ", StringComparison.Ordinal)
                || plain.Contains(": draw ", StringComparison.Ordinal) || plain.Contains(", draw ", StringComparison.Ordinal)
                || plain.Contains("then draw ", StringComparison.Ordinal) || plain.Contains(" and draw ", StringComparison.Ordinal);

            if (draws && DrawClause().Match(plain) is { Success: true } draw)
            {
                return Math.Max(1, Number(draw.Groups[1].Value));
            }
        }

        return 0;
    }

    /// <summary>A spell that only makes mana: Dark Ritual. Worth casting for what it pays for and for nothing else.</summary>
    public static bool Ritual(string text)
    {
        string plain = Plain(text).TrimStart();
        return (plain.StartsWith("add {", StringComparison.Ordinal) || plain.StartsWith("add two mana", StringComparison.Ordinal)
            || plain.StartsWith("add x mana", StringComparison.Ordinal)) && Draws(text) == 0;
    }

    /// <summary>
    /// The mana a permanent's own ability makes each time it is used, and whether it can only be used the once: a Mox
    /// makes one for good, Sol Ring two, Black Lotus three and then it is gone. Zero for a card with no such ability,
    /// for one that costs mana to use (Chromatic Star gives back what it took), and for one that asks for a card —
    /// Lion's Eye Diamond's hand, Mox Diamond's land, Chrome Mox's imprint — which is a trade this does not price.
    /// </summary>
    public static (int Amount, bool Once) ManaMade(string text)
    {
        string plain = Plain(text);
        if (plain.Contains("you may discard", StringComparison.Ordinal) || plain.Contains("you may exile", StringComparison.Ordinal))
        {
            return default;
        }

        (int Amount, bool Once) best = default;

        foreach (string line in plain.Split('\n'))
        {
            int at = line.IndexOf(": add ", StringComparison.Ordinal);
            if (at < 0)
            {
                continue;
            }

            // Tapped for, or a counter taken off for: not a planeswalker's "+1: Add {R}{R}".
            string cost = line[..at];
            if (ManaValue(cost) > 0 || cost.Contains("discard", StringComparison.Ordinal)
                || !(cost.Contains("{t}", StringComparison.Ordinal) || cost.Contains("remove", StringComparison.Ordinal)))
            {
                continue;
            }

            string adds = line[(at + 6)..];
            int stop = adds.IndexOf('.', StringComparison.Ordinal);
            adds = stop < 0 ? adds : adds[..stop];

            int amount = adds.Contains("three mana", StringComparison.Ordinal) ? 3
                : adds.Contains("two mana", StringComparison.Ordinal) ? 2
                : adds.Contains(" mana", StringComparison.Ordinal) ? 1
                : Math.Max(adds.Split([" or ", ", "], StringSplitOptions.None).Max(ManaValue), 1);

            bool once = cost.Contains("sacrifice", StringComparison.Ordinal) || cost.Contains("remove", StringComparison.Ordinal)
                || plain.Contains("doesn't untap during your untap step", StringComparison.Ordinal);

            // The ability that keeps giving before the one that gives once, and of two alike the one that gives more.
            if (best.Amount == 0 || (best.Once && !once) || (best.Once == once && amount > best.Amount))
            {
                best = (amount, once);
            }
        }

        return best;
    }

    /// <summary>Whether a piece of text gives its controller another turn.</summary>
    public static bool ExtraTurn(string text) => Plain(text).Contains("extra turn", StringComparison.Ordinal);

    /// <summary>
    /// Whether a piece of text does the same thing to everyone: "each player", "all creatures", "all permanents".
    /// Whether that is good depends on the board, so one this seat cannot weigh is one it does not cast.
    /// </summary>
    public static bool Sweeping(string text)
    {
        string plain = Plain(text);
        return plain.Contains("each player", StringComparison.Ordinal) || plain.Contains("all permanents", StringComparison.Ordinal)
            || plain.Contains("all creatures", StringComparison.Ordinal) || plain.Contains("all artifacts", StringComparison.Ordinal)
            || plain.Contains("each creature", StringComparison.Ordinal) || plain.Contains("all lands", StringComparison.Ordinal);
    }

    /// <summary>How many cards a piece of text has a player of its controller's choosing draw, or zero.</summary>
    public static int TargetDraws(string text)
    {
        string plain = Plain(text);
        int at = plain.IndexOf("target player draws", StringComparison.Ordinal);
        return at >= 0 && DrawClause().Match(plain, at) is { Success: true } draw ? Math.Max(1, Number(draw.Groups[1].Value)) : 0;
    }

    /// <summary>Whether a piece of text lets its controller choose what they draw next: a scry, a surveil, a look at the top.</summary>
    public static bool Selects(string text)
    {
        string plain = Plain(text);
        return plain.Contains("scry", StringComparison.Ordinal) || plain.Contains("surveil", StringComparison.Ordinal)
            || plain.Contains("look at the top", StringComparison.Ordinal);
    }

    /// <summary>How much life a piece of text has its controller gain, or zero.</summary>
    public static int LifeGain(string text) =>
        LifeClause().Match(Plain(text)) is { Success: true } gain ? Number(gain.Groups[1].Value) : 0;

    /// <summary>
    /// The creature type an ability asks its own creature to be before it does anything — the Spirit in Figure of
    /// Destiny's "If this creature is a Spirit, it becomes…" — or null for an ability with no such condition.
    /// </summary>
    public static string? RequiredType(string text) =>
        IfItIsClause().Match(Plain(text)) is { Success: true } condition ? condition.Groups[1].Value : null;

    /// <summary>How many Phyrexian mana symbols (CR 107.4f) a mana cost has, each payable with two life instead: 2 for "{1}{B/P}{B/P}".</summary>
    public static int Phyrexian(string manaCost) =>
        ManaSymbol().Matches(manaCost ?? string.Empty).Count(symbol => symbol.Value.EndsWith("/P}", StringComparison.OrdinalIgnoreCase));

    /// <summary>The power a crew ability asks for (CR 702.122a) — 3 for "Crew 3" — or zero for an ability that is not one.</summary>
    public static int Crew(string ability) =>
        CrewClause().Match(Plain(ability)) is { Success: true } crew ? Number(crew.Groups[1].Value) : 0;

    /// <summary>
    /// What a loyalty ability does to its planeswalker's loyalty, read off the front of it — 2 for "+2: ...", -3 for
    /// "−3: ..." — or null for an ability that is not one.
    /// </summary>
    public static int? Loyalty(string ability)
    {
        if (LoyaltyClause().Match(ability ?? string.Empty) is not { Success: true } cost)
        {
            return null;
        }

        int amount = Number(cost.Groups[2].Value);
        return cost.Groups[1].Value is "" or "+" ? amount : -amount;
    }

    /// <summary>The base power and toughness a piece of text sets, or null when it sets none.</summary>
    public static (int Power, int Toughness)? BaseSize(string text) =>
        BaseSizeClause().Match(Plain(text)) is { Success: true } size
            ? (Number(size.Groups[1].Value), Number(size.Groups[2].Value))
            : null;

    /// <summary>A number as a card writes it: "three" is 3, "a" is 1, "x" is -1, and anything unread 0.</summary>
    public static int Count(string word) => Number(word);

    private static int Number(string word) => word switch
    {
        "a" or "an" or "one" => 1,
        "two" => 2,
        "three" => 3,
        "four" => 4,
        "five" => 5,
        "x" => -1,
        _ => int.TryParse(word, NumberStyles.None, CultureInfo.InvariantCulture, out int number) ? number : 0,
    };
}
