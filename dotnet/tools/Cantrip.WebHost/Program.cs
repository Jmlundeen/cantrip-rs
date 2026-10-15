using Cantrip.Server.Host;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Logging;

// Serves a built web client: the folder of static files `dotnet publish` makes of dotnet\src\Cantrip.Client.Web, as a real
// host would, compressed copies and all (WebClientFiles), so the shipping build can be played and measured locally
// (tools\play-release-build.cmd).
//
// Usage: cantrip-web-host --root PATH [--port N]

string? root = Arg("--root");
if (root is null || !Directory.Exists(root))
{
    Console.Error.WriteLine("Usage: cantrip-web-host --root PATH [--port N]  (PATH is the published wwwroot folder)");
    return 2;
}

int port = int.TryParse(Arg("--port"), out int given) ? given : 5100;
root = Path.GetFullPath(root);

WebApplicationBuilder builder = WebApplication.CreateBuilder();
builder.Logging.ClearProviders();
builder.WebHost.UseUrls($"http://localhost:{port}");

WebApplication app = builder.Build();
app.UseWebClient(root);

Console.WriteLine($"Serving {root} at http://localhost:{port}/");
await app.RunAsync();
return 0;

string? Arg(string name)
{
    int at = Array.IndexOf(args, name);
    return at >= 0 && at + 1 < args.Length ? args[at + 1] : null;
}
