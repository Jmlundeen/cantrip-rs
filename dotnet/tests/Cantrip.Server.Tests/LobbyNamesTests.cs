using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// Whose name a seat carries. A ticketed seat (a Discord player's) is named by the table when it is reserved, and the
/// lobby is told so, to show the name rather than offer to change it; a seat taken at the table is named by its player.
/// </summary>
public sealed class LobbyNamesTests
{
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(10);

    [Fact]
    public async Task A_ticketed_seat_is_fixed_and_an_open_seat_is_not()
    {
        DeckList deck = DeckList.LoadDirectory(RepoRoot.DemoDecks).First();
        await using GameServer ticketed = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, RequireTickets = true, Decks = [deck],
        });
        await using GameServer open = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, Decks = [deck],
        });

        string ticket = ticketed.Reserve(1, "Alice#1234", deck.Name);
        TablePlayerView reserved = await SeatedAsync(ticketed.Port, "Alice#1234", ticket);
        TablePlayerView free = await SeatedAsync(open.Port, "Bob", null);

        Assert.True(reserved.Fixed);
        Assert.Equal("Alice#1234", reserved.Name);
        Assert.False(free.Fixed);
    }

    private static async Task<TablePlayerView> SeatedAsync(int port, string name, string? ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(new Uri($"ws://127.0.0.1:{port}/game")));
        client.Join(name, 1, ticket);

        DateTime deadline = DateTime.UtcNow + Patience;
        while (DateTime.UtcNow < deadline)
        {
            client.Pump();
            if (client.Table?.Players.FirstOrDefault(player => player.Seat == 1) is { } seated)
            {
                return seated;
            }

            await Task.Delay(20);
        }

        throw new TimeoutException("The table never listed the seat.");
    }
}
