using Cantrip.Server.Events;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using static Cantrip.Discord.SiteSignIn;

namespace Cantrip.Discord;

public sealed class LeagueSiteOptions
{
    public required LeagueHosts Leagues { get; init; }

    /// <summary>The web client a seat opens in, and where its card art is (<see cref="ClientSite.Link"/>).</summary>
    public required Uri Client { get; init; }

    public Uri? Art { get; init; }

    /// <summary>Who is signed in, and the player they sit down as.</summary>
    public required SiteSignInOptions SignIn { get; init; }

    /// <summary>Who may open which table, when the tables are behind sign-in (<see cref="TableAccess"/>); null for none.</summary>
    internal TableAccess? Access { get; init; }
}

/// <summary>
/// The leagues on the bot's site: every league being run, and each league's own page, where a signed-in player enters
/// it, looks for matches, sees and changes their deck, and ends their run.
/// <code>
/// /leagues                   every league, and the player's standing in each
/// /leagues/{id}              a league's page
/// /leagues/api/list          the leagues, as the list shows them
/// /leagues/api/league/{id}   a league as the player sees it - which says they are still there, if they are looking for a match
/// /leagues/api/deck/{id}     ?run=ID: one of the player's own runs' deck
/// /leagues/api/enter         POST {league}: a new run, and a seat at its draft
/// /leagues/api/play          POST {league}: a seat wherever the run is being played - its draft, or its match
/// /leagues/api/search        POST {league}: look for a match
/// /leagues/api/stop          POST {league}: stop looking
/// /leagues/api/edit          POST {league}: a seat at the table the run's deck is changed at
/// /leagues/api/end           POST {league}: end the run
/// </code>
/// Names and records only, never a player id or a ticket; a deck only its own player's.
/// </summary>
public static class LeagueSite
{
    public const string Route = "leagues";

    private static readonly Lazy<string> ListPage = new(() => SiteChrome.Resource("leagues.html"));
    private static readonly Lazy<string> LeaguePage = new(() => SiteChrome.Resource("league.html"));

    /// <summary>A league's page on the site, which its tables lead back to.</summary>
    public static Uri PageOf(Uri site, string league) => new(site, $"{Route}/{Uri.EscapeDataString(league)}");

    public static IEndpointRouteBuilder MapLeagues(this IEndpointRouteBuilder app, LeagueSiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        app.MapGet($"/{Route}", (HttpContext context) => SiteChrome.For(context).Page(context, ListPage.Value, SiteSection.Leagues));
        app.MapGet($"/{Route}/{{id}}", (HttpContext context, string id) => options.Leagues.Find(id) is null
            ? SiteChrome.For(context).Page(context, SiteChrome.Resource("notfound.html"), SiteSection.None, StatusCodes.Status404NotFound)
            : SiteChrome.For(context).Page(context, LeaguePage.Value, SiteSection.Leagues));

        app.MapGet($"/{Route}/api/list", (HttpContext context) => Fresh(context, List(options, Account(context))));

        app.MapGet($"/{Route}/api/league/{{id}}", (HttpContext context, string id) =>
        {
            if (options.Leagues.Find(id) is not { } hosted) return Results.NotFound();
            WebAccount? account = Account(context);
            if (account is not null) options.Leagues.Seen(account.Id);
            return Fresh(context, League(options, hosted, account));
        });

        app.MapGet($"/{Route}/api/deck/{{id}}", (HttpContext context, string id, string? run) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            if (options.Leagues.Find(id) is not { } hosted || run is null) return Results.NotFound();
            return hosted.League.DeckOf(DraftHost.PlayerId(new DiscordPlayer(account.Id, account.Name)), run) is { } deck ? Fresh(context, deck) : Results.NotFound();
        });

        app.MapPost($"/{Route}/api/enter", async (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            DiscordPlayer player = await PlayerOf(options.SignIn, account).ConfigureAwait(false);
            return Fresh(context, Seat(options, account.Id, await options.Leagues.EnterAsync(request.League ?? string.Empty, player).ConfigureAwait(false)));
        });

        app.MapPost($"/{Route}/api/play", async (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            var player = new DiscordPlayer(account.Id, account.Name);
            return Fresh(context, Seat(options, account.Id, await options.Leagues.PlayAsync(request.League ?? string.Empty, player).ConfigureAwait(false)));
        });

        app.MapPost($"/{Route}/api/edit", async (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            var player = new DiscordPlayer(account.Id, account.Name);
            return Fresh(context, Seat(options, account.Id, await options.Leagues.EditAsync(request.League ?? string.Empty, player).ConfigureAwait(false)));
        });

        app.MapPost($"/{Route}/api/search", (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            return Fresh(context, new { refusal = options.Leagues.Search(request.League ?? string.Empty, new DiscordPlayer(account.Id, account.Name)) });
        });

        app.MapPost($"/{Route}/api/stop", (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            return Fresh(context, new { refusal = options.Leagues.StopSearching(request.League ?? string.Empty, new DiscordPlayer(account.Id, account.Name)) });
        });

