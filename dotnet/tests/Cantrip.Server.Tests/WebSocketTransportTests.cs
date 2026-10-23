using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Host;
using Cantrip.Server.Rules;
using Cantrip.TestKit;
using Xunit;
using Xunit.Abstractions;

namespace Cantrip.Server.Tests;

/// <summary>
/// A whole game played over a real WebSocket against a real Kestrel server dealing through the real native host.
/// <para>
/// The loopback transport deliberately skips serialisation, so nothing else exercises the MessagePack contracts, the
/// frame reassembly, or the single-threaded pump that keeps concurrent connections from touching the game at once. A
/// break in any of those would be invisible to every other test. On the wall clock because it is a socket and a process;
/// it skips, saying so, when the host binary has not been built (<c>cargo build --profile cards -p cantrip-host</c> at
/// the repository root).
/// </para>
/// </summary>
[Trait("Category", "WallClock")]
public sealed class WebSocketTransportTests
{
    /// <summary>
    /// Generous on purpose: this is an integration test against a real socket, and a tight bound would make it fail
    /// on a loaded machine rather than on a real defect.
    /// </summary>
    private static readonly TimeSpan Patience = TimeSpan.FromMinutes(2);

    private readonly ITestOutputHelper _output;

    public WebSocketTransportTests(ITestOutputHelper output)
    {
        _output = output;
    }

    [Fact]
    public async Task Two_clients_play_a_whole_game_over_a_real_socket()
    {
        if (!File.Exists(NativeHostProcess.Locate()))
        {
            _output.WriteLine($"Skipped: no cantrip-native-host at {NativeHostProcess.Locate()}.");
            return;
        }

        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Players = 2,
            Seed = 4,
            ContentRoot = RepoRoot.ContentRoot,
            Decks = DeckList.LoadDirectory(RepoRoot.DemoDecks).Take(2).ToList(),
            AutoStart = true,
        });

        List<(GameClient Client, RandomPlay Bot)> players = [];
        foreach (string name in (string[])["Alice", "Bob"])
        {
            WebSocketTransport transport = await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{server.Port}/game"));
            var client = new GameClient(transport);
            client.Join(name, 0);
            players.Add((client, new RandomPlay(seed: 100 + players.Count)));
        }

        DateTime deadline = DateTime.UtcNow + Patience;
        int idle = 0;

        while (!players.TrueForAll(player => player.Client.IsOver))
        {
            if (DateTime.UtcNow > deadline)
            {
                throw new TimeoutException($"No end within {Patience.TotalSeconds:F0}s: {string.Join(", ", players.Select(p => $"seat {p.Client.Seat} turn {p.Client.View?.Turn}"))}");
            }

            bool acted = false;
            foreach ((GameClient client, RandomPlay bot) in players)
            {
                client.Pump();
                acted |= !client.IsOver && bot.Act(client);
            }

            if (!acted && ++idle % 64 == 0)
            {
                await Task.Delay(1);
            }
            else
            {
                await Task.Yield();
            }
        }

        foreach ((GameClient client, RandomPlay _) in players)
        {
            Assert.Empty(client.Errors);
            GameOverMessage over = Assert.IsType<GameOverMessage>(client.Outcome);
            Assert.True(over.Outcome is GameOutcomeView.TeamWon or GameOutcomeView.Draw);
            Assert.NotEmpty(client.Log);
        }
    }
}
