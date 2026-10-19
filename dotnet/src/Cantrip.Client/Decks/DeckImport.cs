using System.Globalization;
using System.Text.RegularExpressions;

namespace Cantrip.Client.Decks;

/// <summary>
/// A deck read from a decklist as Magic Online or Arena export it, for the deck editor to import: one card per line,
/// <c>4 Lightning Bolt</c> (or <c>4x</c>), with any set, collector number or foil mark after the name ignored -
/// <c>4 Lightning Bolt (M10) 146</c>, <c>[M10]</c>, <c>*F*</c>.
/// <para>
/// Arena heads its sections: <c>Commander</c>, <c>Companion</c>, <c>Deck</c>, <c>Sideboard</c>, and an <c>About</c>
/// whose <c>Name</c> line names the deck; a companion begins the game outside it (CR 702.139), so it goes to the
/// sideboard, and a <c>Maybeboard</c> is left out. Magic Online heads nothing: the sideboard is what comes after the first
/// blank line, or after a <c>SB:</c> prefix. Lines starting with <c>#</c> or <c>//</c> are comments.
/// </para>
/// <para>
/// Each name is matched to the cards the engine plays (<see cref="Matcher"/>); what matches none is listed as not
/// supported and left out of the deck.
/// </para>
/// </summary>
public static partial class DeckImport
{
    private enum Section { Commanders, Main, Side, Skipped, About }

    /// <summary>What reading a decklist gives: the deck with the cards the engine plays, and the names it does not, as written.</summary>
    public sealed record Result(SavedDeck Deck, IReadOnlyList<string> Unsupported)
    {
        public bool Empty => Deck.Commanders.Count + Deck.Main.Count + Deck.Side.Count + Unsupported.Count == 0;
    }

    [GeneratedRegex(@"^(?:SB:\s*)?(\d+)\s*[xX]?\s+(.+)$", RegexOptions.CultureInvariant)]
    private static partial Regex Counted();

    [GeneratedRegex(@"\s*\*[A-Za-z]+\*", RegexOptions.CultureInvariant)]
    private static partial Regex FoilMark();

    [GeneratedRegex(@"\s+(?:\([^)]*\)|\[[^\]]*\])(?:\s+\S+)?$", RegexOptions.CultureInvariant)]
    private static partial Regex Printing();

    [GeneratedRegex(@"^(?<header>[A-Za-z ]+?)\s*(?:\(\d+\))?:?$", RegexOptions.CultureInvariant)]
    private static partial Regex Header();

    /// <summary>A heading a site puts over a group of cards, <c>Creatures (20)</c> or <c>Lands:</c>, which is no card.</summary>
    [GeneratedRegex(@"(?:\(\d+\)|:)$", RegexOptions.CultureInvariant)]
    private static partial Regex Category();

