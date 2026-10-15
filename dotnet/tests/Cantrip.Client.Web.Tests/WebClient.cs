using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.StaticFiles;
using Microsoft.Extensions.FileProviders;
using Microsoft.Extensions.Logging;
using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// The web client as a browser gets it, and one browser to get it in, shared by every test: the client's Debug build
/// served as static files, and a Chromium launched once. A test opens its own page.
/// <para>
/// Headless unless <c>CANTRIP_WEB_HEADED=1</c>, which shows the browser so the tests can be watched; then the page is
/// as big as its window rather than a fixed size.
/// </para>
/// </summary>
public sealed class WebClient : IAsyncLifetime
{
    /// <summary>The size a headless page is, which is what every screenshot is taken at.</summary>
    public const int Width = 1440;
    public const int Height = 900;

    private WebApplication? _site;
    private IPlaywright? _playwright;

    public IBrowser Browser { get; private set; } = null!;

    /// <summary>The page's address, without a query.</summary>
    public Uri Site { get; private set; } = null!;

    public static string Root { get; } = FindRoot();

    public static string Content => Path.Combine(Root, "content");

    /// <summary>
    /// Where card art is rendered to and kept: the same cache the server and the scenario host use, so a machine that
    /// has served art before serves it again at once.
    /// </summary>
    public static string ArtCache { get; } =
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData), "Cantrip", "art-cache");

    public static bool Headed => Environment.GetEnvironmentVariable("CANTRIP_WEB_HEADED") == "1";

    /// <summary>Where a test's pictures go: TestResults\web under the repository, one folder per test class.</summary>
    public static string Pictures(string test)
    {
        string folder = Path.Combine(Root, "TestResults", "web", test);
        Directory.CreateDirectory(folder);
        return folder;
    }

    public async Task InitializeAsync()
    {
        string built = Path.Combine(Root, "dotnet", "src", "Cantrip.Client.Web", "bin", "Debug", "net10.0-browserwasm", "wwwroot");
        if (!File.Exists(Path.Combine(built, "index.html")))
        {
            throw new InvalidOperationException(
                $"The web client is not built at {built}. Build it first: dotnet build dotnet\\src\\Cantrip.Client.Web (tools\\test.cmd does).");
        }

        _site = await Serve(built);
        Site = new Uri(_site.Urls.First());

        _playwright = await Playwright.CreateAsync();
        Browser = await Launch(_playwright);
    }

    public async Task DisposeAsync()
    {
        if (Browser is not null)
        {
            await Browser.DisposeAsync();
        }

        _playwright?.Dispose();
        if (_site is not null)
        {
            await _site.DisposeAsync();
        }
    }

    /// <summary>
    /// Chrome or Edge if either is installed, so nothing has to be downloaded to run the suite; Playwright's own
    /// Chromium otherwise, which says how to install itself if it is not there either.
    /// </summary>
    private static async Task<IBrowser> Launch(IPlaywright playwright)
    {
        foreach (string? channel in (string?[])["chrome", "msedge", null])
        {
            try
            {
                return await playwright.Chromium.LaunchAsync(new BrowserTypeLaunchOptions { Channel = channel, Headless = !Headed });
            }
            catch (PlaywrightException) when (channel is not null)
            {
                // Not installed; try the next.
            }
        }

        throw new InvalidOperationException("No browser to launch.");
    }

    private static async Task<WebApplication> Serve(string root)
    {
        WebApplicationBuilder builder = WebApplication.CreateBuilder();
        builder.WebHost.UseUrls("http://127.0.0.1:0");
        builder.Logging.ClearProviders();
        WebApplication app = builder.Build();

        var types = new FileExtensionContentTypeProvider();
        types.Mappings[".wasm"] = "application/wasm";
        types.Mappings[".dat"] = "application/octet-stream";
        types.Mappings[".dll"] = "application/octet-stream";
        var files = new PhysicalFileProvider(root);
        app.UseDefaultFiles(new DefaultFilesOptions { FileProvider = files });
        app.UseStaticFiles(new StaticFileOptions { FileProvider = files, ContentTypeProvider = types, ServeUnknownFileTypes = true });
        await app.StartAsync();
        return app;
    }

    private static string FindRoot()
    {
        for (var directory = new DirectoryInfo(AppContext.BaseDirectory); directory is not null; directory = directory.Parent)
        {
            if (Directory.Exists(Path.Combine(directory.FullName, "content", "mtg")))
            {
                return directory.FullName;
            }
        }

        throw new DirectoryNotFoundException("No repository root (a folder holding content\\mtg) above the test assembly.");
    }
}

/// <summary>
/// Every browser test runs in this one collection: one browser for them all, and one page at a time, since each is a
/// whole WebAssembly runtime and a machine running several at once measures itself rather than the client.
/// </summary>
[CollectionDefinition(Name)]
public sealed class WebClientCollection : ICollectionFixture<WebClient>
{
    public const string Name = "web client";
}
