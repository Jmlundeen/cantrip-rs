using System.Globalization;
using System.Reflection;
using Cantrip.Server.Events;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using static Cantrip.Discord.SiteSignIn;

namespace Cantrip.Discord;

public sealed class QueueSiteOptions
{
    public required DraftQueues Queues { get; init; }

    /// <summary>The drafts under way, from the same board as the status page: only what everyone may see of them.</summary>
    public required EventStatusBoard Board { get; init; }

    /// <summary>For Back to my draft: the Winston and grid drafts, and the booster pods, a player may be in.</summary>
    public DraftHost? Drafts { get; init; }

    public PodHosts? Pods { get; init; }

    /// <summary>The web client a seat opens in, and where its card art is (<see cref="ClientSite.Link"/>).</summary>
    public required Uri Client { get; init; }

    public Uri? Art { get; init; }

    /// <summary>Who is signed in, and the player they sit down as.</summary>
    public required SiteSignInOptions SignIn { get; init; }

    /// <summary>Who may open which table, when the tables are behind sign-in (<see cref="TableAccess"/>); null for none.</summary>
    internal TableAccess? Access { get; init; }

    /// <summary>A queue's channel by name, as the page labels the queue; null, or a null answer, labels it by its cube alone.</summary>
    public Func<ulong, string?>? ChannelName { get; init; }
}

/// <summary>
/// The draft queue page, open to everyone at <c>/queue</c>: a section for each kind of draft, with every queue the Discord
/// bot has, every table still forming and who is at it, and every draft under way and where it has got to. A player who
/// signs in with Discord (<see cref="SiteSignIn"/>) joins a table from it - the same tables a queue channel's Join fills -
/// or sets one up with settings of their own, and goes straight to their seat in the web client.
/// <code>
/// /queue                     the page
/// /queue/api/board           the queues, the tables forming, the drafts under way, and who is signed in
/// /queue/api/join            POST {table} or {queue}: a seat, as a link to the web client
/// /queue/api/create          POST settings: a table of the player's own, and a seat at it
/// /queue/api/return          POST: the player's seat back in the draft they are in
/// </code>
/// A seat's link carries its ticket after the <c>#</c>, as every other link to a table does, and goes only to the player
/// it seats. The board shows each player's name, picture and Discord account (<see cref="PlayerFace"/>), never a ticket.
/// </summary>
public static class QueueSite
{
    public const string Route = "draft";

