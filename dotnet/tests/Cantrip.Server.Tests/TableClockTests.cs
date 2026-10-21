using Cantrip.Client;
using Cantrip.Core;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.Server.Rules;
using Cantrip.TestKit;
using Xunit;
using Xunit.Abstractions;

namespace Cantrip.Server.Tests;

/// <summary>
/// Every table has chess clocks (docs/events.md, "Clocks"), as Magic Online keeps them: each player's runs while the game
/// waits on them (GameServerOptions.PlayerClock, an hour), and whoever's runs out loses - a lobby's table, a scenario's, an
/// event's, a game in one process. Time is a fake the test moves: nothing here waits on the wall clock but for the table's
/// pump to look again, which it does at least every tenth of a second. On the wall clock because each is a real table over a
/// socket; those dealing through the native host skip, saying so, when it has not been built.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class TableClockTests(ITestOutputHelper output)
{
    private static readonly TimeSpan Patience = TimeSpan.FromSeconds(30);

    /// <summary>Each player's clock at a table that says nothing else: an hour.</summary>
    private static readonly TimeSpan TwoPlayerClock = MatchClocks.DefaultPlayerClock;

    private sealed class Clock : TimeProvider
    {
        private long _ticks = new DateTimeOffset(2026, 10, 21, 0, 0, 0, TimeSpan.Zero).Ticks;
        public override DateTimeOffset GetUtcNow() => new(Interlocked.Read(ref _ticks), TimeSpan.Zero);
        public void Advance(TimeSpan duration) => Interlocked.Add(ref _ticks, duration.Ticks);
    }

    private bool NoHost()
    {
        if (File.Exists(NativeHostProcess.Locate())) return false;
        output.WriteLine($"Skipped: no cantrip-native-host at {NativeHostProcess.Locate()}.");
        return true;
    }

    private static DeckList Deck() => DeckList.LoadDirectory(RepoRoot.DemoDecks).First();

    /// <summary>A lobby's table of <paramref name="players"/>, every seat taken and ready, dealt.</summary>
    private static async Task<(GameServer Server, List<GameClient> Clients)> LobbyAsync(
        Clock clock, int players = 2, TimeSpan? playerClock = null, IRulesHostFactory? rules = null)
    {
        DeckList deck = Deck();
        GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = players, ContentRoot = RepoRoot.ContentRoot, Decks = [deck], AutoStart = true, TimeProvider = clock,
            PlayerClock = playerClock ?? MatchClocks.DefaultPlayerClock, Rules = rules ?? RulesHostFactory.Native,
        });
        var address = new Uri($"ws://127.0.0.1:{server.Port}/game");
        List<GameClient> clients = [];
        for (int seat = 1; seat <= players; seat++)
        {
            var client = new GameClient(await WebSocketTransport.ConnectAsync(address));
            client.Join($"Player {seat}", seat);
            clients.Add(client);
        }

        await Until(() => clients.All(client => client.Clock is { } clocks && clocks.Clocks.Count == players), [.. clients]);
        return (server, clients);
    }

    private static async Task Until(Func<bool> condition, params GameClient[] clients)
    {
        DateTime deadline = DateTime.UtcNow + Patience;
        while (!condition())
        {
            if (DateTime.UtcNow > deadline)
            {
                throw new TimeoutException("The table never got there: " + string.Join(" | ", clients.Select(client =>
                    $"seat {client.Seat} asked {client.Prompt?.Prompt.GetType().Name ?? "nothing"}, clocks "
                    + string.Join(", ", client.Clock?.Clocks.Select(each => $"{each.Seat}:{each.LeftMs}{(each.Running ? "*" : string.Empty)}") ?? []))));
            }
            foreach (GameClient client in clients) client.Pump();
            await Task.Delay(5);
        }
    }

    private static async Task<TableResult> FinishedAsync(GameServer server)
    {
        await server.Finished.WaitAsync(Patience);
        return server.Result!;
    }

    private static void Leave(IEnumerable<GameClient> clients)
    {
        foreach (GameClient client in clients) client.Disconnect();
    }

    /// <summary>
    /// A lobby's table (the server's own, what play-game.cmd and the page's bots play at): each player starts with an hour,
    /// and a player the game waits on who never acts runs out of time and loses through the rules' concession (CR 104.3a),
    /// the other winning.
    /// </summary>
    [Fact]
    public async Task A_lobby_table_has_player_clocks_and_a_player_who_never_acts_loses_on_time()
    {
        if (NoHost()) return;
        var clock = new Clock();
        (GameServer server, List<GameClient> clients) = await LobbyAsync(clock);
        await using (server)
        {
            Assert.All(clients[0].Clock!.Clocks, each => Assert.Equal((long)TwoPlayerClock.TotalMilliseconds, each.LeftMs));
            int waiting = clients[0].Clock!.Clocks.First(each => each.Running).Seat;
            clock.Advance(TwoPlayerClock + TimeSpan.FromMinutes(1));
            TableResult result = await FinishedAsync(server);
            Assert.Equal(TablePhase.Finished, result.Phase);
            Assert.Equal([3 - waiting], result.WinningSeats);
            Assert.Contains("ran out of time", result.Detail, StringComparison.Ordinal);
            Leave(clients);
        }
    }

    /// <summary>
    /// A game won by its rules stays won, though its clocks are charged in the same look: the scripted rules end the game as
    /// a seat concedes, and the time jumps past the asked seat's hour at that very moment, so the table looks at both at
    /// once. The concession decides it, not the clock.
    /// </summary>
    [Fact]
    public async Task A_game_won_as_the_clocks_run_out_stays_won()
    {
        var clock = new Clock();
        var rules = new JumpingRules(clock, TimeSpan.FromHours(3));
        (GameServer server, List<GameClient> clients) = await LobbyAsync(clock, rules: rules);
        await using (server)
        {
            // Seat one is asked (ScriptedHost's opening), so its clock runs; seat two concedes.
            await Until(() => clients[0].Prompt is not null, [.. clients]);
            clients[1].Concede();
            TableResult result = await FinishedAsync(server);
            Assert.Equal([1], result.WinningSeats);
            Assert.DoesNotContain("ran out of time", result.Detail, StringComparison.Ordinal);
            Leave(clients);
        }
    }

    /// <summary>
    /// A game of four: the player the game waits on runs out of time, loses and leaves (CR 104.3a, 800.4a), and the game goes
    /// on for the other three - a later question is someone else's - with no winner yet.
    /// </summary>
    [Fact]
    public async Task In_a_game_of_four_a_player_out_of_time_leaves_and_the_others_play_on()
    {
        if (NoHost()) return;
        var clock = new Clock();
        (GameServer server, List<GameClient> clients) = await LobbyAsync(clock, players: 4);
        await using (server)
        {
            TimeSpan each = MatchClocks.DefaultPlayerClock;
            Assert.All(clients[0].Clock!.Clocks, clock => Assert.Equal((long)each.TotalMilliseconds, clock.LeftMs));
            int waiting = clients[0].Clock!.Clocks.First(clock => clock.Running).Seat;
            clock.Advance(each + TimeSpan.FromMinutes(1));

            GameClient other = clients.First(client => client.Seat != waiting);
            await Until(() => clients.Any(client => client.Seat != waiting && client.Prompt is not null)
                && other.View?.Players.Single(player => player.Seat == waiting).Status != PlayerStatusView.Active, [.. clients]);
            Assert.False(server.Finished.IsCompleted, "three players are still in");
            Assert.Equal(3, other.View!.Players.Count(player => player.Status == PlayerStatusView.Active));
            // The player out of time is told so once the others' run is published (the host shows a run to its answering
            // player alone), and is asked nothing more.
            GameClient gone = clients.Single(client => client.Seat == waiting);
            HashSet<int> answered = [];
            await Until(() =>
            {
                foreach (GameClient client in clients.Where(client => client != gone))
                {
                    if (client.Prompt is { } asked && answered.Add(asked.RequestId)) client.Answer(asked.Prompt.Default());
                }

                return gone.View!.Players.Single(player => player.Seat == waiting).Status != PlayerStatusView.Active;
            }, [.. clients]);
            Assert.Null(gone.Prompt);
            Assert.False(server.Finished.IsCompleted, "three players are still in");
            Leave(clients);
        }
    }

    /// <summary>
    /// A scenario's table has the same clocks, on the fake time it is given: the driven seat, asked and never answering, runs
    /// out of time after its hour and loses.
    /// </summary>
    [Fact]
    public async Task A_scenario_table_has_the_clocks_and_runs_them_on_the_time_it_is_given()
    {
        if (NoHost()) return;
        var clock = new Clock();
        ScenarioSpec scenario = ScenarioSpec.Load(Path.Combine(RepoRoot.Path, "scenarios", "duress-takes-the-named-card.cantrip"), [1]);
        await using ScenarioTable table = await ScenarioTable.StartAsync(scenario, RepoRoot.ContentRoot, clock);
        var client = new GameClient(await WebSocketTransport.ConnectAsync(table.Address));
        client.Join("You", 1);
        await Until(() => client.Clock is { Clocks.Count: 2 }, client);
        Assert.All(client.Clock!.Clocks, each => Assert.Equal((long)TwoPlayerClock.TotalMilliseconds, each.LeftMs));

        clock.Advance(TwoPlayerClock + TimeSpan.FromMinutes(1));
        TableResult result = await FinishedAsync(table.Server);
        Assert.Contains("ran out of time", result.Detail, StringComparison.Ordinal);
        client.Disconnect();
    }

    /// <summary>
    /// An event that gives one seat a clock and has no record of the other (a seat with no participant): both seats still
    /// have clocks - the other the table's own - rather than neither.
    /// </summary>
    [Fact]
    public async Task One_seat_without_a_clock_of_the_events_never_stops_the_others()
    {
        if (NoHost()) return;
        var clock = new Clock();
        var hosted = new Event { Clocks = { [1] = TimeSpan.FromMinutes(10) } };
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, Event = hosted, TimeProvider = clock,
        });
        await Until(() => hosted.Used.Count == 2 || hosted.Ended is not null);
        Assert.Equal(TimeSpan.FromMinutes(10), hosted.Used[1]);
        Assert.Equal(TwoPlayerClock, hosted.Used[2]);

        clock.Advance(TwoPlayerClock + TimeSpan.FromMinutes(1));
        TableResult result = await FinishedAsync(server);
        Assert.Contains("ran out of time", result.Detail, StringComparison.Ordinal);
        Assert.NotEmpty(hosted.RanOut);
    }

    /// <summary>
    /// A game played in one process (HostAndPlay: the bot labs, the language-model games, the playtests) has the clocks too:
    /// whoever plays it charges them as it looks (HostedGame.Tick), and a player the game waits on who never acts loses.
    /// </summary>
    [Fact]
    public void A_game_in_one_process_has_the_clocks_and_a_player_who_never_acts_loses()
    {
        if (NoHost()) return;
        var clock = new Clock();
        using Cantrip.Client.Embedded.HostedGame hosted = Cantrip.Client.Embedded.HostAndPlay.StartTable(
            RulesHostFactory.Native, [Deck()], 7, ["Ann", "Bob"], clock);
        List<GameClient> clients = [.. hosted.Seats.Order().Select(seat =>
        {
            var client = new GameClient(hosted.TransportFor(seat));
            client.Join(seat == 1 ? "Ann" : "Bob");
            return client;
        })];
        hosted.Start();
        hosted.Tick();
        Assert.All(hosted.Clocks!.Left.Values, left => Assert.Equal(TwoPlayerClock, left));
        int waiting = Assert.Single(hosted.Clocks.Running);

        clock.Advance(TwoPlayerClock + TimeSpan.FromMinutes(1));
        hosted.Tick();
        foreach (GameClient client in clients) client.Pump();
        Assert.All(clients, client => Assert.Equal(GameOutcomeView.TeamWon, client.Outcome?.Outcome));
        Assert.Contains(clients[0].View!.Players, player => player.Seat == waiting && player.Status != PlayerStatusView.Active);
    }

    /// <summary>
    /// A seat dealt a game with no time left on its clock (an event's, from its earlier games) runs out at the table's first
    /// look and loses at once, the other winning.
    /// </summary>
    [Fact]
    public async Task A_seat_dealt_a_game_with_no_time_left_loses_at_once()
    {
        if (NoHost()) return;
        var clock = new Clock();
        var hosted = new Event { Clocks = { [1] = TimeSpan.Zero } };
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, Event = hosted, TimeProvider = clock,
        });
        TableResult result = await FinishedAsync(server);
        Assert.Equal([2], result.WinningSeats);
        Assert.Equal([1], hosted.RanOut);
        Assert.Equal("Player 1 ran out of time.", result.Detail);
    }

    /// <summary>
    /// A game its players agree to draw (docs/rules-exceptions.md, RX-4) at an event's table: seat one offers at its priority,
    /// seat two accepts, and the table ends with nobody winning, saying why, and the event is told it was a draw - as it is
    /// told of any other draw - and every seat that the game is over, drawn.
    /// </summary>
    [Fact]
    public async Task A_draw_agreed_at_an_events_table_is_a_draw_for_the_event()
    {
        var clock = new Clock();
        var hosted = new Event();
        await using GameServer server = await GameServer.StartAsync(new GameServerOptions
        {
            Port = 0, Players = 2, ContentRoot = RepoRoot.ContentRoot, Event = hosted, TimeProvider = clock, Rules = new JumpingRules(clock, TimeSpan.Zero),
        });
        var address = new Uri($"ws://127.0.0.1:{server.Port}/game");
        List<GameClient> clients = [];
        for (int seat = 1; seat <= 2; seat++)
        {
            var client = new GameClient(await WebSocketTransport.ConnectAsync(address));
            client.Join($"Player {seat}", seat, $"seat-{seat}");
            clients.Add(client);
        }

        await Until(() => clients[0].CanOfferDraw, [.. clients]);
        clients[0].OfferDraw();
        await Until(() => clients[1].DrawAskedOfMe && clients[1].CanAcceptDraw, [.. clients]);
        clients[1].AnswerDraw(accept: true);

        TableResult result = await FinishedAsync(server);
        Assert.Equal(TablePhase.Finished, result.Phase);
        Assert.Empty(result.WinningSeats);
        Assert.Equal("The players agreed to a draw.", result.Detail);
        Assert.False(hosted.Ended!.Value.Abandoned);
        Assert.Empty(hosted.Ended.Value.Winners);
        await Until(() => clients.TrueForAll(client => client.Outcome?.Outcome == GameOutcomeView.Draw), [.. clients]);
        Leave(clients);
    }

    /// <summary>Scripted rules (<see cref="ScriptedHost"/>) whose concession moves the fake time on as it is made.</summary>
    private sealed class JumpingRules(Clock clock, TimeSpan jump) : IRulesHostFactory
    {
        public IRulesHost Create(IReadOnlyList<RulesSeat> seats, ulong seed, RulesGameOptions options) =>
            NativeRulesHost.NewGame(new Jumping(clock, jump), seats, seed, options);

        private sealed class Jumping(Clock clock, TimeSpan jump) : IHostTransport
        {
            private readonly ScriptedHost _host = new();

            public string Exchange(string request)
            {
                string reply = _host.Exchange(request);
                if (request.Contains("\"Concede\"", StringComparison.Ordinal)) clock.Advance(jump);
                return reply;
            }

            public void Dispose()
            {
            }
        }
    }

    /// <summary>An event of two seats, ready to deal a game of the demo deck, that records what the table tells it.</summary>
    private sealed class Event : ITableEvent
    {
        public Dictionary<int, TimeSpan> Clocks { get; } = [];

        public Dictionary<int, TimeSpan> Used { get; } = [];

        public List<int> RanOut { get; } = [];

        public int Dealt { get; private set; }

        public (bool Abandoned, IReadOnlyList<int> Winners)? Ended { get; private set; }

        public string EventId => "clock-test";

        public string Name => "Clock test";

        public int Seats => 2;

        public TablePhase Phase => TablePhase.DeckBuilding;

        public bool ReadyToDeal => Ended is null;

        public int SeatFor(string ticket) => ticket.StartsWith("seat-", StringComparison.Ordinal) && int.TryParse(ticket[5..], out int seat) ? seat : 0;

        public string NameAt(int seat) => $"Player {seat}";

        public void Seated(int seat)
        {
        }

        public ServerMessage? ViewFor(int seat) => null;

        public string? Receive(int seat, ClientMessage message) => null;

        public IRulesHost Deal(IReadOnlyList<string> names, IRulesHostFactory? rules = null)
        {
            Dealt++;
            List<string> cards = [.. Deck().Entries.SelectMany(entry => Enumerable.Repeat(entry.Card, entry.Copies))];
            return (rules ?? RulesHostFactory.Native).Create([.. names.Select(name => new RulesSeat(name, cards))], 7, new RulesGameOptions());
        }

        public void DealFailed(string why) => throw new InvalidOperationException(why);

        public void GameEnded(bool abandoned, IReadOnlyList<int> winningSeats, string detail) => Ended = (abandoned, winningSeats);

        public TimeSpan? ClockAtDeal(int seat) => Clocks.TryGetValue(seat, out TimeSpan left) ? left : null;

        public void ClockUsed(int seat, TimeSpan left) => Used[seat] = left;

        public void ClockRanOut(int seat) => RanOut.Add(seat);
    }
}
