namespace Cantrip.Client;

/// <summary>
/// Narrowing a long list of named options by what the player has typed, as the dock does for "choose a creature type"
/// (CR 205.3m), where a button per option would not fit. Matching ignores case: an option the text names exactly comes
/// first, then those it begins, then those it appears in, each group in the list's own order.
/// </summary>
public static class OptionFilter
{
    /// <summary>The indexes among <paramref name="among"/> whose option matches <paramref name="typed"/>, best first; all of them, in order, when nothing is typed.</summary>
    public static List<int> Matches(IReadOnlyList<string> options, IEnumerable<int> among, string typed)
    {
        string text = typed.Trim();
        List<int> candidates = [.. among.Where(index => index >= 0 && index < options.Count)];

        if (text.Length == 0)
        {
            return candidates;
        }

        List<int> exact = [.. candidates.Where(index => string.Equals(options[index], text, StringComparison.OrdinalIgnoreCase))];
        List<int> starting = [.. candidates.Where(index => !exact.Contains(index) && options[index].StartsWith(text, StringComparison.OrdinalIgnoreCase))];
        List<int> containing = [.. candidates.Where(index => !exact.Contains(index) && !starting.Contains(index)
            && options[index].Contains(text, StringComparison.OrdinalIgnoreCase))];
        return [.. exact, .. starting, .. containing];
    }

    /// <summary>
    /// The one option the typing settles on, or null: an option named exactly, else the only one that matches at all,
    /// which is what Enter picks.
    /// </summary>
    public static int? Settled(IReadOnlyList<string> options, IEnumerable<int> among, string typed)
    {
        if (typed.Trim().Length == 0)
        {
            return null;
        }

        List<int> matches = Matches(options, among, typed);
        return matches.Count == 1 || (matches.Count > 1 && string.Equals(options[matches[0]], typed.Trim(), StringComparison.OrdinalIgnoreCase))
            ? matches[0]
            : null;
    }
}
