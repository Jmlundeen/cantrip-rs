using System.Text.Json;
using System.Text.Json.Nodes;
using Cantrip.Cards;
using Cantrip.Cards.Model;

namespace Cantrip.Datasets;

/// <summary>Where a card is defined. Diagnostics only: a card is addressed by its name, never by where it lives.</summary>
public readonly record struct CardOrigin(string DatasetId, string Path);

/// <summary>
/// Every dataset a server has loaded, flattened into what drafting, deck building and a card list need: one pool of
/// cards keyed by name, as they are printed, and the set lists.
/// <para>
/// Flattened is the point. A set in one dataset may list cards another defines, and none of them says where the other
/// lives. Datasets are how content is packaged, not how it is addressed. A collision anywhere — two datasets defining one
/// card name, or claiming one set code — fails the load naming both, and is never resolved by load order.
/// </para>
/// <para>
/// Only characteristics are read (<see cref="CardFileReader"/>). The rules of a card are the native engine's, which reads
/// the same card files for itself, so nothing here compiles or runs a card's behaviour.
/// </para>
/// </summary>
public sealed class ContentLibrary
{
    private readonly List<DatasetDescriptor> _datasets = [];
    private readonly Dictionary<CardId, CardOrigin> _origins = [];
    private readonly Dictionary<CardId, JsonObject> _documents = [];
    private readonly SortedDictionary<string, SetList> _sets = new(StringComparer.OrdinalIgnoreCase);

    private ContentLibrary()
    {
    }

    /// <summary>The loaded datasets, in load order.</summary>
    public IReadOnlyList<DatasetDescriptor> Datasets => _datasets;

    /// <summary>Every card every dataset defines, keyed by name.</summary>
    public CardCatalog Cards { get; } = new();

    /// <summary>Where each card is defined.</summary>
    public IReadOnlyDictionary<CardId, CardOrigin> Origins => _origins;

    /// <summary>Every set list every dataset holds, ordered by code.</summary>
    public IReadOnlyList<SetList> Sets => [.. _sets.Values];

    /// <summary>A set by its code, ignoring case.</summary>
    /// <exception cref="KeyNotFoundException">No loaded dataset holds a set with that code.</exception>
    public SetList Set(string code)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(code);

