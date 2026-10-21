using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// The game menu (MainPage.GameMenu): the button of three lines beside the side panel's arrow opens Concede Game and Offer
/// Draw. When the other player offers a draw (docs/rules-exceptions.md, RX-4) the dock asks to accept or decline, and the
/// menu's entry reads Accept Draw while the offer is open; accepting draws the game.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class GameMenuTests(WebClient web)
{
    [Fact]
    public async Task The_game_menu_offers_to_concede_or_draw_and_a_draw_offered_is_accepted_from_the_dock()
    {
        DeckList islands = DeckList.Load(DeckList.Find(Path.Combine(WebClient.Root, "decks"), "demo/2-white-blue"));
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

        var opponent = new GameClient(await WebSocketTransport.ConnectAsync(address));
        opponent.Join("Them", 2, theirs);

        await using TablePage table = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(address.AbsoluteUri)}#ticket={mine}", WebClient.Pictures(nameof(GameMenuTests)));

        using var pumping = new CancellationTokenSource();
        Task pump = Task.CompletedTask;
        try
        {
            TableState state = await table.SettledAsync(state => state.Prompt.Length > 0);

            // Before any offer: the menu has both entries.
            await table.ClickAsync("Game menu", "button");
            state = await table.SettledAsync(state => state.Controls.Any(control => control.Kind == "item" && control.Label == "Offer Draw"));
            Assert.Contains(state.Controls, control => control.Kind == "item" && control.Label == "Concede Game");
            await table.PictureAsync("menu");
            await table.ClickAsync("Game log", "tab");
            await table.SettledAsync(state => !state.Controls.Any(control => control.Kind == "item"));

            // The game is played to the opponent's first priority - the page answering whatever it is asked with its dock's
            // first button, the opponent with each question's default - where the opponent offers a draw.
            int answered = 0;
            DateTime giveUp = DateTime.UtcNow + TimeSpan.FromSeconds(90);
            while (true)
            {
                Assert.True(DateTime.UtcNow < giveUp, "The opponent never had priority.");
                opponent.Pump();
                if (opponent.Prompt is { } asked)
                {
                    if (opponent.CanOfferDraw)
                    {
                        opponent.OfferDraw();
                        break;
                    }

                    opponent.Answer(asked.Prompt.Default());
                }

                state = (await table.StateAsync())!;
                if (state is { Ready: true, Busy: false, RequestId: > 0 } && state.RequestId != answered
                    && state.Dock?.FirstOrDefault(button => button.Enabled) is { } first)
                {
                    answered = state.RequestId;
                    await table.ClickAsync(first.Label, "button");
                }

                await Task.Delay(100);
            }

            // From here on the opponent only passes, which leaves its offer standing.
            pump = Task.Run(async () =>
            {
                while (!pumping.IsCancellationRequested)
                {
                    opponent.Pump();
                    if (opponent.Prompt is { } asked)
                    {
                        opponent.Answer(asked.Prompt.Default());
                    }

                    await Task.Delay(50);
                }
            });

            state = await table.SettledAsync(state => state.Dock?.Any(button => button.Label == "Accept Draw") == true);
            Assert.Contains(state.Dock!, button => button.Label == "Decline");
            Assert.Equal("Them offers a draw", state.Title);
            await table.PictureAsync("offered");

            await table.ClickAsync("Game menu", "button");
            state = await table.SettledAsync(state => state.Controls.Any(control => control.Kind == "item" && control.Label == "Accept Draw"));
            Assert.DoesNotContain(state.Controls, control => control.Kind == "item" && control.Label == "Offer Draw");
            Assert.Contains(state.Controls, control => control.Kind == "item" && control.Label == "Concede Game");
            await table.PictureAsync("menu-accept");
            await table.ClickAsync("Game log", "tab");
            await table.SettledAsync(state => !state.Controls.Any(control => control.Kind == "item"));

            await table.ClickAsync("Accept Draw", "button");
            TableState over = await table.OverAsync(TimeSpan.FromSeconds(30));
            Assert.Contains("draw", over.Outcome, StringComparison.OrdinalIgnoreCase);
            await table.PictureAsync("drawn");
            TableResult result = await server.Finished.WaitAsync(TimeSpan.FromSeconds(30)).ContinueWith(_ => server.Result!);
            Assert.Empty(result.WinningSeats);
            Assert.Equal(GameOutcomeView.Draw, opponent.Outcome?.Outcome ?? await OutcomeAsync(opponent));
        }
        finally
        {
            await pumping.CancelAsync();
            await pump;
            opponent.Disconnect();
        }

        Assert.Empty(table.Errors);
    }

    private static async Task<GameOutcomeView?> OutcomeAsync(GameClient client)
    {
        for (int wait = 0; wait < 100 && client.Outcome is null; wait++)
        {
            await Task.Delay(50);
        }

        return client.Outcome?.Outcome;
    }
}
