using System.Text.Json;

namespace Cantrip.Server.Events;

/// <summary>
/// A cube: a list of card names, one a line, in the decklist format (<see cref="DeckList"/>) - so <c># comments</c> and
/// a count before a name both work, and a cube with two of a card lists it twice or as <c>2 Name</c>. Anything after a
/// <c>Sideboard</c> line is left out, as a cube's maybeboard.
/// </summary>
public static class CubeList
{
    /// <summary>The cube's cards by name, a name repeated for each copy.</summary>
    public static IReadOnlyList<string> Load(string path)
    {
        DeckList list = DeckList.Load(path);
        return [.. list.Entries.SelectMany(entry => Enumerable.Repeat(entry.Card, entry.Copies))];
    }

    /// <summary>
    /// The cube's pick order, best card first: <c>picks</c> in the <c>.picks.json</c> beside the cube file
    /// (<c>LSVCube.picks.json</c> for <c>LSVCube.txt</c>), written by <c>tools/maintenance/pick-order.py</c>. Empty for a cube without
    /// one. What a seat that is not choosing for itself - the pick clock, a bot - takes from a pack.
    /// </summary>
    public static IReadOnlyList<string> PickOrder(string path)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(path);
        string picks = Path.Combine(Path.GetDirectoryName(Path.GetFullPath(path)) ?? ".", Path.GetFileNameWithoutExtension(path) + ".picks.json");
        if (!File.Exists(picks))
        {
            return [];
        }

        using FileStream stream = File.OpenRead(picks);
        using JsonDocument document = JsonDocument.Parse(stream, new JsonDocumentOptions { CommentHandling = JsonCommentHandling.Skip, AllowTrailingCommas = true });
        return document.RootElement.TryGetProperty("picks", out JsonElement order)
            ? [.. order.EnumerateArray().Select(card => card.GetString()).OfType<string>()]
            : [];
    }

    /// <summary>
    /// The cube file a host named: a path that exists, else a <c>.txt</c> of that name in one of the directories given
    /// (<c>LSVCube</c> finds <c>docs/cube-lists/LSVCube.txt</c>).
    /// </summary>
    /// <exception cref="FileNotFoundException">No such file.</exception>
    public static string Find(string cube, params string[] directories)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(cube);
        if (File.Exists(cube))
        {
            return Path.GetFullPath(cube);
        }

        foreach (string directory in directories)
        {
            foreach (string candidate in (string[])[Path.Combine(directory, cube), Path.Combine(directory, cube + ".txt")])
            {
                if (File.Exists(candidate))
                {
                    return Path.GetFullPath(candidate);
                }
            }
        }

        throw new FileNotFoundException($"No cube '{cube}', as a path or in {string.Join(", ", directories)}.");
    }
}
