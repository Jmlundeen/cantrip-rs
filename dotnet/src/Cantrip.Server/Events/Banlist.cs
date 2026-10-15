using System.Reflection;
using System.Text.Json;

namespace Cantrip.Server.Events;

/// <summary>
/// The cards each draft format leaves out of its cube, by name: those that do nothing in it, as Cogwork Librarian does
/// where no pack is passed. Kept in <c>banlist.json</c> beside this file, which the server carries with it.
/// </summary>
public static class Banlist
{
    private static readonly Lazy<IReadOnlyDictionary<string, HashSet<string>>> ByFormat = new(Load);

    /// <summary>The cards banned in a format (<see cref="DraftFormats"/>), by name; none for a format without a list.</summary>
    public static IReadOnlySet<string> For(string format) =>
        ByFormat.Value.TryGetValue(format, out HashSet<string>? banned) ? banned : new HashSet<string>();

    /// <summary>Whether a card is banned in a format.</summary>
    public static bool IsBanned(string format, string card) => For(format).Contains(card);

    private static IReadOnlyDictionary<string, HashSet<string>> Load()
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Cantrip.Server.Events.banlist.json")
            ?? throw new InvalidOperationException("The server was built without its banlist.");
        using JsonDocument document = JsonDocument.Parse(stream, new JsonDocumentOptions { CommentHandling = JsonCommentHandling.Skip, AllowTrailingCommas = true });
        Dictionary<string, HashSet<string>> lists = new(StringComparer.Ordinal);
        foreach (JsonProperty format in document.RootElement.EnumerateObject())
        {
            lists[format.Name] = new HashSet<string>(format.Value.EnumerateArray().Select(card => card.GetString()).OfType<string>(), StringComparer.OrdinalIgnoreCase);
        }

        return lists;
    }
}
