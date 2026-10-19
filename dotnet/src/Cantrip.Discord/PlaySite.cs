using System.Reflection;
using Cantrip.Server;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using static Cantrip.Discord.SiteSignIn;

namespace Cantrip.Discord;

public sealed class PlaySiteOptions
{
    /// <summary>Hosts the games, each format's decks with it (<see cref="MatchmakerOptions.Formats"/>).</summary>
    public required Matchmaker Games { get; init; }

    /// <summary>The web client a seat opens in, and where its card art is (<see cref="ClientSite.Link"/>).</summary>
    public required Uri Client { get; init; }

    public Uri? Art { get; init; }

    /// <summary>Who is signed in, and the player they sit down as.</summary>
    public required SiteSignInOptions SignIn { get; init; }

    /// <summary>Who may open which table, when the tables are behind sign-in (<see cref="TableAccess"/>); null for none.</summary>
    internal TableAccess? Access { get; init; }
}

/// <summary>
/// The constructed play page, open to everyone at <c>/play</c>: a section for each constructed format the bot hosts - the
/// Standard gauntlet, its players each bringing one of the Standard decks - with every game waiting for an opponent and
/// every game being played, by its players' names. A player signed in with Discord (<see cref="SiteSignIn"/>) sets up a
/// game or joins one, choosing their deck as they sit down, and goes straight to their seat in the web client.
/// <code>
/// /play                      the page
/// /play/api/board            the formats and their decks, the games, and who is signed in
/// /play/api/host             POST {format, deck}: a game of the player's own, and its first seat
/// /play/api/join             POST {game, deck}: the other seat; the player's own seat again, for a game they are in
/// /play/api/cancel           POST {game}: calls off the player's own game
/// </code>
/// Nobody is shown anyone's deck: it is for the table to show.
/// </summary>
public static class PlaySite
{
    public const string Route = "constructed";

    private static readonly Lazy<string> Page = new(() =>
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Cantrip.Discord.play.html")
            ?? throw new InvalidOperationException("The constructed play page is not built into the bot.");
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    });

    public static IEndpointRouteBuilder MapConstructedPlay(this IEndpointRouteBuilder app, PlaySiteOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        // The decks each format offers, by name and as the list calls them; read once.
        var formats = options.Games.Formats.Select(format => new
        {
            key = format.Key,
            decks = format.Value.Select(deck => new { name = deck.Name, title = Title(deck) }).OrderBy(deck => deck.title, StringComparer.OrdinalIgnoreCase).ToList(),
        }).ToList();

        app.MapGet($"/{Route}", (HttpContext context) =>
            SiteChrome.For(context).Page(context, Page.Value, SiteSection.Constructed));

        app.MapGet($"/{Route}/api/board", (HttpContext context) =>
        {
            WebAccount? account = Account(context);
            HostedGame? mine = account is null ? null : options.Games.GameOf(account.Id);
            return Fresh(context, new
            {
                signIn = options.SignIn.Enabled,
                me = account is null ? null : new { name = account.Name, game = mine?.Id, format = mine?.Format },
                formats,
                games = options.Games.Formats.Keys.SelectMany(options.Games.Games).Select(game => new
                {
                    id = game.Id,
                    format = game.Format,
                    players = game.Players.Select(options.Games.FaceOf),
                    stage = game.Stage.ToString(),
                    created = game.CreatedAt,
                }),
            });
        });

        app.MapPost($"/{Route}/api/host", async (HttpContext context, HostRequest host) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            if (host.Format is not { Length: > 0 } format) return Fresh(context, Seat(options, account.Id, null, "Choose a format."));
            DiscordPlayer player = await PlayerOf(options.SignIn, account).ConfigureAwait(false);
            return Fresh(context, Seat(options, account.Id, await options.Games.HostAsync(player, format, host.Deck).ConfigureAwait(false)));
        });

        app.MapPost($"/{Route}/api/join", async (HttpContext context, JoinRequest join) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();

            // Only the games of a format: a game hosted from Discord is joined there.
            if (options.Games.Find(join.Game ?? string.Empty) is not { Format: not null })
                return Fresh(context, Seat(options, account.Id, null, "That game has ended."));
            DiscordPlayer player = await PlayerOf(options.SignIn, account).ConfigureAwait(false);
            return Fresh(context, Seat(options, account.Id, await options.Games.JoinAsync(join.Game!, player, join.Deck).ConfigureAwait(false)));
        });

        app.MapPost($"/{Route}/api/cancel", async (HttpContext context, CancelRequest cancel) =>
        {
            if (Account(context) is not { } account) return Results.Unauthorized();
            if (options.Games.Find(cancel.Game ?? string.Empty) is not { Format: not null } game) return Fresh(context, new { refusal = "That game has already ended." });
            string? refusal = await options.Games.CancelAsync(game.Id, new DiscordPlayer(account.Id, account.Name), moderator: false).ConfigureAwait(false);
            return Fresh(context, new { refusal });
        });

        return app;
    }

    public sealed record HostRequest(string? Format, string? Deck);

    public sealed record JoinRequest(string? Game, string? Deck);

    public sealed record CancelRequest(string? Game);

    private static SeatAnswer Seat(PlaySiteOptions options, ulong player, Seating seating) =>
        SeatAnswer.For(options.Client, options.Art, options.Access, player, seating.Game?.Address, seating.Ticket, seating.Refusal);

    private static SeatAnswer Seat(PlaySiteOptions options, ulong player, Seating? seating, string refusal) =>
        SeatAnswer.For(options.Client, options.Art, options.Access, player, null, null, seating?.Refusal ?? refusal);

    /// <summary>
    /// A deck as the list names it: its file's first comment up to a colon (<c># Mono-Red Aggro: a Standard meta deck</c>),
    /// else its file name in words.
    /// </summary>
    internal static string Title(DeckList deck)
    {
        try
        {
            if (File.ReadLines(deck.Source).FirstOrDefault() is { } first && first.StartsWith('#'))
            {
                string title = first.TrimStart('#').Split(':', 2)[0].Trim();
                if (title.Length > 0) return title;
            }
        }
        catch (IOException)
        {
            // Named from the file instead.
        }

        return string.Join(' ', deck.Name.Split('-', StringSplitOptions.RemoveEmptyEntries).Select(word => char.ToUpperInvariant(word[0]) + word[1..]));
    }
}