    private static readonly Lazy<string> Page = new(() =>
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Cantrip.Discord.queue.html")
            ?? throw new InvalidOperationException("The draft queue page is not built into the bot.");
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    });

    public static IEndpointRouteBuilder MapDraftQueues(this IEndpointRouteBuilder app, QueueSiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        app.MapGet($"/{Route}", (HttpContext context) =>
            SiteChrome.For(context).Page(context, Page.Value, SiteSection.Draft));

        app.MapGet($"/{Route}/api/board", (HttpContext context) => Fresh(context, Board(options, Account(context))));

        app.MapPost($"/{Route}/api/join", async (HttpContext context, JoinRequest join) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            DiscordPlayer player = await PlayerOf(options.SignIn, account).ConfigureAwait(false);
            QueueSeating seating = join.Table is { Length: > 0 } table
                ? await options.Queues.JoinTableAsync(table, player).ConfigureAwait(false)
                : ulong.TryParse(join.Queue, NumberStyles.None, CultureInfo.InvariantCulture, out ulong channel)
                    ? await options.Queues.JoinAsync(channel, player, web: true).ConfigureAwait(false)
                    : QueueSeating.Refused("Pick a table or a queue to join.");
            return Fresh(context, Seat(options, account.Id, seating.Address, seating.Ticket, seating.Refusal));
        });

        app.MapPost($"/{Route}/api/create", async (HttpContext context, CreateRequest create) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            DiscordPlayer player = await PlayerOf(options.SignIn, account).ConfigureAwait(false);
            QueueSeating seating = await options.Queues.CreateTableAsync(create.Settings(), player).ConfigureAwait(false);
            return Fresh(context, Seat(options, account.Id, seating.Address, seating.Ticket, seating.Refusal));
        });

        app.MapPost($"/{Route}/api/return", async (HttpContext context, NoBody _) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            var player = new DiscordPlayer(account.Id, account.Name);
            if (options.Pods is not null && await options.Pods.ReturnAsync(player).ConfigureAwait(false) is { Refusal: null } pod)
                return Fresh(context, Seat(options, account.Id, pod.Address, pod.Ticket, null));
            if (options.Drafts is null) return Fresh(context, Seat(options, account.Id, null, null, "You are not in a draft."));
            DraftSeating back = await options.Drafts.ReturnAsync(player).ConfigureAwait(false);
            return Fresh(context, Seat(options, account.Id, back.Draft?.Address, back.Ticket, back.Refusal));
        });

        return app;
    }

    /// <summary>A table or a queue to sit down at, by the board's ids.</summary>
    public sealed record JoinRequest(string? Table, string? Queue);

    /// <summary>A table of the player's own: what it drafts, as a queue's settings say it.</summary>
    public sealed record CreateRequest(string? Format, string? Cube, int Cards = 90, int Piles = 3, int Packs = 0, int PackSize = 15, int Rounds = 3, int MinimumDeck = 40, int Bots = 0)
    {
        public QueueSettings Settings() => new()
        {
            Format = Format?.Trim().ToLowerInvariant() ?? string.Empty,
            Cube = Cube?.Trim() ?? string.Empty,
            Cards = Cards,
            Piles = Piles,
            Packs = Packs,
            PackSize = PackSize,
            Rounds = Rounds,
            MinimumDeck = MinimumDeck,
            Bots = Bots,
        };
    }

    private static SeatAnswer Seat(QueueSiteOptions options, ulong player, Uri? address, string? ticket, string? refusal) =>
        SeatAnswer.For(options.Client, options.Art, options.Access, player, address, ticket, refusal);

    /// <summary>Everything the page shows, and what the player signed in has: their name, the table they sit at, whether they are drafting.</summary>
    internal static object Board(QueueSiteOptions options, WebAccount? account)
    {
        QueueBoard board = options.Queues.Board();
        HashSet<string> formats = [DraftFormats.Booster, DraftFormats.Grid, DraftFormats.Winston];
        return new
        {
            signIn = options.SignIn.Enabled,
            me = account is null ? null : new
            {
                name = account.Name,
                table = options.Queues.TableOf(account.Id),
                drafting = options.Drafts?.InDraft(account.Id) == true || options.Pods?.InDraft(account.Id) == true,
            },
            boosters = options.Pods is not null,
            cubes = options.Queues.Cubes(),
            queues = board.Queues.Select(queue => new
            {
                // As text: a Discord id is past what a number in JavaScript holds exactly.
                id = queue.Settings.Channel.ToString(CultureInfo.InvariantCulture),
                channel = options.ChannelName?.Invoke(queue.Settings.Channel),
                settings = Settings(queue.Settings, queue.Cube),
            }),
            tables = board.Tables.Select(table => new
            {
                id = table.Id,
                queue = table.Channel == 0 ? null : table.Channel.ToString(CultureInfo.InvariantCulture),
                settings = Settings(table.Settings, table.Cube),
                players = table.Players,
                seats = table.Seats,
                checking = table.Checking,
                created = table.Created,
            }),
            underway = options.Board.Active().Events.Where(summary => summary.Live && formats.Contains(summary.Format)).Select(summary => new
            {
                id = summary.Id,
                name = summary.Name,
                format = summary.Format,
                cube = summary.Cube,
                stage = summary.Stage,
                progress = summary.Progress,
                round = summary.Round,
                rounds = summary.Rounds,
                created = summary.Created,
                players = summary.Players.Select(player => new { name = player.Name, dropped = player.Dropped, picture = player.Picture, discord = player.Discord }),
            }),
        };
    }

    private static object Settings(QueueSettings settings, string cube) => new
    {
        format = settings.Format,
        cube,
        cards = settings.Cards,
        piles = settings.Piles,
        packs = settings.PacksDrafted,
        packSize = settings.PackSize,
        rounds = settings.Rounds,
        minimumDeck = settings.MinimumDeck,
        seats = settings.Seats,
        bots = settings.Bots,
    };
}
