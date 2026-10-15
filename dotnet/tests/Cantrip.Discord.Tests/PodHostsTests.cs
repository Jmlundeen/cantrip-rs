using System.Net;
using System.Net.Sockets;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The bot taking its booster drafts up again after a restart, from a data directory it may share with pods hosted
/// some other way - a pod played locally - which it leaves to their own hosts.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class PodHostsTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-pods-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    [Fact]
    public async Task Only_a_pod_that_came_from_a_queue_is_taken_up_again()
    {
        var store = new EventStore(_data);
        Create(store);
        string queued = Create(store);
        BoosterPod.Open(store, queued, ContentCache.Get(RepoRoot.ContentRoot)).Keep(PodHosts.QueueKey, "4242");

        await using var pods = new PodHosts(new PodHostsOptions { ContentRoot = RepoRoot.ContentRoot, Store = store, Ports = FreePorts(2) });
        Assert.Equal(1, await pods.ResumeAsync());
        Assert.Equal(1, pods.Underway(4242));
        Assert.Equal(0, pods.Underway(0));
    }

    private static string Create(EventStore store) => BoosterPod.Create(store, ContentCache.Get(RepoRoot.ContentRoot), new BoosterDraftSetup
    {
        Name = "Pod",
        Cube = "RW Aggro",
        CubeCards = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
        Packs = 1,
        PackSize = 5,
        Rounds = 1,
        Seed = 5,
        Players = [.. Enumerable.Range(1, 8).Select(seat => ($"p{seat}", $"Player {seat}"))],
    }).Record.Id;

    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }
}
