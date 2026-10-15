using Cantrip.Server;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A seat taken by a ticket, as a player clicking "Open in Cantrip" in Discord takes theirs: the link names the table
/// in its query and carries the ticket after its hash, where no request and no log ever holds it (the Discord bot's
/// ClientSite). A table that seats only ticket holders refuses anyone else, so a table reached is a ticket that worked.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class TicketTests(WebClient web)
{
    [Fact]
    public async Task A_ticket_after_the_hash_seats_its_player_and_leaves_the_address_bar()
    {
        string decks = Path.Combine(WebClient.Root, "decks");
        DeckList islands = DeckList.Load(DeckList.Find(decks, "demo/2-white-blue"));
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0,
            Players = 2,
            ContentRoot = WebClient.Content,
            RequireTickets = true,
            Decks = [islands],
        });

        string mine = server.Reserve(1, "You", islands.Name);
        string theirs = server.Reserve(2, "Them", islands.Name);
        var address = new Uri($"ws://localhost:{server.Port}/game");

        // The other seat claims its own ticket, so the table fills and starts.
        var opponent = new GameClient(await WebSocketTransport.ConnectAsync(address));
        opponent.Join("Them", 2, theirs);

        await using TablePage table = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(address.AbsoluteUri)}#ticket={mine}", WebClient.Pictures(nameof(TicketTests)));

        // Kept pumping, so the opponent's seat answers nothing but stays connected while the page sits down.
        using var pumping = new CancellationTokenSource();
        Task pump = Task.Run(async () =>
        {
            while (!pumping.IsCancellationRequested)
            {
                opponent.Pump();
                await Task.Delay(50);
            }
        });

        await table.SettledAsync();
        await table.PictureAsync("seated");
        string shown = await table.Page.EvaluateAsync<string>("() => location.href");

        await pumping.CancelAsync();
        await pump;
        opponent.Disconnect();

        Assert.DoesNotContain(mine, shown, StringComparison.OrdinalIgnoreCase);
        Assert.Empty(table.Errors);
    }
}
