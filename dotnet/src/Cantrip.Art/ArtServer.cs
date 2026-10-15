using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Logging;
using Microsoft.Net.Http.Headers;

namespace Cantrip.Art;

/// <summary>
/// Card art over HTTP at <c>/art/{size}/{slug}.webp</c>, and who illustrated it at <c>/art/artists.json</c>.
/// <para>
/// The path is the cache folder's own layout, so the whole cache can be copied to any static host or CDN and clients
/// pointed at it with nothing else changing: a client composes the URL from a template it is configured with, never
/// from anything this server tells it. Answers carry ordinary HTTP caching - a week fresh, an ETag to revalidate
/// against - and allow any origin to read them, since card art is public and the browser client is served from
/// elsewhere.
/// </para>
/// </summary>
public static class ArtServer
{
    public const string Route = "/art/{size:int}/{slug}.webp";

    /// <summary>Who illustrated each picture (<see cref="ArtStore.CreditsFile"/>), beside the sizes as in the cache folder.</summary>
    public const string CreditsRoute = "/art/" + ArtStore.CreditsFile;

    private const string Fresh = "public, max-age=604800, stale-while-revalidate=86400";

    /// <summary>A missing card is asked about again after a while, not on every draw and not never.</summary>
    private const string Missing = "public, max-age=600";

    /// <param name="log">Where to write one line per request, as a CDN origin's access log would; null writes none.</param>
    public static IEndpointRouteBuilder MapCardArt(this IEndpointRouteBuilder endpoints, ArtStore store, ILogger? log = null)
    {
        ArgumentNullException.ThrowIfNull(endpoints);
        ArgumentNullException.ThrowIfNull(store);

        endpoints.MapMethods(Route, [HttpMethods.Get, HttpMethods.Head], async (int size, string slug, HttpContext context) =>
        {
            context.Response.OnStarting(() =>
            {
                log?.LogInformation("art {Size}/{Slug} {Status}", size, slug, context.Response.StatusCode);
                return Task.CompletedTask;
            });

            context.Response.Headers.AccessControlAllowOrigin = "*";
            context.Response.Headers.XContentTypeOptions = "nosniff";

            ArtRendition? rendition = await store.GetAsync(slug, size, context.RequestAborted).ConfigureAwait(false);
            if (rendition is null)
            {
                context.Response.Headers.CacheControl = Missing;
                return Results.NotFound();
            }

            context.Response.Headers.CacheControl = Fresh;

            // Results.File answers If-None-Match and If-Modified-Since itself, with a 304 and no body.
            return Results.File(
                rendition.Path,
                "image/webp",
                lastModified: rendition.LastModified,
                entityTag: EntityTagHeaderValue.Parse(rendition.ETag));
        });

        endpoints.MapMethods(CreditsRoute, [HttpMethods.Get, HttpMethods.Head], (HttpContext context) =>
        {
            context.Response.Headers.AccessControlAllowOrigin = "*";
            context.Response.Headers.XContentTypeOptions = "nosniff";

            // Asked about again as often as a missing picture is, since a credit is added with the picture it is for.
            context.Response.Headers.CacheControl = Missing;
            if (store.Credits is not { } credits)
            {
                return Results.NotFound();
            }

            return Results.File(
                credits.Path,
                "application/json",
                lastModified: credits.LastModified,
                entityTag: EntityTagHeaderValue.Parse(credits.ETag));
        });

        return endpoints;
    }

    /// <summary>Serves card art on its own, on its own port: beside a game server, or as the origin behind a CDN.</summary>
    public static async Task<WebApplication> StartAsync(ArtStore store, string listenAddress, int port, bool logRequests = false)
    {
        ArgumentNullException.ThrowIfNull(store);

        WebApplicationBuilder builder = WebApplication.CreateBuilder();
        builder.WebHost.UseUrls($"http://{listenAddress}:{port}");
        builder.Logging.SetMinimumLevel(LogLevel.Warning);
        builder.Services.AddRouting();

        builder.Logging.AddFilter("Cantrip.Art", logRequests ? LogLevel.Information : LogLevel.Warning);

        WebApplication app = builder.Build();
        app.MapCardArt(store, app.Services.GetRequiredService<ILoggerFactory>().CreateLogger("Cantrip.Art"));

        await app.StartAsync().ConfigureAwait(false);
        return app;
    }
}
