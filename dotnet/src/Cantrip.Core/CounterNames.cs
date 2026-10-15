using System.Text;

namespace Cantrip.Core;

/// <summary>
/// How a kind of counter is named (CR 122.1), and the two shapes that name takes.
/// <para>
/// A counter is identified by its name and by nothing else. "Put a mining counter on it" and "put a charge counter
/// on it" are two different counters, and a card that counts, moves or removes one kind must not find the other —
/// which is the whole reason the name travels rather than a closed set of kinds. The names below are only the ones
/// the rules attach meaning to; any other name a card prints is just as real a counter, and needs nothing here.
/// </para>
/// <para>
/// In Core because both sides need it: the engine identifies counters by these names, and a client that has never
/// heard of a counter still has to draw the one the board is carrying.
/// </para>
/// </summary>
public static class CounterNames
{
    /// <summary>CR 122.1c: the two that change power and toughness.</summary>
    public const string PlusOnePlusOne = "+1/+1";
    public const string MinusOneMinusOne = "-1/-1";

    /// <summary>CR 306.5b: a planeswalker's loyalty is the number of these on it.</summary>
    public const string Loyalty = "loyalty";

    /// <summary>CR 310.4c: a battle's defense is the number of these on it.</summary>
    public const string Defense = "defense";

    /// <summary>CR 122.1b: keyword counters, which grant their permanent the keyword they are named for.</summary>
    public const string Indestructible = "indestructible";
    public const string Lifelink = "lifelink";
    public const string Flying = "flying";
    public const string Hexproof = "hexproof";

    /// <summary>CR 122.1d: "if a permanent with a stun counter would become untapped, remove one instead".</summary>
    public const string Stun = "stun";

    /// <summary>CR 122.1h: one or more of these replace a trip to the graveyard with exile.</summary>
    public const string Finality = "finality";

    /// <summary>CR 122.1c, 714.3: a Saga's progress through its chapters.</summary>
    public const string Lore = "lore";

    /// <summary>CR 702.62a: suspend's time counters, one taken off at each of its owner's upkeeps.</summary>
    public const string Time = "time";

    /// <summary>CR 122.1a: counters with no rules meaning of their own, which their permanent's abilities read.</summary>
    public const string Charge = "charge";
    public const string Corpse = "corpse";

    /// <summary>CR 122.1f: counters on a player, ten of which lose the game. Kept as the player's own running total.</summary>
    public const string Poison = "poison";

    /// <summary>CR 107.16, 122.1: energy counters on a player, the {E} symbol. Kept as the player's own running total.</summary>
    public const string Energy = "energy";

    /// <summary>
    /// A counter name as it is stored and compared: trimmed, and lower case, so "Mining", "mining" and " mining "
    /// are one counter. Case is a matter of how a card file or a rules text happens to be written, never a
    /// distinction between two kinds of counter.
    /// </summary>
    public static string Normalize(string? name) =>
        string.IsNullOrWhiteSpace(name) ? string.Empty : name.Trim().ToLowerInvariant();

    /// <summary>
    /// The same name as an identifier, for somewhere a name cannot go: a state-dump key, a log field, a settings
    /// entry. "mining" is <c>Mining</c> and "+1/+1" is <c>PlusOnePlusOne</c>, which is what those keys read before
    /// counters were named and is why they still do.
    /// <para>
    /// A one-way spelling. Two names could in principle produce one token, so nothing reads identity back out of
    /// it; it is for people, and <see cref="Normalize"/> is for the game.
    /// </para>
    /// </summary>
    public static string Token(string? name)
    {
        string normalized = Normalize(name);

        if (normalized.Length == 0)
        {
            return string.Empty;
        }

        // The two the rules spell out in words rather than digits, and the two whose tokens were already written
        // down: every scenario that checks a +1/+1 counter names this key.
        if (normalized == PlusOnePlusOne)
        {
            return "PlusOnePlusOne";
        }

        if (normalized == MinusOneMinusOne)
        {
            return "MinusOneMinusOne";
        }

        var token = new StringBuilder(normalized.Length);
        bool starting = true;

        foreach (char letter in normalized)
        {
            if (letter == '+')
            {
                token.Append("Plus");
                starting = true;
                continue;
            }

            if (letter == '-')
            {
                token.Append("Minus");
                starting = true;
                continue;
            }

            if (!char.IsLetterOrDigit(letter))
            {
                // Any other punctuation — the slash in "+1/+1", a space in a two-word name — is a word boundary.
                starting = true;
                continue;
            }

            token.Append(starting ? char.ToUpperInvariant(letter) : letter);
            starting = false;
        }

        return token.Length == 0 ? normalized.ToUpperInvariant() : token.ToString();
    }
}
