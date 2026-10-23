using System.Globalization;
using System.Text.RegularExpressions;
using Cantrip.Cards.Model;

namespace Cantrip.Server;

/// <summary>One line of a decklist: some copies of a card, named as it is printed.</summary>
public sealed record DeckListEntry(string Card, int Copies, int Line);

/// <summary>
/// A deck read from a plain-text decklist: one card per line, <c>4 Lightning Bolt</c>, or just the name for one copy.
/// Blank lines and lines starting with <c>#</c> are ignored. A line reading <c>Sideboard</c> on its own ends the deck
/// and starts the sideboard, whose cards begin the game outside it (CR 100.4, 400.11a): what a card that brings one in chooses
/// among (North Wind Avatar). A line reading <c>Companion</c> names, on the next line, the deck's companion (CR 702.139a): a
/// sideboard card of its own, revealed before the game (CR 103.2b) - the server refuses the deck if it doesn't fulfil the
/// companion's condition.
/// <para>
/// A data file rather than code, and kept outside <c>content</c>: cards are content the game ships, while a deck is
/// something a player makes, so decks are headed for user data. Cards are named rather than given by id for the same
/// reason — a person writes "Lightning Bolt", not "cube/lightning-bolt" — and the names are resolved against whatever
/// cards the table loaded, so a deck naming a card that is not there is an error before the game is dealt.
/// </para>
/// </summary>
public sealed class DeckList
{
    private static readonly Regex Counted = new(@"^(\d+)\s+(.+)$", RegexOptions.CultureInvariant);

    /// <summary>The file's name without its extension, as <c>1-black-red</c>.</summary>
    public required string Name { get; init; }

    /// <summary>Where it was read from, for error messages.</summary>
    public required string Source { get; init; }

    public required IReadOnlyList<DeckListEntry> Entries { get; init; }

    /// <summary>
    /// The cards after the <c>Sideboard</c> line, which are not part of the starting deck (CR 702.139b). Empty for
    /// almost every decklist.
    /// </summary>
    public IReadOnlyList<DeckListEntry> Sideboard { get; init; } = [];

    /// <summary>The card after the <c>Companion</c> line: the deck's companion (CR 702.139a), or null for none.</summary>
    public string? Companion { get; init; }

    public int Count => Entries.Sum(entry => entry.Copies);

