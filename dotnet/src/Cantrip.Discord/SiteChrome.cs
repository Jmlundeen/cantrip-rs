using System.Net;
using System.Reflection;
using System.Text;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.DependencyInjection;

namespace Cantrip.Discord;

/// <summary>The site's sections, as its menu bar names them, for the page to light its own.</summary>
public enum SiteSection { None, Home, Draft, Leagues, Constructed, Decks, Events }

/// <summary>
/// What every page of the bot's site shares, apart from game tables: the menu bar - the logo, the sections the site has,
/// and who is signed in - the footer, and the stylesheet and script behind them (<c>site.css</c>, <c>site.js</c>). A page
/// is a template with markers the chrome fills in as it is served:
/// <code>
/// &lt;!--site:head--&gt;     the base address, the icon, the stylesheet and the script
/// &lt;!--site:header--&gt;   the menu bar, its section lit
/// &lt;!--site:footer--&gt;   the footer
/// </code>
/// One chrome serves the whole site, so every page links the same places by the same names. The sections not served -
/// constructed play with no Standard decks, drafts, leagues and decks with no data directory - are left out of the menu.
/// </summary>
public sealed class SiteChrome
{
    /// <summary>Where the shared stylesheet and script are, beside the pages.</summary>
    public const string StyleRoute = "site.css";

    public const string ScriptRoute = "site.js";

    public const string GitHub = "https://github.com/cantrip-rs/";

    public const string Discord = "https://discord.gg/x24qcVeT2w";

    private static readonly Lazy<string> Style = new(() => Resource("site.css"));
    private static readonly Lazy<string> Script = new(() => Resource("site.js"));

    /// <summary>A chrome for a page served on its own, with every section in its menu and nobody able to sign in.</summary>
    public static SiteChrome Default { get; } = new();

    public bool SignIn { get; init; }

    public bool Drafts { get; init; } = true;

    public bool Leagues { get; init; } = true;

    public bool Constructed { get; init; } = true;

    public bool Decks { get; init; } = true;

    public bool Events { get; init; } = true;

    /// <summary>The chrome the site was started with, or <see cref="Default"/>.</summary>
    public static SiteChrome For(HttpContext context)
    {
        ArgumentNullException.ThrowIfNull(context);
        return context.RequestServices.GetService<SiteChrome>() ?? Default;
    }

    /// <summary>A page's template, built into the bot.</summary>
    public static string Resource(string name)
    {
        using Stream stream = Assembly.GetExecutingAssembly().GetManifestResourceStream($"Cantrip.Discord.{name}")
            ?? throw new InvalidOperationException($"{name} is not built into the bot.");
        using var reader = new StreamReader(stream);
        return reader.ReadToEnd();
    }