        app.MapPost($"/{Route}/api/end", async (HttpContext context, LeagueRequest request) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            var player = new DiscordPlayer(account.Id, account.Name);
            return Fresh(context, new { refusal = await options.Leagues.EndRunAsync(request.League ?? string.Empty, player).ConfigureAwait(false) });
        });

        return app;
    }

    /// <summary>A league asked about, by its id.</summary>
    public sealed record LeagueRequest(string? League);

    private static SeatAnswer Seat(LeagueSiteOptions options, ulong player, LeagueSeating seating) =>
        SeatAnswer.For(options.Client, options.Art, options.Access, player, seating.Address, seating.Ticket, seating.Refusal);

    /// <summary>Every league, and - for a player signed in - how they stand in each.</summary>
    internal static object List(LeagueSiteOptions options, WebAccount? account)
    {
        string? player = account is null ? null : DraftHost.PlayerId(new DiscordPlayer(account.Id, account.Name));
        return new
        {
            signIn = options.SignIn.Enabled,
            me = account is null ? null : new { name = account.Name },
            leagues = options.Leagues.Leagues.Select(hosted =>
            {
                LeagueView view = hosted.League.View(player);
                return new
                {
                    id = hosted.Id,
                    name = view.Name,
                    description = hosted.Definition.Description,
                    cube = CubeName(hosted),
                    rules = Rules(view.Settings),
                    open = view.Open,
                    upcoming = view.Upcoming,
                    playing = view.Players.Count,
                    entrants = view.Entrants,
                    runs = view.RunsPlayed,
                    trophies = view.Trophies.Sum(each => each.Trophies),
                    searching = view.Searching,
                    leader = view.Trophies.FirstOrDefault(),
                    mine = view.Mine is { } mine ? Mine(mine) : null,
                };
            }),
        };
    }

    /// <summary>One league as its page shows it to whoever is looking.</summary>
    internal static object League(LeagueSiteOptions options, HostedLeague hosted, WebAccount? account)
    {
        string? player = account is null ? null : DraftHost.PlayerId(new DiscordPlayer(account.Id, account.Name));
        LeagueView view = hosted.League.View(player);
        return new
        {
            signIn = options.SignIn.Enabled,
            me = account is null ? null : new { name = account.Name },
            id = hosted.Id,
            name = view.Name,
            description = hosted.Definition.Description,
            cube = CubeName(hosted),
            rules = Rules(view.Settings),
            open = view.Open,
            upcoming = view.Upcoming,
            entrants = view.Entrants,
            runs = view.RunsPlayed,
            searching = view.Searching,
            players = view.Players.Select(Run),
            trophies = view.Trophies,
            recent = view.Recent,
            mine = view.Mine is { } mine ? Mine(mine) : null,
        };
    }

    /// <summary>A league's rules as the pages read them, each choice by its name.</summary>
    internal static object Rules(LeagueSettings rules) => new
    {
        format = rules.Format,
        podSize = rules.PodSize,
        packs = rules.Packs,
        packSize = rules.PackSize,
        minimumDeck = rules.MinimumDeck,
        gamesToWin = rules.GamesToWin,
        winsToTrophy = rules.WinsToTrophy,
        lossesToEnd = rules.LossesToEnd,
        deckEditing = rules.DeckEditing == LeagueDeckEditing.BetweenMatches ? "betweenMatches" : "untilFirstMatch",
        maxEntries = rules.MaxEntries,
        pairing = rules.Pairing switch { LeaguePairingMode.Free => "free", LeaguePairingMode.RecordStrict => "recordStrict", _ => "recordLoose" },
        allowRepeatPairings = rules.AllowRepeatPairings,
        starts = rules.Starts,
        ends = rules.Ends,
    };

    internal static string CubeName(HostedLeague hosted) =>
        string.IsNullOrWhiteSpace(hosted.Definition.CubeName) ? hosted.Definition.Settings.Cube : hosted.Definition.CubeName;

    internal static object Mine(LeagueMine mine) => new
    {
        current = mine.Current is { } current ? Run(current) : null,
        previous = mine.Previous.Select(Run),
        entriesUsed = mine.EntriesUsed,
        entriesLeft = mine.EntriesLeft,
        entryRefusal = mine.EntryRefusal,
        editRefusal = mine.EditRefusal,
        searchingSince = mine.SearchingSince,
        opponent = mine.Opponent,
        matchReady = mine.MatchReady,
        editing = mine.Editing,
        trophies = mine.Trophies,
    };

    /// <summary>A run, as anyone may see it: who, how it stands, whom it played. Its id is only for its own player to ask for its deck by.</summary>
    private static object Run(LeagueRunView run) => new
    {
        id = run.Id,
        name = run.Name,
        player = run.Player,
        entry = run.Entry,
        stage = run.Stage.ToString(),
        wins = run.Wins,
        losses = run.Losses,
        draws = run.Draws,
        ending = run.Ending,
        status = run.Status,
        started = run.Started,
        ended = run.Ended,
        deck = run.Deck,
        matches = run.Matches,
    };
}
