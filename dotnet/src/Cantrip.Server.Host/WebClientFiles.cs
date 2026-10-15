using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using Microsoft.AspNetCore.StaticFiles;
using Microsoft.Extensions.FileProviders;

namespace Cantrip.Server.Host;

/// <summary>
/// Serves a built web client: the folder of static files a build or <c>dotnet publish</c> makes of
/// dotnet\src\Cantrip.Client.Web, as any static host would. What the Discord bot hands its players and dotnet\tools\Cantrip.WebHost
/// serves locally.
/// <para>
/// A publish writes a compressed copy - gzip (.gz), and brotli (.br) unless told not to - beside every file it can
/// compress. A browser that accepts one is sent it, with the file's own type, which is most of what a player downloads:
/// the runtime is 67 MB as it stands and 11-15 MB compressed, the difference between a table a home connection can
/// serve to a crowd and one it cannot.
/// </para>
/// </summary>
public static class WebClientFiles
{
    /// <summary>Serves the client in <paramref name="root"/> (the folder holding its index.html) at the site's root.</summary>
    public static WebApplication UseWebClient(this WebApplication app, string root)
    {
        ArgumentNullException.ThrowIfNull(app);

        var types = new FileExtensionContentTypeProvider();
        types.Mappings[".wasm"] = "application/wasm";
        types.Mappings[".dat"] = "application/octet-stream";
        types.Mappings[".blat"] = "application/octet-stream";
        types.Mappings[".dll"] = "application/octet-stream";
        types.Mappings[".webmanifest"] = "application/manifest+json";

        var files = new PhysicalFileProvider(Path.GetFullPath(root));

        // A request for a file that has a compressed copy the browser accepts is answered with that copy.
        app.Use(async (context, next) =>
        {
            string path = context.Request.Path.Value ?? "/";
            string accepts = context.Request.Headers.AcceptEncoding.ToString();

            foreach ((string encoding, string extension) in (ReadOnlySpan<(string, string)>)[("br", ".br"), ("gzip", ".gz")])
            {
                if (!accepts.Contains(encoding, StringComparison.OrdinalIgnoreCase) || path.EndsWith('/'))
                {
                    continue;
                }

                if (files.GetFileInfo(path + extension) is { Exists: true } && types.TryGetContentType(path, out string? type))
                {
                    context.Request.Path = path + extension;
                    context.Response.Headers.ContentEncoding = encoding;
                    context.Response.Headers.Vary = "Accept-Encoding";
                    context.Response.OnStarting(() =>
                    {
                        context.Response.ContentType = type;
                        return Task.CompletedTask;
                    });
                    break;
                }
            }

            await next(context);
        });

        // Asked again each time, so a new build is what the next reload gets; the hashed framework files still come from
        // the browser's cache when they have not changed.
        app.UseDefaultFiles(new DefaultFilesOptions { FileProvider = files });
        app.UseStaticFiles(new StaticFileOptions
        {
            FileProvider = files,
            ContentTypeProvider = types,
            ServeUnknownFileTypes = true,
            OnPrepareResponse = context => context.Context.Response.Headers.CacheControl = "no-cache",
        });

        return app;
    }

    /// <summary>
    /// Where this checkout's web client is built: the Release publish tools\play-release-build.cmd makes, or the Debug build
    /// any <c>dotnet build</c> leaves - whichever was built last, so a publish left behind by later changes is not served
    /// in preference to a build that has them. Null when neither is there.
    /// </summary>
    public static string? Built(string repositoryRoot)
    {
        string bin = Path.Combine(repositoryRoot, "dotnet", "src", "Cantrip.Client.Web", "bin");
        return ((string[])[
                Path.Combine(bin, "Release", "net10.0-browserwasm", "publish", "wwwroot"),
                Path.Combine(bin, "Debug", "net10.0-browserwasm", "wwwroot")])
            .Where(candidate => File.Exists(Path.Combine(candidate, "index.html")))
            .OrderByDescending(BuiltAt)
            .FirstOrDefault();
    }

    /// <summary>When a built client was built: its newest file.</summary>
    public static DateTime BuiltAt(string root) =>
        Directory.EnumerateFiles(root, "*", SearchOption.AllDirectories).Select(File.GetLastWriteTimeUtc).DefaultIfEmpty(DateTime.MinValue).Max();

    /// <summary>
    /// Why a built client would refuse every table - it was built before the wire protocol last changed, so it asks for
    /// an older protocol than the server speaks - or null when it was built since. Read from the checkout's protocol
    /// sources, so only for a client built from this checkout.
    /// </summary>
    public static string? Stale(string root, string repositoryRoot)
    {
        string protocol = Path.Combine(repositoryRoot, "dotnet", "src", "Cantrip.Protocol");
        if (!Directory.Exists(protocol)) return null;
        DateTime changed = Directory.EnumerateFiles(protocol, "*.cs", SearchOption.AllDirectories)
            .Where(file => !file.Contains($"{Path.DirectorySeparatorChar}obj{Path.DirectorySeparatorChar}", StringComparison.Ordinal))
            .Select(File.GetLastWriteTimeUtc).DefaultIfEmpty(DateTime.MinValue).Max();
        DateTime built = BuiltAt(root);
        return changed > built
            ? $"The web client in {root} was built {built.ToLocalTime():g}, before the wire protocol last changed ({changed.ToLocalTime():g}): players will be refused with \"Expected protocol\". Run tools\\play-release-build.cmd (or tools\\test.cmd), then start this again."
            : null;
    }
}
