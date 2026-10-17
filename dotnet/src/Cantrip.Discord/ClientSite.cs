using Cantrip.Art;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;

namespace Cantrip.Discord;

/// <summary>
/// The web client every ticket reply opens, and the link that opens a seat in it. A player clicks it in Discord and is
/// at the table in their browser: nothing to install, nothing to register.
/// <para>
/// The client is static and knows nothing of any table. The link names the table in its query and carries the ticket
/// after its <c>#</c>, as <c>/?server=ws%3A%2F%2Fhost%3A7782%2Fgame&amp;art=...#ticket=...</c>. A fragment never
/// leaves the browser, so the ticket is in no request and no log, and the client takes it out of the address bar as
/// the page loads (its Ticket.js). Because nothing about the site is per-game, one serves every table - and it could
/// as well be served from any static host or CDN: <see cref="DiscordBotOptions.Client"/> is just a URL.
/// </para>
/// </summary>
internal static class ClientSite
{
    /// <summary>The link a player clicks: the client, with the table and art in its query and the ticket after the hash.</summary>
    /// <param name="art">Where card art is, as the client's <c>?art=</c> takes it; null leaves the client to its default.</param>
    public static Uri Link(Uri client, Uri address, string ticket, Uri? art) => new UriBuilder(client)
    {
        Query = $"server={Uri.EscapeDataString(address.AbsoluteUri)}" + (art is null ? string.Empty : $"&art={Uri.EscapeDataString(art.AbsoluteUri)}"),
        Fragment = $"ticket={Uri.EscapeDataString(ticket)}",
    }.Uri;

    /// <summary>
    /// Serves the built web client in <paramref name="root"/>, and card art beside it at <c>/art</c>, on one port until
    /// disposed; and, given <paramref name="tables"/>, every one of those table ports at <c>/table/{port}/game</c>
    /// (<see cref="TableProxy"/>), so that one public address reaches everything; and, given <paramref name="portraits"/>,
    /// players' pictures at <c>/portraits/{player}.png</c>, where the tables tell the client they are; and, given
    /// <paramref name="status"/>, the server's status page at <c>/status</c> (<see cref="StatusSite"/>); and, given
    /// <paramref name="queues"/>, the draft queue page at <c>/queue</c> (<see cref="QueueSite"/>); and, given
    /// <paramref name="play"/>, the constructed play page at <c>/play</c> (<see cref="PlaySite"/>) - each signing players in
    /// with <paramref name="signIn"/> (<see cref="SiteSignIn"/>).
    /// </summary>
    public static async Task<WebApplication> StartAsync(string listen, int port, string root, ArtStore art, IReadOnlySet<int>? tables = null, PortraitStore? portraits = null,
        EventStatusBoard? status = null, SiteSignInOptions? signIn = null, QueueSiteOptions? queues = null, PlaySiteOptions? play = null)
    {
        WebApplicationBuilder builder = WebApplication.CreateBuilder();
        builder.WebHost.UseUrls($"http://{listen}:{port}");
        builder.Logging.SetMinimumLevel(LogLevel.Warning);
        builder.Services.AddRouting();
        if (signIn is not null) builder.Services.AddSiteSignIn(signIn);
        WebApplication app = builder.Build();

        // A ping every 30 seconds: a tunnel or proxy in front (Cloudflare closes a WebSocket silent for 100 seconds) would
        // otherwise drop a player who is only thinking - over a pile, or a deck.
        app.UseWebSockets(new WebSocketOptions { KeepAliveInterval = TimeSpan.FromSeconds(30) });
        app.UseWebClient(root);
        app.UseRouting();
        if (signIn is not null) app.UseSiteSignIn(signIn);
        app.MapCardArt(art);
        if (portraits is not null)
        {
            // Asked for by a path that names when the picture was written, so a browser may keep each one for good.
            app.MapGet($"/{PortraitStore.Route}/{{name}}", (string name, HttpContext context) =>
            {
                if (portraits.Served(name) is not { } file) return Results.NotFound();
                context.Response.Headers.CacheControl = "public, max-age=31536000, immutable";
                return Results.File(file, "image/png");
            });
        }

        if (status is not null)
        {
            app.MapEventStatus(status);
        }

        if (signIn is not null)
        {
            app.MapSiteSignIn(signIn);
        }

        if (queues is not null)
        {
            app.MapDraftQueues(queues);
        }

        if (play is not null)
        {
            app.MapConstructedPlay(play);
        }

        if (tables is not null)
        {
            // The tables listen where the bot told them to; reached here from this machine.
            app.MapTables(listen is "0.0.0.0" or "*" or "+" ? "127.0.0.1" : listen, tables);
        }

        await app.StartAsync().ConfigureAwait(false);
        return app;
    }
}
