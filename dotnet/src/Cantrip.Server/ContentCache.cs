using Cantrip.Datasets;

namespace Cantrip.Server;

/// <summary>
/// The content of a content root, loaded once per process.
/// <para>
/// Here rather than in each caller so a dedicated server, a Discord bot and a host-and-play client read the cards the
/// same way. What is loaded is each card as printed and the set lists (<see cref="ContentLibrary"/>): drafting and deck
/// building read it. The rules of a card are the native engine's, which reads the same files itself.
/// </para>
/// </summary>
public static class ContentCache
{
    /// <summary>
    /// The datasets under a content root, loaded once per process.
    /// <para>
    /// Held rather than reloaded because a card pool is read by every table and every pod. Reloading edited content is
    /// a development action; it is not something dealing a game should do behind the operator's back.
    /// </para>
    /// </summary>
    /// <param name="contentRoot">
    /// The directory holding datasets (<c>content/</c>). Passed in rather than discovered, because an exported game, a
    /// project run from the editor, and a server started from a shell all resolve paths differently, and guessing
    /// wrong produces a confusing empty pool instead of an error.
    /// </param>
    public static ContentLibrary Get(string contentRoot)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(contentRoot);

        string root = Path.GetFullPath(contentRoot);

        lock (Loaded)
        {
            if (!Loaded.TryGetValue(root, out ContentLibrary? library))
            {
                library = ContentLibrary.LoadAll(root);
                Loaded[root] = library;
            }

            return library;
        }
    }

    private static readonly Dictionary<string, ContentLibrary> Loaded = new(StringComparer.Ordinal);
}
