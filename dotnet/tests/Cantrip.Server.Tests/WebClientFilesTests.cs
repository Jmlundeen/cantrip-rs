using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Which built web client a host serves, in a checkout laid out in a folder of the test's own: the newer of the Release
/// publish and the Debug build, and a warning when the one served was built before the protocol last changed - which is
/// what players would otherwise only find out from "Expected protocol" on opening their table.
/// </summary>
public sealed class WebClientFilesTests : IDisposable
{
    private readonly string _root = Path.Combine(Path.GetTempPath(), "cantrip-client-files-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_root))
        {
            Directory.Delete(_root, recursive: true);
        }
    }

    private string Client(string relative, DateTime builtAt)
    {
        string folder = Path.Combine(_root, "dotnet", "src", "Cantrip.Client.Web", "bin", relative);
        Directory.CreateDirectory(folder);
        string index = Path.Combine(folder, "index.html");
        File.WriteAllText(index, "<html></html>");
        File.SetLastWriteTimeUtc(index, builtAt);
        return folder;
    }

    private void ProtocolChanged(DateTime at)
    {
        string folder = Path.Combine(_root, "dotnet", "src", "Cantrip.Protocol");
        Directory.CreateDirectory(folder);
        string file = Path.Combine(folder, "Messages.cs");
        File.WriteAllText(file, "// the wire");
        File.SetLastWriteTimeUtc(file, at);
    }

    [Fact]
    public void The_newer_of_the_release_publish_and_the_debug_build_is_served()
    {
        var day = new DateTime(2026, 10, 1, 12, 0, 0, DateTimeKind.Utc);
        string release = Client(Path.Combine("Release", "net10.0-browserwasm", "publish", "wwwroot"), day);
        string debug = Client(Path.Combine("Debug", "net10.0-browserwasm", "wwwroot"), day.AddHours(3));

        Assert.Equal(debug, WebClientFiles.Built(_root));

        File.SetLastWriteTimeUtc(Path.Combine(release, "index.html"), day.AddHours(5));
        Assert.Equal(release, WebClientFiles.Built(_root));
    }

    [Fact]
    public void A_client_built_before_the_protocol_last_changed_is_said_to_be_stale()
    {
        var day = new DateTime(2026, 10, 1, 12, 0, 0, DateTimeKind.Utc);
        string release = Client(Path.Combine("Release", "net10.0-browserwasm", "publish", "wwwroot"), day);

        ProtocolChanged(day.AddHours(-1));
        Assert.Null(WebClientFiles.Stale(release, _root));

        ProtocolChanged(day.AddHours(1));
        Assert.Contains("Expected protocol", WebClientFiles.Stale(release, _root), StringComparison.Ordinal);
    }
}
