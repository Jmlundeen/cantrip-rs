using System.Text.Json;
using Cantrip.Client.Decks;
using Cantrip.Datasets;
using Cantrip.Projection;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;
using MessagePack;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using static Cantrip.Discord.SiteSignIn;

namespace Cantrip.Discord;

public sealed class DeckSiteOptions
{
    /// <summary>The content the catalog is every card of, and whose datasets' <c>legality.json</c> say where each is legal.</summary>
    public required string ContentRoot { get; init; }

    /// <summary>The data directory the decks are kept in, each player's in a folder of their own.</summary>
    public required EventStore Store { get; init; }

    /// <summary>Who is signed in.</summary>
    public required SiteSignInOptions SignIn { get; init; }

    public TimeProvider TimeProvider { get; init; } = TimeProvider.System;
}

/// <summary>
/// The constructed deck editor's half on the bot's site: the player's deck library, the cards a deck is built from, and
/// each player's saved decks. The editor itself is the web client (<c>/?decks</c>), which draws the cards as it draws a
/// draft's pool.
/// <code>
/// /decks                     the deck library: the player's decks by format, to open, copy, rename and delete, and a new one
/// /decks/edit?deck=ID        to the web client's deck editor, on that deck - or ?format=KEY for a new deck in a format
/// /decks/api/formats         the formats a deck is built for, by key and title
/// /decks/api/catalog         every card the engine plays, as printed, with its legalities and color identity (MessagePack)
/// /decks/api/list            the signed-in player's decks (DeckSummary), by format then name
/// /decks/api/deck/{id}       one of them (SavedDeck)
/// /decks/api/save            POST a SavedDeck: kept, legal or not; an empty id is a new deck (DeckSaved)
/// /decks/api/delete          POST {id}: gone
/// </code>
/// Decks are kept in the data directory as <c>decks/{player}/{deck}.json</c>, in the API's own shape (<see cref="DeckJson"/>).
/// A deck is refused only for being malformed - an unknown format, no name, an absurd count - never for being illegal:
/// a deck is checked in the editor, and saved however it stands.
/// </summary>
public static class DeckSite
{
    public const string Route = "decks";

    /// <summary>Where the editor is opened from the site: <c>/decks/edit?deck=ID</c>, or <c>?format=modern</c> for a new deck.</summary>
    public const string EditRoute = "edit";

    private static readonly Lazy<string> Library = new(() => SiteChrome.Resource("decks.html"));

    public static IEndpointRouteBuilder MapDecks(this IEndpointRouteBuilder app, DeckSiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);
        var decks = new DeckStore(options.Store, options.TimeProvider);
        var catalog = new Lazy<byte[]>(() => MessagePackSerializer.Serialize(DeckCatalog.Build(options.ContentRoot)));

        // The library is the site's page; the editor is the web client, opened on a deck, or a new one in a format.
        app.MapGet($"/{Route}", (HttpContext context) => SiteChrome.For(context).Page(context, Library.Value, SiteSection.Decks));
        app.MapGet($"/{Route}/{EditRoute}", (HttpContext context, string? deck, string? format) =>
        {
            string query = "?" + Route
                + (deck is { Length: > 0 } ? "&deck=" + Uri.EscapeDataString(deck) : string.Empty)
                + (format is { Length: > 0 } ? "&format=" + Uri.EscapeDataString(format) : string.Empty);
            return Results.Redirect(context.Request.PathBase + "/" + query);
        });

        app.MapGet($"/{Route}/api/formats", (HttpContext context) =>
            Fresh(context, DeckFormat.All.Select(format => new { key = format.Key, title = format.Title })));

        app.MapGet($"/{Route}/api/catalog", (HttpContext context) =>
        {
            context.Response.Headers.CacheControl = "public, max-age=3600";
            return Results.Bytes(catalog.Value, "application/x-msgpack");
        });

        app.MapGet($"/{Route}/api/list", (HttpContext context) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            return Json(context, decks.List(account.Id), DeckJson.Default.ListDeckSummary);
        });

        app.MapGet($"/{Route}/api/deck/{{id}}", (HttpContext context, string id) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            return decks.Load(account.Id, id) is { } deck ? Json(context, deck, DeckJson.Default.SavedDeck) : NotFound(context);
        });

        app.MapPost($"/{Route}/api/save", async (HttpContext context) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            if (!context.Request.HasJsonContentType()) return Results.StatusCode(StatusCodes.Status415UnsupportedMediaType);
            SavedDeck? deck;
            try
            {
                deck = await JsonSerializer.DeserializeAsync(context.Request.Body, DeckJson.Default.SavedDeck, context.RequestAborted).ConfigureAwait(false);
            }
            catch (JsonException)
            {
                deck = null;
            }

            DeckSaved saved = deck is null ? new DeckSaved(null, "That is not a deck.") : decks.Save(account.Id, deck);
            return Json(context, saved, DeckJson.Default.DeckSaved);
        });

        app.MapPost($"/{Route}/api/delete", (HttpContext context, DeleteRequest delete) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            context.Response.Headers.CacheControl = "no-store";
            return decks.Delete(account.Id, delete.Id ?? string.Empty) ? Results.NoContent() : Results.NotFound();
        });

        return app;
    }

    /// <summary>The deck to delete, by its id.</summary>
    public sealed record DeleteRequest(string? Id);

    private static IResult Json<T>(HttpContext context, T value, System.Text.Json.Serialization.Metadata.JsonTypeInfo<T> type)
    {
        context.Response.Headers.CacheControl = "no-store";
        return Results.Text(JsonSerializer.Serialize(value, type), "application/json; charset=utf-8");
    }

    private static IResult NotFound(HttpContext context)
    {
        context.Response.Headers.CacheControl = "no-store";
        return Results.NotFound();
    }
}

