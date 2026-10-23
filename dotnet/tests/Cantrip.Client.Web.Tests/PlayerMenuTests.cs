using Cantrip.Server;
using Cantrip.Server.Host;
using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A right click on the other player's box at a table offers to message them on Discord, for a player who came from there:
/// the table is told their account (TablePlayerView.Discord), and the menu opens their Discord page in a new tab.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class PlayerMenuTests(WebClient web)
{
    [Fact]
    public async Task A_right_click_on_the_opponent_offers_to_message_them_on_Discord_in_a_new_tab()
    {
        DeckList islands = DeckList.Load(DeckList.Find(Path.Combine(WebClient.Root, "decks"), "demo/2-white-blue"));
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Format = Cantrip.Decks.DeckFormat.Limited(),
            Port = 0,
            Players = 2,
            ContentRoot = WebClient.Content,
            RequireTickets = true,
            Decks = [islands],
        });

        string mine = server.Reserve(1, "You", islands.Name, discord: "1001");
        string theirs = server.Reserve(2, "Them", islands.Name, discord: "1002");
        var address = new Uri($"ws://localhost:{server.Port}/game");

        var opponent = new GameClient(await WebSocketTransport.ConnectAsync(address));
        opponent.Join("Them", 2, theirs);

        await using TablePage table = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(address.AbsoluteUri)}#ticket={mine}", WebClient.Pictures(nameof(PlayerMenuTests)));
        // Discord's page, answered here: what matters is that a tab goes there.
        await table.Page.Context.RouteAsync("https://discord.com/users/**", route => route.FulfillAsync(new RouteFulfillOptions
        {
            ContentType = "text/html",
            Body = "<!doctype html><title>Discord</title>",
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
            TableState state = await table.SettledAsync(state => state.Prompt.Length > 0 && state.Players?.Count == 2);

            // Their own box offers nothing: no menu opens.
            PlayerState me = state.Players!.Single(player => player.Seat == state.Seat);
            await table.Page.Mouse.ClickAsync(me.X, me.Y, new MouseClickOptions { Button = MouseButton.Right });
            Assert.DoesNotContain((await table.SettledAsync()).Controls, control => control.Label == "DM on Discord");

            PlayerState them = state.Players!.Single(player => player.Seat != state.Seat);
            await table.Page.Mouse.ClickAsync(them.X, them.Y, new MouseClickOptions { Button = MouseButton.Right });
            await table.SettledAsync(state => state.Controls.Any(control => control.Kind == "item" && control.Label == "DM on Discord"));
            await table.PictureAsync("player-menu");

            IPage discord = await table.Page.Context.RunAndWaitForPageAsync(() => table.ClickAsync("DM on Discord", "item"));
            await discord.WaitForURLAsync("https://discord.com/users/1002");
            Assert.Equal("Discord", await discord.TitleAsync());

            // The table stays where it was, and the menu is gone.
            Assert.DoesNotContain((await table.SettledAsync()).Controls, control => control.Label == "DM on Discord");
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
