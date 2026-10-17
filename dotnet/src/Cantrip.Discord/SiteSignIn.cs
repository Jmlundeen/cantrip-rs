using System.Globalization;
using System.Security.Claims;
using System.Text.Json;
using Microsoft.AspNetCore.Authentication;
using Microsoft.AspNetCore.Authentication.Cookies;
using Microsoft.AspNetCore.Authentication.OAuth;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.DataProtection;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.Routing;
using Microsoft.Extensions.DependencyInjection;

namespace Cantrip.Discord;

/// <summary>A Discord account as signing in proved it: its id, the name it goes by, and its avatar's hash (null for none).</summary>
public sealed record WebAccount(ulong Id, string Name, string? Avatar);

public sealed class SiteSignInOptions
{
    /// <summary>
    /// The site's own public address, which Discord sends a player back to after they sign in: everything behind the web
    /// port, as <see cref="DiscordBotOptions.Client"/> is.
    /// </summary>
    public required Uri Site { get; init; }

    /// <summary>The Discord application's OAuth2 client id and secret; with either missing, nobody can sign in and the pages only show.</summary>
    public string? ClientId { get; init; }

    public string? ClientSecret { get; init; }

    /// <summary>Where the keys the sign-in cookie is sealed with are kept, so a restart leaves everyone signed in; null keeps them in memory.</summary>
    public string? Keys { get; init; }

    /// <summary>
    /// The player a signed-in account sits down as: their name on the Discord server, and their picture kept, as a Join in
    /// Discord does. Null takes the account's own name.
    /// </summary>
    public Func<WebAccount, Task<DiscordPlayer>>? Player { get; init; }

    public bool Enabled => !string.IsNullOrWhiteSpace(ClientId) && !string.IsNullOrWhiteSpace(ClientSecret);
}

/// <summary>
/// Signing in with Discord on the bot's site, for the pages a player takes a seat from - the draft queues
/// (<see cref="QueueSite"/>) and constructed play (<see cref="PlaySite"/>):
/// <code>
/// /signin?then=queue         to Discord and back to that page, signed in
/// /signin/discord            where Discord sends the player back
/// /signout                   POST
/// </code>
/// Discord's OAuth2 code flow with PKCE, asking only to identify the account; the token is used once, to learn who it
/// is, and dropped. What is kept is a cookie sealed by the server, naming the account, for thirty days.
/// </summary>
public static class SiteSignIn
{
    private const string Discord = "Discord";
    private const string AvatarClaim = "urn:discord:avatar";

    /// <summary>The pages a sign-in may come back to.</summary>
    private static readonly HashSet<string> Pages = new(StringComparer.Ordinal) { QueueSite.Route, PlaySite.Route };

    /// <summary>Where Discord sends a player back to once they have signed in; registered as a redirect of the application.</summary>
    public static Uri Callback(Uri site) => new(site, "signin/discord");

    /// <summary>The sign-in services: a cookie, and Discord to sign in with. Only when sign-in is set up.</summary>
    public static void AddSiteSignIn(this IServiceCollection services, SiteSignInOptions options)
    {
        ArgumentNullException.ThrowIfNull(options);
        if (!options.Enabled) return;

        IDataProtectionBuilder protection = services.AddDataProtection().SetApplicationName("cantrip-site");
        if (options.Keys is { } keys)
        {
            protection.PersistKeysToFileSystem(Directory.CreateDirectory(keys));
            if (OperatingSystem.IsWindows()) protection.ProtectKeysWithDpapi();
        }

        services.AddAuthentication(CookieAuthenticationDefaults.AuthenticationScheme)
            .AddCookie(cookie =>
            {
                cookie.Cookie.Name = "cantrip.signin";
                cookie.Cookie.HttpOnly = true;
                cookie.Cookie.SameSite = SameSiteMode.Lax;
                cookie.ExpireTimeSpan = TimeSpan.FromDays(30);
                cookie.SlidingExpiration = true;

                // The pages ask for JSON, so an answer that needs signing in is said, not redirected.
                cookie.Events.OnRedirectToLogin = context =>
                {
                    context.Response.StatusCode = StatusCodes.Status401Unauthorized;
                    return Task.CompletedTask;
                };
            })
            .AddOAuth(Discord, oauth =>
            {
                oauth.ClientId = options.ClientId!;
                oauth.ClientSecret = options.ClientSecret!;
                oauth.CallbackPath = "/signin/discord";
                oauth.AuthorizationEndpoint = "https://discord.com/oauth2/authorize";
                oauth.TokenEndpoint = "https://discord.com/api/oauth2/token";
                oauth.UserInformationEndpoint = "https://discord.com/api/users/@me";
                oauth.Scope.Add("identify");
                oauth.UsePkce = true;
                oauth.SaveTokens = false;

                // Discord sends the player back with a top-level GET, which a Lax cookie goes along with; None would need
                // HTTPS, and a server on this machine has none.
                oauth.CorrelationCookie.SameSite = SameSiteMode.Lax;
                oauth.ClaimActions.MapJsonKey(ClaimTypes.NameIdentifier, "id");
                oauth.ClaimActions.MapCustomJson(ClaimTypes.Name, user =>
                    user.TryGetProperty("global_name", out JsonElement shown) && shown.ValueKind == JsonValueKind.String ? shown.GetString() : user.GetProperty("username").GetString());
                oauth.ClaimActions.MapJsonKey(AvatarClaim, "avatar");
                oauth.Events.OnCreatingTicket = async context =>
                {
                    using var request = new HttpRequestMessage(HttpMethod.Get, context.Options.UserInformationEndpoint);
                    request.Headers.Authorization = new System.Net.Http.Headers.AuthenticationHeaderValue("Bearer", context.AccessToken);
                    using HttpResponseMessage response = await context.Backchannel.SendAsync(request, context.HttpContext.RequestAborted).ConfigureAwait(false);
                    response.EnsureSuccessStatusCode();
                    using JsonDocument user = JsonDocument.Parse(await response.Content.ReadAsStringAsync(context.HttpContext.RequestAborted).ConfigureAwait(false));
                    context.RunClaimActions(user.RootElement);
                };

                // Declined, or Discord could not be reached: back to the page, which says so.
                oauth.Events.OnRemoteFailure = context =>
                {
                    context.Response.Redirect($"{context.Properties?.RedirectUri ?? "/" + QueueSite.Route}?signin=failed");
                    context.HandleResponse();
                    return Task.CompletedTask;
                };
            });
    }

