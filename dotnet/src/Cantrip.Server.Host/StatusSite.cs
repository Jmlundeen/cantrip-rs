using System.Globalization;
using System.Reflection;
using Cantrip.Server.Events;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;

namespace Cantrip.Server.Host;

/// <summary>
/// The server's status page, open to everyone at <c>/status</c>: every event being played and the tables forming, and
/// the history of events over, filtered by when, kind, cube, player and how they ended - each opened for its draft,
/// pairings, results and standings. Only what <see cref="EventStatusBoard"/> makes public is served.
/// <code>
/// /status                       the page
/// /status/api/active            events being played, and the host's other tables
/// /status/api/history?...       events over: from, to (ISO 8601), kind, cube, player, outcome, limit
/// /status/api/events/{id}       one event
/// </code>
/// Every answer carries the server's time (<c>now</c>), which the page counts its clocks down from.
/// </summary>
public static class StatusSite
{
    public const string Route = "status";

    private static readonly Lazy<string> Page = new(() =>
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream("Cantrip.Server.Host.status.html")
            ?? throw new InvalidOperationException("The status page is not built into the host.");
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    });

    /// <summary>The page itself, for a test to see it is there.</summary>
    internal static string Html => Page.Value;

    public static IEndpointRouteBuilder MapEventStatus(this IEndpointRouteBuilder app, EventStatusBoard board)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(board);

        app.MapGet($"/{Route}", (HttpContext context) =>
        {
            context.Response.Headers.CacheControl = "no-cache";
            return Results.Content(Page.Value, "text/html; charset=utf-8");
        });

        app.MapGet($"/{Route}/api/active", (HttpContext context) => Fresh(context, board.Active()));

        app.MapGet($"/{Route}/api/history", (HttpContext context) =>
        {
            IQueryCollection query = context.Request.Query;
            return Fresh(context, board.History(new StatusQuery(
                Time(query["from"]),
                Time(query["to"]),
                Text(query["kind"]),
                Text(query["cube"]),
                Text(query["player"]),
                Text(query["outcome"]),
                int.TryParse(query["limit"], NumberStyles.Integer, CultureInfo.InvariantCulture, out int limit) ? limit : 200)));
        });

        app.MapGet($"/{Route}/api/events/{{id}}", (string id, HttpContext context) =>
            board.Detail(id) is { } detail ? Fresh(context, detail) : Results.NotFound());

        return app;
    }

    private static IResult Fresh<T>(HttpContext context, T value)
    {
        context.Response.Headers.CacheControl = "no-store";
        return Results.Json(new { now = DateTimeOffset.UtcNow, value });
    }

    private static string? Text(string? value) => string.IsNullOrWhiteSpace(value) ? null : value.Trim();

    private static DateTimeOffset? Time(string? value) =>
        DateTimeOffset.TryParse(value, CultureInfo.InvariantCulture, DateTimeStyles.AssumeUniversal, out DateTimeOffset time) ? time : null;
}
