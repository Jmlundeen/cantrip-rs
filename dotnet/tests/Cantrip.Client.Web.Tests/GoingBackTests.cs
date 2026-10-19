using Cantrip.Server;
using Cantrip.Server.Host;
using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A game over at a table that names the page its players came from - the Discord bot's constructed play, or its draft
/// queues - takes the player back there: the result is shown with a button that goes at once, and the page goes by itself
/// after a while (Cantrip.Client.Presentation's GoingBack, which its own tests hold to the wait).
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class GoingBackTests(WebClient web)
{
    [Fact]
    public async Task The_result_of_a_constructed_game_has_a_button_back_to_the_constructed_page()
    {
        // The site's page, answered here: what matters is that the browser goes there.
        const string Constructed = "http://cantrip.test/constructed";
        string decks = Path.Combine(WebClient.Root, "decks");
        DeckList islands = DeckList.Load(DeckList.Find(decks, "demo/2-white-blue"));
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0,
            Players = 2,
            ContentRoot = WebClient.Content,
            RequireTickets = true,
            Decks = [islands],
            Return = new ReturnPage(new Uri(Constructed), "Constructed Play"),
        });

        string mine = server.Reserve(1, "You", islands.Name);
        string theirs = server.Reserve(2, "Them", islands.Name);
        var address = new Uri($"ws://localhost:{server.Port}/game");

        var opponent = new GameClient(await WebSocketTransport.ConnectAsync(address));
        opponent.Join("Them", 2, theirs);

        await using TablePage table = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(address.AbsoluteUri)}#ticket={mine}", WebClient.Pictures(nameof(GoingBackTests)));
        await table.Page.RouteAsync(Constructed, route => route.FulfillAsync(new RouteFulfillOptions
        {
            ContentType = "text/html",
            Body = "<!doctype html><title>Constructed Play</title><h1>Constructed Play</h1>",
        }));

        using var pumping = new CancellationTokenSource();
        Task pump = Task.Run(async () =>
        {
            while (!pumping.IsCancellationRequested)
            {
                opponent.Pump();
                await Task.Delay(50);
            }
        });

        try
        {
            await table.SettledAsync(state => state.Prompt.Length > 0);

            // The opponent concedes: the game, and with it the match, is over.
            opponent.Concede();
            await table.OverAsync(TimeSpan.FromSeconds(60));
            TableState over = await table.SettledAsync(state => state.Controls.Any(control => control.Label == "Back to Constructed Play"));
            await table.PictureAsync("over");
            Assert.Contains(over.Controls, control => control.Label == "Look at the board");

            await table.ClickAsync("Back to Constructed Play");
            await table.Page.WaitForURLAsync(Constructed, new PageWaitForURLOptions { Timeout = 30_000 });
            Assert.Equal("Constructed Play", await table.Page.TitleAsync());
        }
        finally
        {
            await pumping.CancelAsync();
            await pump;
            opponent.Disconnect();
        }

        Assert.Empty(table.Errors);
    }
}
