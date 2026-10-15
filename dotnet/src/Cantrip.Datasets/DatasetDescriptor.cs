using System.Text.Json;
using System.Text.RegularExpressions;

namespace Cantrip.Datasets;

/// <summary>
/// What a dataset folder declares about itself, read from <c>dataset.json</c>.
/// <para>
/// Deliberately short, because everything else about a dataset is its folder layout. A dataset is a folder with a
/// <c>dataset.json</c> and any of these subfolders, each holding one kind of asset and each optional:
/// </para>
/// <list type="bullet">
/// <item><c>cards/</c> — card definitions, one card per <c>.json</c> file, named for the card.</item>
/// <item><c>sets/</c> — set lists: cards that exist somewhere, with sparse overrides for that set.</item>
/// <item><c>dungeons/</c> — dungeon room graphs. Read by nothing here.</item>
/// <item><c>images/</c> — reserved for card images; not read yet.</item>
/// </list>
/// <para>
/// Nothing is listed in the descriptor because nothing needs to be: every file under those folders is found by
/// looking. That is what keeps a card atomic — adding one is adding a file, and two people adding two cards never
/// edit the same file.
/// </para>
/// </summary>
public sealed partial class DatasetDescriptor
{
    /// <summary>The file a dataset folder is recognised by.</summary>
    public const string FileName = "dataset.json";

    /// <summary>Card definitions: <c>cards/**/*.json</c>, one card per file.</summary>
    public const string CardsFolder = "cards";

    /// <summary>Set lists: <c>sets/**/*.json</c>.</summary>
    public const string SetsFolder = "sets";

    /// <summary>Card images. Reserved: the loader does not read it yet.</summary>
    public const string ImagesFolder = "images";

    /// <summary>
    /// The dataset's id: lowercase letters, digits and hyphens. Names the dataset in a game's content stamp and in
    /// diagnostics, and nothing else — no card, script or set is addressed through it.
    /// </summary>
    public required string Id { get; init; }

    /// <summary>Display name.</summary>
    public required string Name { get; init; }

    /// <summary>The dataset's own version string, opaque to the host.</summary>
    public string Version { get; init; } = "0";

    /// <summary>The folder this descriptor was read from.</summary>
    public string Folder { get; init; } = string.Empty;

    public string CardsDirectory => Path.Combine(Folder, CardsFolder);

    public string SetsDirectory => Path.Combine(Folder, SetsFolder);

    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
    };

    /// <summary>Whether a folder is a dataset: whether it holds a <c>dataset.json</c>.</summary>
    public static bool IsDataset(string folder) =>
        !string.IsNullOrWhiteSpace(folder) && File.Exists(Path.Combine(folder, FileName));

    /// <summary>Reads and validates <c>dataset.json</c> from a dataset folder.</summary>
    /// <exception cref="DatasetLoadException">It is missing, unreadable, has a bad id, or was written for another card API.</exception>
    public static DatasetDescriptor Read(string folder)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(folder);

        string path = Path.Combine(folder, FileName);
        if (!File.Exists(path))
        {
            throw new DatasetLoadException(folder, $"no {FileName}. A dataset is a folder with a {FileName} in it.");
        }

        Document document;
        try
        {
            document = JsonSerializer.Deserialize<Document>(File.ReadAllText(path), Options)
                ?? throw new DatasetLoadException(folder, $"{FileName} is empty.");
        }
        catch (JsonException error)
        {
            throw new DatasetLoadException(folder, $"{FileName} is not valid JSON: {error.Message}");
        }

        string id = document.Id?.Trim() ?? string.Empty;

        if (id.Length == 0)
        {
            throw new DatasetLoadException(folder, $"{FileName} has no \"id\".");
        }

        if (!IdPattern().IsMatch(id))
        {
            throw new DatasetLoadException(
                folder, $"dataset id '{id}' may use only lowercase letters, digits and hyphens.");
        }

        return new DatasetDescriptor
        {
            Id = id,
            Name = string.IsNullOrWhiteSpace(document.Name) ? id : document.Name.Trim(),
            Version = string.IsNullOrWhiteSpace(document.Version) ? "0" : document.Version.Trim(),
            Folder = Path.GetFullPath(folder),
        };
    }

    [GeneratedRegex("^[a-z0-9][a-z0-9-]*$", RegexOptions.CultureInvariant)]
    private static partial Regex IdPattern();

    /// <summary>The JSON shape, kept private so the public descriptor can be validated and normalised.</summary>
    private sealed class Document
    {
        public string? Id { get; set; }

        public string? Name { get; set; }

        public string? Version { get; set; }
    }
}
