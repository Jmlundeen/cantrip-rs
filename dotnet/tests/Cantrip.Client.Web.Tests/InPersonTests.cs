using Cantrip.Art;
using Cantrip.Protocol;
using Cantrip.Protocol.InPerson;
using Cantrip.Server.Host;
using Cantrip.Server.Rules;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// In-person play's tablets (docs/in-person.md): the public board, shared between the players with seat 2's half turned
/// to face them, or one for each player; tapped with a finger, and sending the relay what was tapped and nothing else.
/// <para>
/// Fed here by the test rather than a relay: a real table plays the Bolt-the-Goyf scenario to Lightning Bolt on the stack,
/// and what it sent a seat - a state, events - is handed to the tablet as the public feed would carry it.
/// </para>
/// </summary>
[Collection(WebClientCollection.Name)]
[Trait("Category", "WallClock")]
public sealed class InPersonTests(WebClient web)
{
    private const string Scenario = "in-person-lightning-bolt-aimed-from-the-tablets-kills-tarmogoyf";

    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(90);

    [Fact]
    public async Task A_tablet_draws_the_public_board_and_sends_only_what_is_tapped()
    {
        IReadOnlyList<ServerMessage> feed = await BoltOnTheStackAsync();
        await using Microsoft.AspNetCore.Builder.WebApplication art = await ArtServer.StartAsync(
            new ArtStore(new ArtStoreOptions { ContentRoot = WebClient.Content, CacheDirectory = WebClient.ArtCache }), "127.0.0.1", 0);
        string pictures = WebClient.Pictures(nameof(InPersonTests));
        string query = $"art={Uri.EscapeDataString(art.Urls.First())}";

        // The shared board: before the game, where the draft stands; then the game, seat 2's half turned at the top.
        await using (TabletDriver shared = await TabletDriver.OpenAsync(web, query, pictures, "shared"))
        {
            await shared.FeedAsync(new DeviceWelcomeMessage { Role = DeviceRole.Tablet, Side = 0, PlayerNames = ["Bot1", "Bot2"] });
            await shared.FeedAsync(new DevicePhaseMessage
            {
                Phase = TablePhase.Drafting,
                Line = "Grid 4 of 18 · Bot2 to pick",
                Connected = ["Bot1's phone", "Bot2's phone", "the table"],
            });
            await shared.SettledAsync(state => state.Screen == "phase" && state.PhaseLine.Contains("Grid 4", StringComparison.Ordinal));
            await shared.PictureAsync("phase");

            TabletState board = await FeedGameAsync(shared, feed);
            await shared.PictureAsync("board");
            Assert.Equal("shared", board.Layout);

            TabletCard goyf = board.Cards.Single(card => card.Name == "Tarmogoyf");
            TabletCard mountain = board.Cards.First(card => card.Name == "Mountain");
            Assert.Equal(("opponent-battlefield", 2), (goyf.Place, goyf.Seat));
            Assert.True(goyf.Y < TabletDriver.Height / 2f, $"Bot2's Tarmogoyf is on the turned half at the top, not at {goyf.Y}.");
            Assert.Equal(("battlefield", 1), (mountain.Place, mountain.Seat));
            Assert.True(mountain.Y > TabletDriver.Height / 2f, $"Bot1's Mountain is on the bottom half, not at {mountain.Y}.");
            Assert.Contains(board.Cards, card => card.Name == "Lightning Bolt" && card.Place == "stack");
            Assert.Equal([1, 2], board.Players.Select(player => player.Seat).Order());
            Assert.True(board.Players.Single(player => player.Seat == 2).Turned);

            // A tap on the turned half picks what is under the finger there, and the tablet sends it and says nothing.
            await shared.TapCardAsync(goyf);
            TabletState tapped = await shared.SettledAsync(state => state.Taps.Count == 1);
            Assert.Equal(new TabletTap(goyf.Handle, 0, tapped.Taps[0].Sequence), tapped.Taps[0]);

            await shared.TapCardAsync(mountain);
            tapped = await shared.SettledAsync(state => state.Taps.Count == 2);
            Assert.Equal(mountain.Handle, tapped.Taps[1].Object);

            await shared.TapPlayerAsync(2);
            tapped = await shared.SettledAsync(state => state.Taps.Count == 3);
            Assert.Equal((0, 2), (tapped.Taps[2].Object, tapped.Taps[2].Player));

            // The shared board shows the stack once, over the line between the halves: Bot1's Bolt upright, facing Bot1, and
            // set off toward Bot1's half at the bottom.
            TabletCard bolt = tapped.Cards.Single(card => card.Name == "Lightning Bolt");
            Assert.Equal("stack", bolt.Place);
            Assert.True(tapped.StackShown);
            Assert.True(bolt.Y > TabletDriver.Height * 0.3f && bolt.Y < TabletDriver.Height * 0.7f, $"The stack is over the middle of the board, not at {bolt.Y}.");
            await shared.TapCardAsync(bolt);
            tapped = await shared.SettledAsync(state => state.Taps.Count == 4);
            Assert.Equal(bolt.Handle, tapped.Taps[3].Object);

            // One turn strip, between the halves, read by the player whose turn it is: turned round only for seat 2.
            Assert.True(tapped.TurnStrip is { Step.Length: > 0 }, "The tablet shows the turn.");
            Assert.Equal(tapped.TurnStrip!.Faces == 2, tapped.TurnStrip.Turned);

            // One prompt, between the rails down the board's left edge, the asked player's as their phone said it, and pressed
            // from there: Bot1 asked, upright, their Pass sent for Bot1's phone to press. Then Bot2 asked, searching their
            // library: the same panel turned to face them, showing only that a choice is being made on their phone.
            await shared.FeedAsync(new TablePromptMessage
            {
                Seat = 1,
                Prompt = new PromptCard
                {
                    Version = 4,
                    Asking = true,
                    Title = "Your first main phase",
                    Detail = "Cast a spell",
                    Rows =
                    [
                        new PromptRow { Items = [new PromptItem { Id = 0, Kind = PromptItemKind.Toggle, Text = "Auto Pass" }] },
                        new PromptRow { Items = [new PromptItem { Id = 1, Kind = PromptItemKind.Button, Text = "Pass", Primary = true }] },
                    ],
                },
            });
            await shared.FeedAsync(new TablePromptMessage { Seat = 2, Prompt = new PromptCard { Version = 2, Title = "Waiting for Bot1" } });
            TabletState prompted = await shared.SettledAsync(state => state.Prompt is { Seat: 1, Prompted: true, Asking: true } && state.Rails?.Count == 2);
            await shared.PictureAsync("prompt");
            TabletPromptState bot1 = prompted.Prompt!;
            Assert.Equal(("Your first main phase", false, false), (bot1.Title, bot1.Private, bot1.Turned));
            TabletRail rail1 = prompted.Rails!.Single(rail => rail.Seat == 1);
            TabletRail rail2 = prompted.Rails!.Single(rail => rail.Seat == 2);
            Assert.True(rail1.X < TabletDriver.Width / 4f && rail2.X < TabletDriver.Width / 4f, "Both rails are down the left edge.");
            Assert.True(rail2.Y < bot1.Box!.Y && bot1.Box.Y < rail1.Y, "The prompt is between the rails, seat 2's above it and seat 1's below.");
            TabletPromptItem pass = bot1.Items.Single(item => item.Text == "Pass");
            await shared.TapAsync(pass.X, pass.Y);
            await shared.SettledAsync(state => state.Presses?.SequenceEqual([new TabletPress(1, 4, 1)]) == true);

            await shared.FeedAsync(new TablePromptMessage { Seat = 1, Prompt = new PromptCard { Version = 5, Title = "Waiting for Bot2" } });
            await shared.FeedAsync(new TablePromptMessage { Seat = 2, Prompt = new PromptCard { Version = 3, Private = true, Asking = true } });
            TabletState turned = await shared.SettledAsync(state => state.Prompt is { Seat: 2, Asking: true });
            await shared.PictureAsync("prompt-turned");
            Assert.True(turned.Prompt!.Private && turned.Prompt.Turned, "Bot2's private choice is shown turned to face them.");
            Assert.Empty(turned.Prompt.Items);

            // The stack folds to its caps, and opens again.
            await shared.TapAsync(tapped.StackFold!.X, tapped.StackFold.Y);
            TabletState folded = await shared.SettledAsync(state => state.StackCollapsed && !state.Cards.Any(card => card.Name == "Lightning Bolt"));
            await shared.PictureAsync("stack-folded");
            await shared.TapAsync(folded.StackFold!.X, folded.StackFold.Y);
            tapped = await shared.SettledAsync(state => !state.StackCollapsed && state.Cards.Any(card => card.Name == "Lightning Bolt"));

            // A long press holds the card up to read, turned to face the player across from it, and taps nothing.
            await shared.PressAsync(goyf);
            await shared.SettledAsync(state => state.Held > 0);
            await shared.PictureAsync("held-up");
            await shared.LiftAsync();
            TabletState after = await shared.SettledAsync(state => state.Held == 0);
            Assert.Equal(4, after.Taps.Count);
            Assert.Empty(shared.Errors);
        }

        // A player's own board: that seat's half, the stack, and both life totals.
        foreach (int side in (int[])[1, 2])
        {
            await using TabletDriver own = await TabletDriver.OpenAsync(web, $"{query}&side={side}", pictures, $"side-{side}");
            await own.FeedAsync(new DeviceWelcomeMessage { Role = DeviceRole.Tablet, Side = side, PlayerNames = ["Bot1", "Bot2"] });
            TabletState board = await FeedGameAsync(own, feed);
            await own.PictureAsync("board");

            Assert.Equal($"side {side}", board.Layout);
            Assert.All(board.Cards.Where(card => card.Place != "stack"), card => Assert.Equal((side, "battlefield"), (card.Seat, card.Place)));
            Assert.Contains(board.Cards, card => card.Name == "Lightning Bolt" && card.Place == "stack");
            Assert.Equal([side], board.Players.Select(player => player.Seat));
            Assert.Contains("Bot1  20", board.Lives, StringComparison.Ordinal);
            Assert.Contains("Bot2  20", board.Lives, StringComparison.Ordinal);

            // The stack lies over the board; folded, it leaves the creatures under it to be tapped.
            await own.TapAsync(board.StackFold!.X, board.StackFold.Y);
            board = await own.SettledAsync(state => state.StackCollapsed);
            TabletCard mine = board.Cards.First(card => card.Name == (side == 1 ? "Mountain" : "Tarmogoyf"));
            await own.TapCardAsync(mine);
            TabletState tapped = await own.SettledAsync(state => state.Taps.Count == 1);
            Assert.Equal(mine.Handle, tapped.Taps[0].Object);
            Assert.Empty(own.Errors);
        }
    }

