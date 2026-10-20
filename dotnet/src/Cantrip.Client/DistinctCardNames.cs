namespace Cantrip.Client;

/// <summary>CR 201.2b: a split card can share either name with another card, not just its full display name.</summary>
public static class DistinctCardNames
{
    public static IEnumerable<T> Keep<T>(IEnumerable<T> cards, Func<T, IEnumerable<string>> names)
    {
        HashSet<string> used = [];
        foreach (T card in cards)
        {
            string[] current = names(card).Where(name => !string.IsNullOrWhiteSpace(name)).ToArray();
            if (current.Length == 0 || current.Any(used.Contains))
            {
                continue;
            }

            used.UnionWith(current);
            yield return card;
        }
    }
}
