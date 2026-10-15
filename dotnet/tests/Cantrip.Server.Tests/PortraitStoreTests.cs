using Cantrip.Server.Events;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Players' pictures in the data directory: one to a player, a new one in place of the old, and shown by a path that
/// changes when it does.
/// </summary>
public sealed class PortraitStoreTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-portraits-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    [Fact]
    public async Task A_new_picture_takes_the_place_of_the_old_and_is_shown_by_a_new_path()
    {
        var portraits = new PortraitStore(_data);
        Assert.Null(portraits.PathFor("discord-42"));

        await portraits.SaveAsync("discord-42", [1, 2, 3]);
        string file = Path.Combine(_data, "portraits", "discord-42.png");
        File.SetLastWriteTimeUtc(file, new DateTime(2026, 10, 1, 0, 0, 0, DateTimeKind.Utc));
        string first = portraits.PathFor("discord-42")!;
        Assert.StartsWith("portraits/discord-42.png?v=", first, StringComparison.Ordinal);

        await portraits.SaveAsync("discord-42", [4, 5]);
        Assert.Equal([4, 5], await File.ReadAllBytesAsync(file));
        Assert.NotEqual(first, portraits.PathFor("discord-42"));
        Assert.Equal([file], Directory.GetFiles(Path.Combine(_data, "portraits")));
        Assert.Equal(file, portraits.Served("discord-42.png"));
    }

    [Fact]
    public async Task Nothing_outside_the_directory_is_written_or_served()
    {
        var portraits = new PortraitStore(_data);
        await Assert.ThrowsAsync<ArgumentException>(() => portraits.SaveAsync("../escape", [1]));
        Assert.Null(portraits.PathFor("../escape"));
        Assert.Null(portraits.Served("..%2Fescape.png"));
        Assert.Null(portraits.Served("../events.png"));
        Assert.Null(portraits.Served("nobody.png"));
        Assert.Null(portraits.Served("discord-42"));
    }
}