    /// <summary>
    /// The whole table, end to end: Bot1's phone and one shared tablet at a relay, beside a table playing the Bolt-the-Goyf
    /// scenario. Bolt is tapped in the phone's hand, the Mountain and the Goyf with a finger on the tablet - the Goyf on
    /// seat 2's turned half - and each tap goes through the relay to the phone, which answers the table; the table judges
    /// every answer against the script. No tablet ever names a card in the phone's hand.
    /// </summary>
    [Fact]
    public async Task Bolt_is_aimed_from_a_phone_and_a_shared_tablet_through_the_relay()
    {
        await using var table = new InPersonTable(web, "shared", 0);
        string? failure = await ScenarioPlayer.PlayAsync(web, ScenarioPath(Scenario), atTable: table);
        Assert.True(failure is null, $"{Scenario}:\n{failure}");
        await table.Tablets[0].PictureAsync("over");
        await FlewInFromTheirSideAsync(table.Tablets[0]);
    }

    /// <summary>
    /// Bot1's Mountain, played from their phone, flew onto the tablet from Bot1's side of it - the bottom edge, where they
    /// sit - rather than from their rail.
    /// </summary>
    private static async Task FlewInFromTheirSideAsync(TabletDriver tablet)
    {
        TabletState state = await tablet.SettledAsync();
        TabletCard mountain = state.Cards.First(card => card.Name == "Mountain" && card.Seat == 1);
        Assert.True(mountain.FlewFrom is { } from && from.Y > TabletDriver.Height, $"Bot1's Mountain flew in from {mountain.FlewFrom}, not from below the tablet's bottom edge.");
    }

