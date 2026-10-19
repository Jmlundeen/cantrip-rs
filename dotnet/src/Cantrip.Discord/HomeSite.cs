using System.Reflection;
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
}

/// <summary>
/// The site's front door and the page a signed-in player lands on.
/// <code>
/// /                          the home page: what Cantrip is, and where to go (served by ClientSite for a bare /)
/// /home                      a player's own page: the table or draft they are in and the game they are playing, to rejoin
/// /home/api/board            who is signed in, and what they are in
/// </code>
/// Rejoining is the draft (<see cref="QueueSite"/>) and constructed (<see cref="PlaySite"/>) pages' own API, so a seat is
/// given the same way from every page.
/// </summary>
public static class HomeSite
{
    public const string Route = "home";

    private static readonly Lazy<string> Landing = Read("landing");
    private static readonly Lazy<string> Player = Read("home");

    private static Lazy<string> Read(string name) => new(() =>
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream($"Cantrip.Discord.{name}.html")
            ?? throw new InvalidOperationException($"The {name} page is not built into the bot.");
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    });

    /// <summary>The home page, for the site's address with nothing asked of it.</summary>
    public static IResult LandingPage(HttpContext context)
    {
        context.Response.Headers.CacheControl = "no-cache";
        return Results.Content(Landing.Value, "text/html; charset=utf-8");
    }

    public static IEndpointRouteBuilder MapHome(this IEndpointRouteBuilder app, HomeSiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        app.MapGet($"/{Route}", (HttpContext context) =>
        {
            context.Response.Headers.CacheControl = "no-cache";
            return Results.Content(Player.Value, "text/html; charset=utf-8");
        });

        app.MapGet($"/{Route}/api/board", (HttpContext context) => Fresh(context, Board(options, Account(context))));
        return app;
    }

    /// <summary>What the pages show: whether sign-in is on, who is signed in, and what they are in. Names, never ids or tickets.</summary>
    internal static object Board(HomeSiteOptions options, WebAccount? account)
    {
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
                    format = forming?.Settings.Format,
                    cube = forming?.Cube,
                    players = forming?.Players,
                    seats = forming?.Seats,
                };
            }
        }

        object? constructed = null;
        if (account is not null && options.Constructed?.Games.GameOf(account.Id) is { } game)
        {
            constructed = new { id = game.Id, format = game.Format, stage = game.Stage.ToString(), players = game.Players.Select(player => player.Name) };
        }

        return new
        {
            signIn = options.SignIn.Enabled,
            me = account is null ? null : new { name = account.Name },
            hasDrafts = options.Drafts is not null,
            hasConstructed = options.Constructed is not null,
            hasDecks = options.HasDecks,
            draft,
            constructed,
            leagues = Array.Empty<object>(),
        };
    }
}