    /// <summary>
    /// Has sign-in's requests seen at the site's public address - its scheme and host - so the address Discord is asked to
    /// send a player back to is the one registered, behind a tunnel or a proxy too. Then signs them in. Goes after routing.
    /// </summary>
    public static void UseSiteSignIn(this IApplicationBuilder app, SiteSignInOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);
        if (!options.Enabled) return;

        app.Use((context, next) =>
        {
            if (context.Request.Path.StartsWithSegments("/signin"))
            {
                context.Request.Scheme = options.Site.Scheme;
                context.Request.Host = new HostString(options.Site.Authority);
            }

            return next(context);
        });
        app.UseAuthentication();
    }

    public static IEndpointRouteBuilder MapSiteSignIn(this IEndpointRouteBuilder app, SiteSignInOptions options)
    {
        ArgumentNullException.ThrowIfNull(app);
        ArgumentNullException.ThrowIfNull(options);

        app.MapGet("/signin", (string? then) =>
        {
            string page = "/" + (then is not null && Pages.Contains(then) ? then : QueueSite.Route);
            return options.Enabled
                ? Results.Challenge(new AuthenticationProperties { RedirectUri = page }, [Discord])
                : Results.Redirect(page);
        });

        app.MapPost("/signout", async (HttpContext context, NoBody _) =>
        {
            if (options.Enabled) await context.SignOutAsync(CookieAuthenticationDefaults.AuthenticationScheme).ConfigureAwait(false);
            return Results.NoContent();
        });

        return app;
    }

    /// <summary>The account signed in, or null.</summary>
    public static WebAccount? Account(HttpContext context)
    {
        ArgumentNullException.ThrowIfNull(context);
        return context.User.Identity?.IsAuthenticated == true
            && ulong.TryParse(context.User.FindFirstValue(ClaimTypes.NameIdentifier), NumberStyles.None, CultureInfo.InvariantCulture, out ulong id)
                ? new WebAccount(id, context.User.FindFirstValue(ClaimTypes.Name) ?? "Player", context.User.FindFirstValue(AvatarClaim))
                : null;
    }

    /// <summary>The player an account sits down as.</summary>
    public static Task<DiscordPlayer> PlayerOf(SiteSignInOptions options, WebAccount account)
    {
        ArgumentNullException.ThrowIfNull(options);
        ArgumentNullException.ThrowIfNull(account);
        return options.Player?.Invoke(account) ?? Task.FromResult(new DiscordPlayer(account.Id, account.Name));
    }

    /// <summary>
    /// The body of a POST that carries nothing else. Asked for anyway, as JSON: a page on another site cannot send JSON
    /// here without the browser first asking this server, which never says yes.
    /// </summary>
    public sealed record NoBody;

    /// <summary>A seat as a page takes it: the link that opens it in the web client, or why there is none.</summary>
    public sealed record SeatAnswer(string? Link, string? Refusal)
    {
        public static SeatAnswer For(Uri client, Uri? art, Uri? address, string? ticket, string? refusal) =>
            refusal is null && address is not null && ticket is not null
                ? new SeatAnswer(ClientSite.Link(client, address, ticket, art).AbsoluteUri, null)
                : new SeatAnswer(null, refusal ?? "No seat could be had.");
    }

    /// <summary>An answer of the pages' API: the server's time, and the value.</summary>
    public static IResult Fresh<T>(HttpContext context, T value)
    {
        ArgumentNullException.ThrowIfNull(context);
        context.Response.Headers.CacheControl = "no-store";
        return Results.Json(new { now = DateTimeOffset.UtcNow, value });
    }
}