    /// <summary>
    /// The same with a tablet for each player: the Mountain tapped on Bot1's own board, the Goyf on Bot2's.
    /// </summary>
    [Fact]
    public async Task Bolt_is_aimed_from_a_phone_and_each_players_own_tablet_through_the_relay()
    {
        await using var table = new InPersonTable(web, "own-boards", 1, 2);
        string? failure = await ScenarioPlayer.PlayAsync(web, ScenarioPath(Scenario), atTable: table);
        Assert.True(failure is null, $"{Scenario}:\n{failure}");
        await table.Tablets[0].PictureAsync("over");
        await table.Tablets[1].PictureAsync("over");
        await FlewInFromTheirSideAsync(table.Tablets[0]);
    }

    /// <summary>
    /// A target in a graveyard, tapped on the shared tablet: Reanimate cast from the phone at the Goyf just Bolted, Bot2's
    /// graveyard pile tapped on the turned half to open its list, and the Goyf tapped in it.
    /// </summary>
    [Fact]
    public async Task A_graveyard_target_is_tapped_in_the_list_its_pile_opens_on_the_tablet()
    {
        const string reanimate = "in-person-reanimate-aimed-from-a-tablets-graveyard-pile-takes-the-bolted-tarmogoyf";
        await using var table = new InPersonTable(web, "graveyard", 0);
        string? failure = await ScenarioPlayer.PlayAsync(web, ScenarioPath(reanimate), atTable: table);
        Assert.True(failure is null, $"{reanimate}:\n{failure}");
        await table.Tablets[0].PictureAsync("over");
    }