/// <summary>Each player's saved decks, in the data directory: <c>decks/{player}/{deck}.json</c>.</summary>
internal sealed class DeckStore(EventStore store, TimeProvider? clock = null)
{
    private const string Folder = "decks";

    /// <summary>The most entries a deck may list, across its commanders, deck and sideboard: past any real deck.</summary>
    private const int MostEntries = 300;

    private readonly TimeProvider _clock = clock ?? TimeProvider.System;

    private static string PlayerOf(ulong player) => DraftHost.PlayerId(new DiscordPlayer(player, string.Empty));

    /// <summary>A player's decks, by format in the order they are offered, then by name.</summary>
    public List<DeckSummary> List(ulong player)
    {
        List<SavedDeck> decks = [.. store.PlayerTextIds(Folder, PlayerOf(player)).Select(id => Load(player, id)).OfType<SavedDeck>()];
        return [.. decks
            .OrderBy(deck => DeckFormat.All.Select(format => format.Key).ToList().IndexOf(deck.Format) is int at and >= 0 ? at : int.MaxValue)
            .ThenBy(deck => deck.Name, StringComparer.OrdinalIgnoreCase)
            .ThenBy(deck => deck.Id, StringComparer.Ordinal)
            .Select(deck => new DeckSummary(deck.Id, deck.Name, deck.Format, deck.Commanders.Concat(deck.Main).Concat(deck.Side).Sum(entry => entry.Count), deck.Updated))];
    }

    /// <summary>One of a player's decks, or null for an id that is not one of theirs.</summary>
    public SavedDeck? Load(ulong player, string id)
    {
        if (!EventStore.ValidId(id)) return null;
        string? text = store.LoadPlayerText(Folder, PlayerOf(player), id);
        if (text is null) return null;
        try
        {
            SavedDeck? deck = JsonSerializer.Deserialize(text, DeckJson.Default.SavedDeck);
            if (deck is not null) deck.Id = id;
            return deck;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>Keeps a deck for a player, a new one when it has no id; refused only for being malformed.</summary>
    public DeckSaved Save(ulong player, SavedDeck deck)
    {
        ArgumentNullException.ThrowIfNull(deck);
        if (Malformed(deck) is { } refusal) return new DeckSaved(null, refusal);

        string id = deck.Id;
        if (string.IsNullOrEmpty(id))
        {
            id = EventStore.NewEventId("deck", _clock.GetUtcNow());
        }
        else if (Load(player, id) is null)
        {
            return new DeckSaved(null, "That deck is not one of yours.");
        }

        var kept = new SavedDeck
        {
            Id = id,
            Name = deck.Name.Trim(),
            Format = deck.Format,
            Commanders = [.. deck.Commanders.Select(Tidy)],
            Main = [.. deck.Main.Select(Tidy)],
            Side = [.. deck.Side.Select(Tidy)],
            Updated = _clock.GetUtcNow(),
        };
        store.SavePlayerText(Folder, PlayerOf(player), id, JsonSerializer.Serialize(kept, DeckJson.Default.SavedDeck));
        return new DeckSaved(kept, null);

        static DeckEntry Tidy(DeckEntry entry) => entry with { Card = entry.Card.Trim() };
    }

    /// <summary>Takes one of a player's decks away. Returns whether it was theirs to take.</summary>
    public bool Delete(ulong player, string id) =>
        EventStore.ValidId(id) && store.DeletePlayerText(Folder, PlayerOf(player), id);

    private static string? Malformed(SavedDeck deck)
    {
        if (DeckFormat.Find(deck.Format) is null) return $"\"{deck.Format}\" is not a format a deck is built for.";
        string name = deck.Name?.Trim() ?? string.Empty;
        if (name.Length == 0) return "A deck needs a name.";
        if (name.Length > 100) return "A deck's name is at most 100 characters.";
        List<DeckEntry> entries = [.. deck.Commanders ?? [], .. deck.Main ?? [], .. deck.Side ?? []];
        if (entries.Count > MostEntries) return $"A deck lists at most {MostEntries} cards.";
        foreach (DeckEntry entry in entries)
        {
            if (entry is null || string.IsNullOrWhiteSpace(entry.Card) || entry.Card.Trim().Length > 200) return "Every card of a deck is named.";
            if (entry.Count is < 1 or > 999) return $"{entry.Card.Trim()}: a count is from 1 to 999.";
        }

        deck.Commanders ??= [];
        deck.Main ??= [];
        deck.Side ??= [];
        return null;
    }
}

/// <summary>The catalog the deck editor is built from: every content card as printed, with its datasets' legality data.</summary>
internal static class DeckCatalog
{
    private const string FileName = "legality.json";

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