    /// <summary>Reads a decklist file.</summary>
    /// <exception cref="FormatException">A line is not a card with an optional count of at least one.</exception>
    public static DeckList Load(string path)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        return Parse(Path.GetFileNameWithoutExtension(path), File.ReadAllText(path), path);
    }

    /// <summary>
    /// Every decklist (<c>*.txt</c>) directly in a directory, in ordinal file-name order, which is the order a table
    /// deals them to its seats.
    /// </summary>
    /// <exception cref="InvalidOperationException">The directory holds no decklists.</exception>
    public static IReadOnlyList<DeckList> LoadDirectory(string directory)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(directory);

        List<DeckList> decks = [.. Directory.GetFiles(directory, "*.txt")
            .Order(StringComparer.Ordinal)
            .Select(Load)];

        return decks.Count > 0 ? decks : throw new InvalidOperationException($"No decklists (*.txt) in {directory}.");
    }

    /// <summary>
    /// The decklist file a player named: a path that exists, else a file under <paramref name="decksRoot"/>, with or
    /// without <c>.txt</c> (<c>legend-rule-test</c>, <c>demo/1-black-red</c>).
    /// </summary>
    /// <exception cref="FileNotFoundException">No such file.</exception>
    public static string Find(string decksRoot, string deck)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(decksRoot);
        ArgumentException.ThrowIfNullOrWhiteSpace(deck);

        foreach (string candidate in (string[])[deck, Path.Combine(decksRoot, deck), Path.Combine(decksRoot, deck + ".txt")])
        {
            if (File.Exists(candidate))
            {
                return candidate;
            }
        }

        throw new FileNotFoundException($"No decklist '{deck}', as a path or under {decksRoot}.");
    }

    public static DeckList Parse(string name, string text, string source)
    {
        ArgumentNullException.ThrowIfNull(text);

        List<DeckListEntry> entries = [];
        List<DeckListEntry> sideboard = [];
        string[] lines = text.Split('\n');
        bool afterSideboardLine = false;
        bool companionNext = false;
        string? companion = null;

        for (int index = 0; index < lines.Length; index++)
        {
            string line = lines[index].Trim();
            if (line.Length == 0 || line.StartsWith('#'))
            {
                continue;
            }

            // The one header this format has. Everything after it is outside the game.
            if (string.Equals(line, "Sideboard", StringComparison.OrdinalIgnoreCase))
            {
                afterSideboardLine = true;
                continue;
            }

            int number = index + 1;
            if (string.Equals(line, "Companion", StringComparison.OrdinalIgnoreCase))
            {
                companionNext = companion is null ? true : throw new FormatException($"{source} line {number}: a deck has one companion.");
                continue;
            }

            if (companionNext)
            {
                // "1 Lurrus of the Dream-Den" or the name alone: one card.
                Match one = Counted.Match(line);
                companion = !one.Success ? line : one.Groups[1].Value == "1" ? one.Groups[2].Value.Trim()
                    : throw new FormatException($"{source} line {number}: a companion is one card.");
                companionNext = false;
                continue;
            }

            List<DeckListEntry> into = afterSideboardLine ? sideboard : entries;
            if (Counted.Match(line) is { Success: true } counted)
            {
                int copies = int.Parse(counted.Groups[1].Value, CultureInfo.InvariantCulture);
                into.Add(copies > 0
                    ? new DeckListEntry(counted.Groups[2].Value.Trim(), copies, number)
                    : throw new FormatException($"{source} line {number}: a card needs at least one copy."));
            }
            else
            {
                into.Add(new DeckListEntry(line, 1, number));
            }
        }

        if (companionNext)
        {
            throw new FormatException($"{source}: the Companion line names no card after it.");
        }

        return new DeckList { Name = name, Source = source, Entries = entries, Sideboard = sideboard, Companion = companion };
    }

    /// <summary>
    /// The deck as card ids, resolved by name (ignoring case) against the cards a table loaded.
    /// </summary>
    /// <exception cref="InvalidOperationException">A name matches no loaded card, or more than one.</exception>
    public List<CardId> Resolve(IEnumerable<CardCatalog> catalogs) => Resolve(catalogs, Entries);

    /// <summary>The sideboard as card ids, resolved the same way.</summary>
    public List<CardId> ResolveSideboard(IEnumerable<CardCatalog> catalogs) => Resolve(catalogs, Sideboard);

    private List<CardId> Resolve(IEnumerable<CardCatalog> catalogs, IReadOnlyList<DeckListEntry> lines)
    {
        ArgumentNullException.ThrowIfNull(catalogs);

        Dictionary<string, CardId?> byName = new(StringComparer.OrdinalIgnoreCase);
        foreach (CardDefinition card in catalogs.SelectMany(catalog => catalog.InOrder()))
        {
            // Null marks a name two cards share, which a decklist cannot pick between.
            byName[card.Name] = byName.ContainsKey(card.Name) ? null : card.Id;
        }

        List<CardId> deck = [];
        foreach (DeckListEntry entry in lines)
        {
            CardId id = byName.TryGetValue(entry.Card, out CardId? found)
                ? found ?? throw new InvalidOperationException(
                    $"{Source} line {entry.Line}: more than one loaded card is named \"{entry.Card}\".")
                : throw new InvalidOperationException($"{Source} line {entry.Line}: no loaded card is named \"{entry.Card}\".");

            deck.AddRange(Enumerable.Repeat(id, entry.Copies));
        }

        return deck;
    }
}