    private static string ScenarioPath(string name) => Path.Combine(WebClient.Root, "scenarios", name + ".cantrip");

    /// <summary>Hands a tablet the game as the public feed would, and waits for Lightning Bolt on its stack.</summary>
    private static async Task<TabletState> FeedGameAsync(TabletDriver tablet, IReadOnlyList<ServerMessage> feed)
    {
        foreach (ServerMessage message in feed)
        {
            await tablet.FeedAsync(new PublicFeedMessage { Message = message });
        }

        return await tablet.SettledAsync(state => state.Screen == "board" && state.Cards.Any(card => card.Name == "Lightning Bolt" && card.Place == "stack"));
    }

    /// <summary>
    /// Plays the scenario at a real table, seat 2 answered as the script means, to Lightning Bolt on the stack aimed at
    /// Tarmogoyf; and returns what seat 2 was sent, as the public feed carries it: the welcome for seat zero, with no
    /// decklist, then the states and events. Seat 2's hand is in those views, but a tablet draws no hand.
    /// </summary>
    private static async Task<IReadOnlyList<ServerMessage>> BoltOnTheStackAsync()
    {
        ScenarioSpec scenario = ScenarioSpec.Load(Path.Combine(WebClient.Root, "scenarios", Scenario + ".cantrip"), [2]);
        await using ScenarioTable table = await ScenarioTable.StartAsync(scenario, WebClient.Content);
        var recording = new RecordingTransport(await WebSocketTransport.ConnectAsync(table.Address));
        var bot2 = new GameClient(recording);
        bot2.Join("Bot2", 2);

        try
        {
            while (true)
            {
                ScenarioProgress progress = await table.NextAsync(Patience);
                Assert.True(progress.Verdict is null, "The scenario ended before Lightning Bolt was on the stack.");
                if (progress.Expected is not { Seat: 2 } asked)
                {
                    continue;
                }

                DateTime giveUp = DateTime.UtcNow + Patience;
                while (bot2.Prompt?.RequestId != asked.RequestId)
                {
                    Assert.True(DateTime.UtcNow < giveUp, $"Bot2 was never sent request {asked.RequestId}.");
                    if (bot2.Pump() == 0)
                    {
                        await Task.Delay(5);
                    }
                }

                if (bot2.Zone(ZoneKindView.Stack, 0).Any(card => card.Name == "Lightning Bolt"))
                {
                    break;
                }

                bot2.Answer(asked.Answer ?? throw new Xunit.Sdk.XunitException($"Bot2 is not meant to undo ({asked.Line})."));
            }
        }
        finally
        {
            bot2.Disconnect();
        }

        return [.. recording.Received.Select(message => message is WelcomeMessage welcome
                ? new WelcomeMessage { Seat = 0, PlayerNames = welcome.PlayerNames }
                : message)
            .Where(message => message is WelcomeMessage or StateMessage or EventsMessage or GameOverMessage)];
    }

    /// <summary>A transport that keeps everything it receives.</summary>
    private sealed class RecordingTransport(IGameTransport inner) : IGameTransport
    {
        private readonly List<ServerMessage> _received = [];

        public IReadOnlyList<ServerMessage> Received => _received;

        public bool IsConnected => inner.IsConnected;

        public void Send(ClientMessage message) => inner.Send(message);

        public IReadOnlyList<ServerMessage> Receive()
        {
            IReadOnlyList<ServerMessage> received = inner.Receive();
            _received.AddRange(received);
            return received;
        }

        public void Dispose() => inner.Dispose();
    }
}
