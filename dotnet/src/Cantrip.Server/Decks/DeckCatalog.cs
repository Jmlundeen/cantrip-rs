using System.Text.Json;
using Cantrip.Datasets;
using Cantrip.Decks;
using Cantrip.Projection;
using Cantrip.Protocol;

namespace Cantrip.Server.Decks;

/// <summary>The catalog the deck editor is built from: every content card as printed, with its datasets' legality data.</summary>
public static class DeckCatalog
{
    private const string FileName = "legality.json";

    private static readonly System.Collections.Concurrent.ConcurrentDictionary<string, Lazy<(CardCatalogView View, Dictionary<string, CatalogCard> ByName)>> Built = new(StringComparer.Ordinal);

    /// <summary>The catalog of a content root, built once and kept: what the server checks every deck against.</summary>
    public static CardCatalogView For(string contentRoot) => Kept(contentRoot).View;

    /// <summary>The catalog of a content root by card name, built once and kept.</summary>
    public static IReadOnlyDictionary<string, CatalogCard> ByName(string contentRoot) => Kept(contentRoot).ByName;

    private static (CardCatalogView View, Dictionary<string, CatalogCard> ByName) Kept(string contentRoot)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(contentRoot);
        return Built.GetOrAdd(Path.GetFullPath(contentRoot), root => new Lazy<(CardCatalogView, Dictionary<string, CatalogCard>)>(() =>
        {
            CardCatalogView view = Build(root);
            return (view, view.Cards.GroupBy(card => card.Name, StringComparer.Ordinal).ToDictionary(group => group.Key, group => group.First(), StringComparer.Ordinal));
        })).Value;
    }

    /// <summary>Every content card as printed, with its datasets' legality data: what <see cref="For"/> keeps.</summary>
    public static CardCatalogView Build(string contentRoot)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(contentRoot);
        ContentLibrary content = ContentCache.Get(contentRoot);
        var oracle = new CardOracle(content.Cards);
        (string date, Dictionary<string, LegalityEntry> legal) = Read(contentRoot);

        var catalog = new CardCatalogView { LegalityDate = date };
        foreach (Cantrip.Cards.Model.CardDefinition card in content.Cards.InOrder())
        {
            if (oracle.Oracle(card.Id) is not { } view) continue;
            LegalityEntry? entry = legal.GetValueOrDefault(view.Name);
            catalog.Cards.Add(new CatalogCard
            {
                Card = view,
                Identity = [.. (entry?.Identity ?? string.Empty).Select(Color).OfType<ColorView>()],
                Legalities = entry?.Legalities is { } legalities ? new Dictionary<string, string>(legalities, StringComparer.Ordinal) : [],
            });
        }

        return catalog;
    }

    /// <summary>One card of a <c>legality.json</c>.</summary>
    private sealed record LegalityEntry(string? Identity, Dictionary<string, string>? Legalities);

    private sealed record LegalityFile(string? Updated, Dictionary<string, LegalityEntry>? Cards);

    /// <summary>Every dataset's legality data, merged: the newest date, and each card by name.</summary>
    private static (string Date, Dictionary<string, LegalityEntry> Cards) Read(string contentRoot)
    {
        var options = new JsonSerializerOptions { PropertyNameCaseInsensitive = true };
        Dictionary<string, LegalityEntry> cards = new(StringComparer.Ordinal);
        string date = string.Empty;
        string root = Path.GetFullPath(contentRoot);
        IEnumerable<string> datasets = Directory.Exists(root) ? Directory.EnumerateDirectories(root).Where(DatasetDescriptor.IsDataset).Order(StringComparer.Ordinal) : [];
        foreach (string dataset in datasets)
        {
            string path = Path.Combine(dataset, FileName);
            if (!File.Exists(path)) continue;
            LegalityFile? file = JsonSerializer.Deserialize<LegalityFile>(File.ReadAllText(path), options);
            if (file?.Updated is { } updated && string.CompareOrdinal(updated, date) > 0) date = updated;
            foreach ((string name, LegalityEntry entry) in file?.Cards ?? []) cards.TryAdd(name, entry);
        }

        return (date, cards);
    }

    private static ColorView? Color(char letter) => letter switch
    {
        'W' => ColorView.White,
        'U' => ColorView.Blue,
        'B' => ColorView.Black,
        'R' => ColorView.Red,
        'G' => ColorView.Green,
        _ => null,
    };
}