    /// <summary>The page, its markers filled in for whoever asked, never kept by a cache: what it shows depends on who is signed in.</summary>
    public IResult Page(HttpContext context, string template, SiteSection section, int status = StatusCodes.Status200OK)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(template);
        context.Response.Headers.CacheControl = "no-cache";
        return Results.Content(Render(context, template, section), "text/html; charset=utf-8", Encoding.UTF8, status);
    }

    /// <summary>Fills a template's markers in.</summary>
    public string Render(HttpContext context, string template, SiteSection section)
    {
        ArgumentNullException.ThrowIfNull(context);
        ArgumentNullException.ThrowIfNull(template);
        WebAccount? account = SiteSignIn.Account(context);
        return template
            .Replace("<!--site:head-->", Head(context), StringComparison.Ordinal)
            .Replace("<!--site:header-->", Header(section, account), StringComparison.Ordinal)
            .Replace("<!--site:footer-->", Footer(), StringComparison.Ordinal);
    }

    /// <summary>Serves the stylesheet and the script every page shares.</summary>
    public static IResult Asset(HttpContext context, string name)
    {
        ArgumentNullException.ThrowIfNull(context);
        context.Response.Headers.CacheControl = "no-cache";
        return name == StyleRoute
            ? Results.Text(Style.Value, "text/css; charset=utf-8")
            : Results.Text(Script.Value, "text/javascript; charset=utf-8");
    }

    /// <summary>
    /// The base every link of the page is relative to - the site's root, wherever the page is - and what it loads. The
    /// script is not deferred, so the page's own script finds its helpers there; it waits for the page itself.
    /// </summary>
    private static string Head(HttpContext context)
    {
        string root = WebUtility.HtmlEncode(context.Request.PathBase.Value?.TrimEnd('/') + "/");
        return $"""
            <base href="{root}">
            <meta name="theme-color" content="#111318">
            <link rel="icon" type="image/png" href="{HomeSite.LogoRoute}">
            <link rel="stylesheet" href="{StyleRoute}">
            <script src="{ScriptRoute}"></script>
            """;
    }

    private string Header(SiteSection section, WebAccount? account)
    {
        var links = new StringBuilder();
        void Link(SiteSection which, string href, string label)
        {
            string current = which == section ? " aria-current=\"page\"" : string.Empty;
            links.Append($"""<a href="{href}"{current}>{label}</a>""");
        }

        Link(SiteSection.Home, HomeSite.Route, "Home");
        if (Drafts) Link(SiteSection.Draft, QueueSite.Route, "Draft");
        if (Leagues) Link(SiteSection.Leagues, LeagueSite.Route, "Leagues");
        if (Constructed) Link(SiteSection.Constructed, PlaySite.Route, "Constructed");
        if (Decks) Link(SiteSection.Decks, DeckSite.Route, "Decks");
        if (Events) Link(SiteSection.Events, Cantrip.Server.Host.StatusSite.Route, "Events");

        string who;
        if (account is not null)
        {
            string name = WebUtility.HtmlEncode(account.Name);
            string initial = WebUtility.HtmlEncode(account.Name.Length > 0 ? char.ToUpperInvariant(account.Name[0]).ToString() : "?");
            // Discord's own picture of the account, by its hash (hex, "a_" first for an animated one); else its initial.
            string picture = account.Avatar is { Length: > 0 and <= 64 } avatar && avatar.All(c => char.IsAsciiLetterOrDigit(c) || c == '_')
                ? $"""<img src="https://cdn.discordapp.com/avatars/{account.Id}/{avatar}.png?size=64" alt="" width="28" height="28" loading="lazy">"""
                : $"""<span aria-hidden="true">{initial}</span>""";
            who = $"""
                <a class="site-me" href="{HomeSite.Route}" title="Your page"><span class="site-avatar">{picture}</span><span class="site-name">{name}</span></a>
                <button type="button" class="button quiet small" data-site-signout>Sign out</button>
                """;
        }
        else if (SignIn)
        {
            who = """<a class="button discord small" href="signin" data-site-signin>Sign in with Discord</a>""";
        }
        else
        {
            who = string.Empty;
        }

        return $"""
            <a class="site-skip" href="#main">Skip to content</a>
            <header class="site-header">
              <div class="site-bar">
                <a class="site-logo" href="./" aria-label="Cantrip home"><img src="{HomeSite.LogoRoute}" alt="Cantrip" width="36" height="36"></a>
                <button type="button" class="site-menu-toggle" aria-expanded="false" aria-controls="site-nav" data-site-menu>
                  <span class="site-menu-icon" aria-hidden="true"></span><span class="visually-hidden">Menu</span>
                </button>
                <nav class="site-nav" id="site-nav" aria-label="Site">{links}</nav>
                <div class="site-account">{who}</div>
              </div>
            </header>
            <div class="site-banner" id="site-banner" role="status" hidden></div>
            """;
    }

    private static string Footer()
    {
        return $"""
            <footer class="site-footer">
              <div class="site-footer-inner">
                <a class="site-footer-logo" href="./" aria-label="Cantrip home"><img src="{HomeSite.LogoRoute}" alt="" width="24" height="24"><span>Cantrip</span></a>
                <nav aria-label="More">
                  <a href="{HomeSite.AboutRoute}">About</a>
                  <a href="{GitHub}">GitHub</a>
                  <a href="{Discord}">Discord</a>
                </nav>
              </div>
            </footer>
            """;
    }
}