        return _sets.TryGetValue(code.Trim(), out SetList? set)
            ? set
            : throw new KeyNotFoundException(
                $"No set '{code}'. Sets: {(_sets.Count == 0 ? "none" : string.Join(", ", _sets.Keys))}.");
    }

    /// <summary>
    /// Every card, with the named sets' overrides applied: a card a set specialises is that set's version of it, and
    /// every other card is as defined.
    /// </summary>
    /// <exception cref="KeyNotFoundException">A set code is not loaded.</exception>
    /// <exception cref="InvalidOperationException">Two of the named sets specialise the same card.</exception>
    public CardCatalog Catalog(params string[] setCodes)
    {
        ArgumentNullException.ThrowIfNull(setCodes);

        var cards = new CardCatalog { Cards = new Dictionary<CardId, CardDefinition>(Cards.Cards) };
        Dictionary<CardId, SetList> specialisedBy = [];

        foreach (string code in setCodes)
        {
            SetList set = Set(code);

            foreach ((CardId id, CardDefinition card) in set.Overrides)
            {
                if (specialisedBy.TryGetValue(id, out SetList? other) && !ReferenceEquals(other, set))
                {
                    throw new InvalidOperationException(
                        $"Sets '{other.Code}' and '{set.Code}' both override '{id}', so a game with both has no one "
                        + "answer for what that card is.");
                }

                specialisedBy[id] = set;
                cards.Cards[id] = card;
            }
        }

        return cards;
    }

    /// <summary>
    /// Loads every dataset directly under <paramref name="root"/>, in ordinal order of folder name so the load order is
    /// the same on every machine. A missing root is an empty library.
    /// </summary>
    public static ContentLibrary LoadAll(string root)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(root);

        if (!Directory.Exists(root))
        {
            return new ContentLibrary();
        }

        List<string> folders = [.. Directory.EnumerateDirectories(root).Where(DatasetDescriptor.IsDataset)];
        folders.Sort(static (a, b) => string.CompareOrdinal(Path.GetFileName(a), Path.GetFileName(b)));

        return Load(folders);
    }

    /// <summary>Loads named dataset folders, in the order given.</summary>
    public static ContentLibrary Load(IEnumerable<string> folders)
    {
        ArgumentNullException.ThrowIfNull(folders);

        var library = new ContentLibrary();
        List<DatasetDescriptor> descriptors = [.. folders.Select(DatasetDescriptor.Read)];

        library.CheckIds(descriptors);
        library._datasets.AddRange(descriptors);
        library.LoadCards(descriptors);
        library.LoadSets(descriptors);
        return library;
    }

    private void CheckIds(IReadOnlyList<DatasetDescriptor> descriptors)
    {
        var seen = new Dictionary<string, DatasetDescriptor>(StringComparer.Ordinal);

        foreach (DatasetDescriptor descriptor in descriptors)
        {
            if (seen.TryGetValue(descriptor.Id, out DatasetDescriptor? existing))
            {
                throw new DatasetLoadException(
                    descriptor.Folder,
                    $"two folders both claim the dataset id '{descriptor.Id}'; the other is {existing.Folder}.");
            }

            seen[descriptor.Id] = descriptor;
        }
    }

    /// <summary>
    /// Reads every card file of every dataset into the one pool. Every problem in every file is collected before
    /// failing, so one load tells an author everything that needs fixing.
    /// </summary>
    private void LoadCards(IReadOnlyList<DatasetDescriptor> descriptors)
    {
        List<string> problems = [];
        string? firstFolder = null;

        void Problem(DatasetDescriptor dataset, string message)
        {
            firstFolder ??= dataset.Folder;
            problems.Add(message);
        }

        foreach (DatasetDescriptor dataset in descriptors)
        {
            foreach (string path in JsonFiles(dataset.CardsDirectory))
            {
                string shown = Shown(dataset, path);
                string text = File.ReadAllText(path);

                JsonNode? node;
                try
                {
                    node = JsonNode.Parse(text, documentOptions: MergePatch.DocumentOptions);
                }
                catch (JsonException error)
                {
                    Problem(dataset, $"{shown}: not valid JSON: {error.Message}");
                    continue;
                }

                if (node is not JsonObject document)
                {
                    Problem(dataset, $"{shown}: a card file holds exactly one card, as a JSON object.");
                    continue;
                }

                CardDefinition card;
                try
                {
                    card = CardFileReader.Load(text);
                }
                catch (CardFileException error)
                {
                    foreach (string problem in error.Problems)
                    {
                        Problem(dataset, $"{shown}: {problem}");
                    }

                    continue;
                }

                string expected = CardFileName.For(card.Name);

                if (!string.Equals(Path.GetFileName(path), expected, StringComparison.Ordinal))
                {
                    Problem(dataset, $"{shown}: '{card.Name}' belongs in a file named {expected}.");
                    continue;
                }

                if (_origins.TryGetValue(card.Id, out CardOrigin existing))
                {
                    Problem(
                        dataset,
                        $"{shown}: '{card.Name}' is already defined in {existing.Path} (dataset '{existing.DatasetId}'). "
                        + "A card's name is its identity, so it is defined once; a set that prints it differently "
                        + "overrides it in its set list instead.");
                    continue;
                }

                Cards.Cards.Add(card.Id, card);
                _origins.Add(card.Id, new CardOrigin(dataset.Id, shown));
                _documents.Add(card.Id, document);
            }
        }

        if (problems.Count > 0)
        {
            throw new DatasetLoadException(firstFolder!, Describe("card", problems));
        }
    }

    /// <summary>
    /// Reads every set list. A set may list only cards some dataset defines, and its overrides are merged over those
    /// definitions and read here, so a bad override fails the load rather than the game that first uses the set.
    /// </summary>
    private void LoadSets(IReadOnlyList<DatasetDescriptor> descriptors)
    {
        List<string> problems = [];
        string? firstFolder = null;

        void Problem(DatasetDescriptor dataset, string message)
        {
            firstFolder ??= dataset.Folder;
            problems.Add(message);
        }

        foreach (DatasetDescriptor dataset in descriptors)
        {
            foreach (string path in JsonFiles(dataset.SetsDirectory))
            {
                string shown = Shown(dataset, path);
                int before = problems.Count;

                JsonObject? document;
                try
                {
                    document = JsonNode.Parse(File.ReadAllText(path), documentOptions: MergePatch.DocumentOptions) as JsonObject;
                }
                catch (JsonException error)
                {
                    Problem(dataset, $"{shown}: not valid JSON: {error.Message}");
                    continue;
                }

                string code = document?["code"] is JsonValue codeValue && codeValue.TryGetValue(out string? c)
                    ? c.Trim()
                    : string.Empty;

                if (document is null || code.Length == 0)
                {
                    Problem(dataset, $"{shown}: a set list is an object with a \"code\" and a \"cards\" array.");
                    continue;
                }

                string name = document["name"] is JsonValue nameValue && nameValue.TryGetValue(out string? n)
                    && !string.IsNullOrWhiteSpace(n)
                    ? n.Trim()
                    : code;

                if (_sets.TryGetValue(code, out SetList? existing))
                {
                    Problem(dataset, $"{shown}: set code '{code}' is already used by {existing.Path}.");
                    continue;
                }

                if (document["cards"] is not JsonArray entries)
                {
                    Problem(dataset, $"{shown}: set '{code}' has no \"cards\" array.");
                    continue;
                }

                List<CardId> listed = [];
                Dictionary<CardId, CardDefinition> overrides = [];

                foreach (JsonNode? entry in entries)
                {
                    (string? cardName, JsonObject? patch) = entry switch
                    {
                        JsonValue value when value.TryGetValue(out string? text) => (text, null),
                        JsonObject item when item["name"] is JsonValue value && value.TryGetValue(out string? text) =>
                            (text, item["override"] as JsonObject),
                        _ => (null, null),
                    };

                    if (string.IsNullOrWhiteSpace(cardName))
                    {
                        Problem(dataset, $"{shown}: every entry is a card name, or {{ \"name\": ..., \"override\": {{ ... }} }}.");
                        continue;
                    }

                    if (entry is JsonObject withOverride && withOverride.ContainsKey("override") && patch is null)
                    {
                        Problem(dataset, $"{shown}: the override of '{cardName}' is not a JSON object of card fields.");
                        continue;
                    }

                    var id = new CardId(cardName.Trim());

                    if (!Cards.Cards.ContainsKey(id))
                    {
                        Problem(
                            dataset,
                            $"{shown}: '{id}' is not defined in any dataset. A set lists cards that exist; it cannot "
                            + $"add one — define it in a dataset's cards/ folder as {CardFileName.For(id.Value)}.");
                        continue;
                    }

                    if (listed.Contains(id))
                    {
                        Problem(dataset, $"{shown}: '{id}' is listed twice.");
                        continue;
                    }

                    listed.Add(id);

                    if (patch is null)
                    {
                        continue;
                    }

                    if (patch.ContainsKey("name"))
                    {
                        Problem(dataset, $"{shown}: the override of '{id}' changes its name, and a name is a card's identity.");
                        continue;
                    }

                    string merged = MergePatch.Apply(_documents[id], patch)!.ToJsonString();

                    try
                    {
                        overrides[id] = CardFileReader.Load(merged);
                    }
                    catch (CardFileException error)
                    {
                        foreach (string problem in error.Problems)
                        {
                            Problem(dataset, $"{shown}: override of {problem}");
                        }
                    }
                }

                if (problems.Count == before)
                {
                    _sets.Add(code, new SetList(code, name, dataset.Id, shown, listed, overrides));
                }
            }
        }

        if (problems.Count > 0)
        {
            throw new DatasetLoadException(firstFolder!, Describe("set list", problems));
        }
    }

    /// <summary>Every <c>.json</c> file under a folder, recursively, in ordinal order of path. None if it is absent.</summary>
    private static List<string> JsonFiles(string directory)
    {
        if (!Directory.Exists(directory))
        {
            return [];
        }

        List<string> files = [.. Directory.EnumerateFiles(directory, "*.json", SearchOption.AllDirectories)];
        files.Sort(StringComparer.Ordinal);
        return files;
    }

    /// <summary>A file as a diagnostic names it: the dataset folder's name, then the path inside it.</summary>
    private static string Shown(DatasetDescriptor dataset, string path) =>
        $"{Path.GetFileName(dataset.Folder)}/{Path.GetRelativePath(dataset.Folder, path).Replace('\\', '/')}";

    private static string Describe(string what, IReadOnlyList<string> problems) =>
        $"{problems.Count} {what} problem(s):{Environment.NewLine}"
        + string.Join(Environment.NewLine, problems.Select(problem => $"  - {problem}"));
}
