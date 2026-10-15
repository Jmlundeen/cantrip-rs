using System.Net;
using System.Net.Sockets;
using System.Net.WebSockets;
using Cantrip.Art;
using Cantrip.Client;
using Cantrip.Server;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// Every table behind the bot's one web port, as an HTTPS tunnel in front of it needs: the address a player is given,
/// a seat taken through the web port by its ticket, and no way through to a port that is not one of the bot's tables.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class TableProxyTests
{
    [Fact]
    public void Behind_a_public_url_a_table_is_a_path_on_it_and_https_becomes_wss()
    {
        Assert.Equal("wss://cantrip.example.com/table/7790/game", TableProxy.Address(new Uri("https://cantrip.example.com/"), 7790).AbsoluteUri);
        Assert.Equal("ws://localhost:7779/table/7781/game", TableProxy.Address(new Uri("http://localhost:7779/"), 7781).AbsoluteUri);
        Assert.Equal("wss://example.com/cantrip/table/7790/game", TableProxy.Address(new Uri("https://example.com/cantrip/"), 7790).AbsoluteUri);
    }

    [Fact]
    public async Task A_player_takes_their_seat_through_the_web_port_and_nothing_else_is_reachable_through_it()
    {
        string root = Directory.CreateTempSubdirectory("cantrip-table-proxy").FullName;
        try
        {
            await File.WriteAllTextAsync(Path.Combine(root, "index.html"), "<!doctype html><title>Cantrip</title>");
            List<int> free = FreePorts(3);
            (int web, int table, int other) = (free[0], free[1], free[2]);
            var site = new Uri($"http://127.0.0.1:{web}/");

            DeckList deck = DeckList.Load(DeckList.Find(Path.Combine(RepoRoot.Path, "decks"), "rw-aggro"));
            await using var matchmaker = new Matchmaker(new MatchmakerOptions
            {
                ContentRoot = RepoRoot.ContentRoot,
                Deck = deck,
                Ports = [table],
                TableAddress = port => TableProxy.Address(site, port),
            });
            var art = new ArtStore(new ArtStoreOptions { ContentRoot = root, CacheDirectory = Path.Combine(root, "art") });
            await using var server = await ClientSite.StartAsync("127.0.0.1", web, root, art, new HashSet<int> { table });

            Seating seating = await matchmaker.HostAsync(new DiscordPlayer(1, "Alice"));
            Assert.Equal($"ws://127.0.0.1:{web}/table/{table}/game", seating.Game!.Address.AbsoluteUri);

            var alice = new GameClient(await WebSocketTransport.ConnectAsync(seating.Game.Address));
            alice.Join("ignored", ticket: seating.Ticket);
            DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(30);
            while (alice.Membership is null && alice.Errors.Count == 0 && DateTime.UtcNow < giveUp)
            {
                alice.Pump();
                await Task.Delay(5);
            }

            Assert.Equal(1, alice.Membership?.Seat);
            alice.Disconnect();

            // A port that is not one of the bot's tables is not passed through to.
            using var probe = new ClientWebSocket();
            await Assert.ThrowsAnyAsync<WebSocketException>(() => probe.ConnectAsync(new Uri($"ws://127.0.0.1:{web}/table/{other}/game"), CancellationToken.None));
        }
        finally
        {
            Directory.Delete(root, recursive: true);
        }
    }

    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }
}
