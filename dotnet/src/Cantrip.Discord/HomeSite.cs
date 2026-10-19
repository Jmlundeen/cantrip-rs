using System.Reflection;
using Cantrip.Server.Events;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using static Cantrip.Discord.SiteSignIn;

namespace Cantrip.Discord;

public sealed class HomeSiteOptions
{
    /// <summary>Who is signed in.</summary>
    public required SiteSignInOptions SignIn { get; init; }

    /// <summary>The draft queues, for the drafts a player is in; null when the bot keeps none.</summary>
    public QueueSiteOptions? Drafts { get; init; }

    /// <summary>The constructed games, for the one a player is in; null when the bot hosts none.</summary>
    public PlaySiteOptions? Constructed { get; init; }

    /// <summary>Whether the deck editor is there (<see cref="DeckSite"/>): only with a data directory to keep decks in.</summary>
    public bool HasDecks { get; init; }

    /// <summary>The data directory: each player's events, and their decks. Null for none.</summary>
    public EventStore? Store { get; init; }
}

/// <summary>
/// The site's front door and the page a signed-in player lands on.
/// <code>
/// /                          the home page for anyone not signed in: the logo, what Cantrip is, and where to go (served by
///                            ClientSite for a bare /); a player signed in goes on to /home
/// /about                     the same page, for anyone
/// /logo.png                  the Cantrip logo
/// /home                      a player's own page: what they are playing, their events being played and over, and where to go
/// /home/api/board            who is signed in, what they are in, and their events; ?brief=1 leaves out what is over
/// /home/api/drop             POST {event}: the player drops out of an event they are in
/// /home/api/leave            POST: the player gives up their seat at a table still filling
/// </code>
/// Rejoining is the draft (<see cref="QueueSite"/>) and constructed (<see cref="PlaySite"/>) pages' own API, so a seat is
/// given the same way from every page.
/// </summary>
public static class HomeSite
{
    public const string Route = "home";

    /// <summary>The home page as anyone sees it, signed in or not.</summary>
    public const string AboutRoute = "about";

    /// <summary>Where the logo is: beside the home page, whatever else the site serves.</summary>
    public const string LogoRoute = "logo.png";

    private static readonly Lazy<string> Landing = new(() => SiteChrome.Resource("landing.html"));
    private static readonly Lazy<string> Player = new(() => SiteChrome.Resource("home.html"));