    /// <summary>Reads a decklist, matching each name to a card the engine plays by <paramref name="find"/> (null for none).</summary>
    public static Result Read(string text, Func<string, string?> find)
    {
        ArgumentNullException.ThrowIfNull(text);
        ArgumentNullException.ThrowIfNull(find);

        var deck = new SavedDeck();
        List<string> unsupported = [];
        Section section = Section.Main;
        bool headed = false;
        bool anyCard = false;

        // A XAML text box ends its lines with a bare \r; a paste from elsewhere may have \r\n or \n.
        foreach (string raw in text.Split(["\r\n", "\r", "\n"], StringSplitOptions.None))
        {
            string line = raw.Trim();
            if (line.Length == 0)
            {
                // Unheaded, as Magic Online writes it: the sideboard follows the first blank line after a card.
                if (!headed && anyCard && section == Section.Main)
                {
                    section = Section.Side;
                }

                continue;
            }

            if (line.StartsWith('#') || line.StartsWith("//", StringComparison.Ordinal))
            {
                continue;
            }

            if (Counted().Match(line) is not { Success: true } counted)
            {
                if (HeaderOf(line) is { } next)
                {
                    section = next;
                    headed = true;
                }
                else if (section == Section.About && line.StartsWith("Name ", StringComparison.OrdinalIgnoreCase))
                {
                    deck.Name = line[5..].Trim();
                }
                else if (section is not (Section.About or Section.Skipped) && !Category().IsMatch(line))
                {
                    // A name with no count is one copy, as a plain decklist has it.
                    Add(deck, section, line, 1, find, unsupported);
                    anyCard = true;
                }

                continue;
            }

            bool sideboarded = line.StartsWith("SB:", StringComparison.OrdinalIgnoreCase);
            int copies = int.TryParse(counted.Groups[1].Value, NumberStyles.None, CultureInfo.InvariantCulture, out int count) ? count : 0;
            if (copies <= 0 || section is Section.About or Section.Skipped)
            {
                continue;
            }

            Add(deck, sideboarded ? Section.Side : section, counted.Groups[2].Value, copies, find, unsupported);
            anyCard = true;
        }

        return new Result(deck, unsupported);
    }

    /// <summary>
    /// Finds the engine's name for a name a decklist writes: as it is, ignoring case; a split card or a double-faced one
    /// written with its faces (<c>Fire // Ice</c>, or Magic Online's <c>Fire/Ice</c>) or by its front face alone.
    /// </summary>
    public static Func<string, string?> Matcher(IEnumerable<string> names)
    {
        ArgumentNullException.ThrowIfNull(names);
        Dictionary<string, string> exact = new(StringComparer.OrdinalIgnoreCase);
        Dictionary<string, string> fronts = new(StringComparer.OrdinalIgnoreCase);
        foreach (string name in names)
        {
            exact.TryAdd(name, name);
            if (name.IndexOf("//", StringComparison.Ordinal) is > 0 and int split)
            {
                fronts.TryAdd(name[..split].Trim(), name);
            }
        }

        return written =>
        {
            string name = Regex.Replace(written.Trim(), @"\s*/{1,2}\s*", " // ");
            if (exact.TryGetValue(name, out string? found) || fronts.TryGetValue(name, out found))
            {
                return found;
            }

            // Both faces written of a card the engine names by its front: Delver of Secrets // Insectile Aberration.
            int split = name.IndexOf(" // ", StringComparison.Ordinal);
            return split > 0 && exact.TryGetValue(name[..split], out found) ? found : null;
        };
    }

    private static Section? HeaderOf(string line)
    {
        if (Header().Match(line) is not { Success: true } match)
        {
            return null;
        }

        return match.Groups["header"].Value.ToLowerInvariant() switch
        {
            "commander" or "commanders" => Section.Commanders,
            "deck" or "main" or "maindeck" or "main deck" or "mainboard" => Section.Main,
            "sideboard" or "side" or "companion" => Section.Side,
            "maybeboard" or "considering" => Section.Skipped,
            "about" => Section.About,
            _ => null,
        };
    }

    private static void Add(SavedDeck deck, Section section, string written, int copies, Func<string, string?> find, List<string> unsupported)
    {
        string name = Printing().Replace(FoilMark().Replace(written, string.Empty), string.Empty).Trim();
        if (name.Length == 0)
        {
            return;
        }

        if (find(name) is not { } card)
        {
            if (!unsupported.Contains(name, StringComparer.OrdinalIgnoreCase))
            {
                unsupported.Add(name);
            }

            return;
        }

        List<DeckEntry> into = section switch
        {
            Section.Commanders => deck.Commanders,
            Section.Side => deck.Side,
            _ => deck.Main,
        };
        int had = into.FindIndex(entry => entry.Card == card);
        if (had >= 0)
        {
            into[had] = into[had] with { Count = into[had].Count + copies };
        }
        else
        {
            into.Add(new DeckEntry(card, copies));
        }
    }
}
