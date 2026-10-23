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
            Format = Cantrip.Decks.DeckFormat.Limited(),
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

    [Fact]
    public async Task A_draft_called_off_because_the_other_player_dropped_says_so_with_a_button_back_to_the_site()
    {
        const string Queues = "http://cantrip.test/draft";
        string data = Path.Combine(Path.GetTempPath(), "cantrip-going-back-" + Guid.NewGuid().ToString("N"));
        try
        {
            var store = new Cantrip.Server.Events.EventStore(data);
            Cantrip.Server.Events.CreatedEvent created = Cantrip.Server.Events.DraftPod.CreateWinston(store, ContentCache.Get(WebClient.Content), new Cantrip.Server.Events.WinstonDraftSetup
            {
                Name = "Winston draft: RW Aggro",
                Cube = "rw-aggro",
                CubeCards = Cantrip.Server.Events.CubeList.Load(Path.Combine(WebClient.Root, "decks", "rw-aggro.txt")),
                Cards = 18,
                Players = [("alice", "Alice"), ("bob", "Bob")],
            });
            Cantrip.Server.Events.DraftPod pod = Cantrip.Server.Events.DraftPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content));
            await using GameServer server = await GameServer.StartAsync(new GameServerOptions
            {
                Format = Cantrip.Decks.DeckFormat.Limited(),
                Port = 0,
                Players = 2,
                ContentRoot = WebClient.Content,
                Event = pod,
                Return = new ReturnPage(new Uri(Queues), "Draft Queues"),
            });
            var address = new Uri($"ws://localhost:{server.Port}/game");

            await using TablePage table = await TablePage.OpenAsync(
                web, $"server={Uri.EscapeDataString(address.AbsoluteUri)}#ticket={created.Tickets["alice"]}", WebClient.Pictures(nameof(GoingBackTests)));
            await table.Page.RouteAsync(Queues, route => route.FulfillAsync(new RouteFulfillOptions
            {
                ContentType = "text/html",
                Body = "<!doctype html><title>Draft Queues</title><h1>Draft Queues</h1>",
            }));

            // Alice waits for Bob to sit down; Bob drops from the site instead.
            for (int wait = 0; wait < 300 && store.LoadEvent(created.Record.Id)!.Participant("alice")!.Seated is false; wait++) await Task.Delay(100);
            Assert.True(store.LoadEvent(created.Record.Id)!.Participant("alice")!.Seated);
            Assert.Null(pod.Drop("bob"));
            await table.SettledAsync(state => state.Controls.Any(control => control.Label == "Back to Draft Queues"));
            await table.PictureAsync("called-off");

            await table.ClickAsync("Back to Draft Queues");
            await table.Page.WaitForURLAsync(Queues, new PageWaitForURLOptions { Timeout = 30_000 });
            Assert.Empty(table.Errors);
        }
        finally
        {
            if (Directory.Exists(data)) Directory.Delete(data, recursive: true);
        }
    }
}