    private static readonly Lazy<byte[]> LogoImage = new(() =>
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Cantrip.Discord.logo.png")
            ?? throw new InvalidOperationException("The logo is not built into the bot.");
        using var copy = new MemoryStream();
        stream.CopyTo(copy);
        return copy.ToArray();
    });

    /// <summary>The home page, for the site's address with nothing asked of it, and for /about.</summary>
    public static IResult LandingPage(HttpContext context) => SiteChrome.For(context).Page(context, Landing.Value, SiteSection.None);

    /// <summary>The logo, which every page's menu bar shows and names its tab's icon.</summary>
    public static IResult Logo(HttpContext context)
    {
        ArgumentNullException.ThrowIfNull(context);
        context.Response.Headers.CacheControl = "public, max-age=86400";
        return Results.File(LogoImage.Value, "image/png");
    }

    public static IEndpointRouteBuilder MapHome(this IEndpointRouteBuilder app, HomeSiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        app.MapGet($"/{AboutRoute}", LandingPage);
        app.MapGet($"/{Route}", (HttpContext context) => SiteChrome.For(context).Page(context, Player.Value, SiteSection.Home));

        // ?brief=1, as every page asks it for the seat a player has somewhere: what they are in, not what they have played.
        app.MapGet($"/{Route}/api/board", (HttpContext context, string? brief) => Fresh(context, Board(options, Account(context), brief is "1" or "true")));

        app.MapPost($"/{Route}/api/drop", async (HttpContext context, DropRequest drop) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            return Fresh(context, new { refusal = await DropAsync(options, account, drop.Event ?? string.Empty).ConfigureAwait(false) });
        });

        app.MapPost($"/{Route}/api/leave", async (HttpContext context, NoBody _) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            string? refusal = options.Drafts is { } drafts ? await drafts.Queues.LeaveAsync(account.Id).ConfigureAwait(false) : "There are no tables here.";
            return Fresh(context, new { refusal });
        });

        return app;
    }

    /// <summary>The event to drop out of, by its id.</summary>
    public sealed record DropRequest(string? Event);

    /// <summary>
    /// Drops a player out of an event they are in: a booster pod (<see cref="PodHosts.DropAsync"/>), or a Winston or grid
    /// draft (<see cref="DraftHost.DropAsync"/>). Null once they have; otherwise why not.
    /// </summary>
    internal static async Task<string?> DropAsync(HomeSiteOptions options, WebAccount account, string eventId)
    {
        var player = new DiscordPlayer(account.Id, account.Name);
        if (options.Drafts?.Pods is { } pods && pods.Hosts(eventId)) return await pods.DropAsync(eventId, player).ConfigureAwait(false);
        if (options.Drafts?.Drafts is { } drafts && drafts.Hosts(eventId)) return await drafts.DropAsync(eventId, player).ConfigureAwait(false);
        return "That event is not being played any more.";
    }

    /// <summary>What the pages show: whether sign-in is on, who is signed in, and what they are in. Names, never ids or tickets.</summary>
    internal static object Board(HomeSiteOptions options, WebAccount? account, bool brief = false)
    {
        string? player = account is null ? null : DraftHost.PlayerId(new DiscordPlayer(account.Id, account.Name));
        DraftHost? hosts = options.Drafts?.Drafts;
        PodHosts? pods = options.Drafts?.Pods;
        bool Hosted(string id) => hosts?.Hosts(id) == true || pods?.Hosts(id) == true;

        IReadOnlyList<PlayerEventSummary> events = player is not null && options.Store is { } store ? PlayerEvents.Of(store, player, Hosted, recent: brief ? 0 : 12) : [];
        PlayerEventSummary? playing = events.FirstOrDefault(each => each.Live && !each.Dropped);

        object? draft = null;
        if (account is not null && options.Drafts is { } drafts)
        {
            string? table = drafts.Queues.TableOf(account.Id);
            QueueBoardTable? forming = table is null ? null : drafts.Queues.Board().Tables.FirstOrDefault(each => each.Id == table);
            bool drafting = drafts.Drafts?.InDraft(account.Id) == true || drafts.Pods?.InDraft(account.Id) == true;
            if (forming is not null || drafting)
            {
                draft = new
                {
                    table,
                    drafting,
                    format = forming?.Settings.Format ?? playing?.Format,
                    cube = forming?.Cube ?? playing?.Cube,
                    players = forming?.Players,
                    seats = forming?.Seats,
                    checking = forming?.Checking == true,
                    @event = drafting ? playing?.Id : null,
                };
            }
        }

        object? constructed = null;
        if (account is not null && options.Constructed?.Games.GameOf(account.Id) is { } game)
        {
            constructed = new { id = game.Id, format = game.Format, stage = game.Stage.ToString(), players = game.Players.Select(each => each.Name) };
        }

        // A draft opened from Discord that nobody has joined yet: no event is made until someone does.
        IEnumerable<object> waiting = account is null || hosts is null ? [] : hosts.Waiting(account.Id).Select(each => new { id = each.Id, created = each.CreatedAt });

        int? decks = account is not null && !brief && options.HasDecks && options.Store is { } kept ? new DeckStore(kept).List(account.Id).Count : null;

        return new
        {
            signIn = options.SignIn.Enabled,
            me = account is null ? null : new { name = account.Name },
            hasDrafts = options.Drafts is not null,
            hasConstructed = options.Constructed is not null,
            hasDecks = options.HasDecks,
            draft,
            constructed,
            waiting,
            decks,
            events = events.Select(each => new
            {
                id = each.Id,
                name = each.Name,
                format = each.Format,
                cube = each.Cube,
                stage = each.Stage.ToString(),
                detail = each.Detail,
                created = each.Created,
                updated = each.Updated,
                live = each.Live,
                dropped = each.Dropped,
                wins = each.Wins,
                losses = each.Losses,
                draws = each.Draws,
                rank = each.Rank,
                players = each.Players,
                round = each.Round,
                rounds = each.Rounds,
                outcome = each.Outcome,
                opponents = each.Opponents,
                canDrop = each.Live && !each.Dropped && Hosted(each.Id),
            }),
        };
    }
}
