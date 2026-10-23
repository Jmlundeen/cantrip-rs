using Cantrip.Art;
using Cantrip.Server;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// One whole game played in the web client: a real server, a bot in the other seat over its own WebSocket, and this
/// seat played by the page itself (<c>?auto</c>), start to finish. What no unit test can say: that the client as a
/// browser runs it - the WebAssembly build, the socket, every redraw a real game asks for - holds up for a game.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class WholeGameTests(WebClient web)
{
    [Fact]
    public async Task A_whole_game_is_played_to_its_end_in_the_browser()
    {
        string decks = Path.Combine(WebClient.Root, "decks");
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Seed = 1,
            ContentRoot = WebClient.Content,
            AutoStart = true,
            Decks = [DeckList.Load(DeckList.Find(decks, "demo/1-black-red")), DeckList.Load(DeckList.Find(decks, "demo/2-white-blue"))],
        });

        // Art from a server of its own, named to the page, as a real host serves it: left to guess, the page would ask
        // the port after the game's, which here is whatever else this machine has listening there.
        await using Microsoft.AspNetCore.Builder.WebApplication art = await ArtServer.StartAsync(
            new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }), "127.0.0.1", 0);

        var address = new Uri($"ws://localhost:{server.Port}/game");
        using var stopping = new CancellationTokenSource();
        Task opponent = PlaySeatAsync(address, seat: 2, stopping.Token);

        await using TablePage table = await TablePage.OpenAsync(
            web,
            $"server={Uri.EscapeDataString(address.AbsoluteUri)}&art={Uri.EscapeDataString(art.Urls.First())}&seat=1&name=You&auto",
            WebClient.Pictures(nameof(WholeGameTests)));

        await table.SettledAsync();
        await table.PictureAsync("dealt");

        TableState over = await table.OverAsync(TimeSpan.FromMinutes(4));
        await table.PictureAsync("over");

        await stopping.CancelAsync();
        await opponent;

        Assert.NotEmpty(over.Outcome);
        Assert.Empty(table.Errors);
    }

    /// <summary>
    /// The opponent is played by the page too (<c>?bots=2</c>): the tactical bot, over a second socket the page opens for
    /// it, with nothing else at the table. That it runs where a player's browser runs is the whole reason it is written
    /// the way it is, and only a browser can say that it does. Asked to (<c>?narrate</c>), it tells its decisions in the
    /// browser's console, which is where a person who lost to it reads why.
    /// </summary>
    [Fact]
    public async Task A_whole_game_is_played_against_a_bot_the_page_plays_itself()
    {
        string decks = Path.Combine(WebClient.Root, "decks");
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Seed = 1,
            ContentRoot = WebClient.Content,
            AutoStart = true,

            // One deck is a mirror: the seats are dealt round the list.
            Decks = [DeckList.Load(DeckList.Find(decks, "demo/3-green-red"))],
        });

        await using Microsoft.AspNetCore.Builder.WebApplication art = await ArtServer.StartAsync(
            new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }), "127.0.0.1", 0);

        var address = new Uri($"ws://localhost:{server.Port}/game");

        // Nobody else joins: seat 1 plays itself the one deterministic way, and seat 2 is the page's to play.
        await using TablePage table = await TablePage.OpenAsync(
            web,
            $"server={Uri.EscapeDataString(address.AbsoluteUri)}&art={Uri.EscapeDataString(art.Urls.First())}&seat=1&name=You&auto&bots=2&narrate",
            WebClient.Pictures(nameof(WholeGameTests)));

        List<string> told = [];
        table.Page.Console += (_, message) =>
        {
            lock (told)
            {
                told.Add(message.Text);
            }
        };

        await table.SettledAsync();
        await table.PictureAsync("bot-dealt");

        TableState over = await table.OverAsync(TimeSpan.FromMinutes(4));
        await table.PictureAsync("bot-over");

        Assert.NotEmpty(over.Outcome);
        Assert.Empty(table.Errors);
        lock (told)
        {
            Assert.Contains(told, line => line.StartsWith("[Bot] T", StringComparison.Ordinal) && line.Contains(" | ", StringComparison.Ordinal));
        }
    }

    /// <summary>A seat played the one deterministic way over its own socket, as dotnet\tools\Cantrip.Bot plays one.</summary>
    private static async Task PlaySeatAsync(Uri address, int seat, CancellationToken stopping)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(address, stopping));
        client.Join($"Bot {seat}", seat);
        var bot = new AutoSeat(client);

        try
        {
            while (!client.IsOver && !stopping.IsCancellationRequested)
            {
                if (!bot.Step())
                {
                    await Task.Delay(15, stopping);
                }
            }
        }
        catch (OperationCanceledException)
        {
        }
        finally
        {
            client.Disconnect();
        }
    }
}
