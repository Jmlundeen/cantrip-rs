using System.Net;
using System.Net.Sockets;
using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Discord.Tests;

/// <summary>
/// The matchmaker behind the bot, with real tables on real sockets and the real RW Aggro list. Nothing here depends on
/// how the cards fall: the games are only ever dealt, left or cancelled.
/// </summary>
[Trait("Category", "WallClock")]
public sealed class MatchmakerTests
{
    private static readonly DiscordPlayer Alice = new(1001, "Alice");
    private static readonly DiscordPlayer Bob = new(1002, "Bob");
    private static readonly DiscordPlayer Carol = new(1003, "Carol");

    private static readonly Lazy<DeckList> RwAggro = new(() =>
    {
        DeckList deck = DeckList.Load(DeckList.Find(Path.Combine(RepoRoot.Path, "decks"), "rw-aggro"));
        deck.Resolve([ContentCache.Get(RepoRoot.ContentRoot).Cards]);
        return deck;
    });

    private sealed class Clock : TimeProvider
    {
        private long _ticks = new DateTimeOffset(2026, 9, 27, 0, 0, 0, TimeSpan.Zero).Ticks;
        public override DateTimeOffset GetUtcNow() => new(Interlocked.Read(ref _ticks), TimeSpan.Zero);
        public void Advance(TimeSpan duration) => Interlocked.Add(ref _ticks, duration.Ticks);
    }

    private static Matchmaker Create(int tables = 2, Clock? clock = null, ReturnPage? back = null) => new(new MatchmakerOptions
    {
        ContentRoot = RepoRoot.ContentRoot,
        Deck = RwAggro.Value,
        Ports = FreePorts(tables),
        TimeProvider = (TimeProvider?)clock ?? TimeProvider.System,
        Return = back,
    });

    [Fact]
    public async Task A_games_table_names_the_constructed_page_to_go_back_to_once_it_is_over()
    {
        var page = new ReturnPage(new Uri("https://cantrip.example/constructed"), "Constructed Play");
        await using Matchmaker matchmaker = Create(back: page);
        Seating hosted = await matchmaker.HostAsync(Alice);
        GameClient alice = await Connect(hosted.Game!, hosted.Ticket!);
        try
        {
            await Until(() => alice.Table is not null, alice);
            Assert.Equal(("https://cantrip.example/constructed", "Constructed Play"), (alice.Table!.Return, alice.Table.ReturnName));
        }
        finally
        {
            alice.Disconnect();
        }
    }

    /// <summary>Ports nothing is listening on right now, as the pool a host would configure.</summary>
    private static List<int> FreePorts(int count)
    {
        List<TcpListener> listeners = [.. Enumerable.Range(0, count).Select(_ => new TcpListener(IPAddress.Loopback, 0))];
        listeners.ForEach(listener => listener.Start());
        List<int> ports = [.. listeners.Select(listener => ((IPEndPoint)listener.LocalEndpoint).Port)];
        listeners.ForEach(listener => listener.Stop());
        return ports;
    }

    private static async Task<GameClient> Connect(HostedGame game, string ticket)
    {
        var client = new GameClient(await WebSocketTransport.ConnectAsync(game.Address));
        client.Join("ignored", ticket: ticket);
        await Until(() => client.Membership is not null || client.Errors.Count > 0, client);
        return client;
    }

    private static async Task Until(Func<bool> condition, params GameClient[] clients)
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(60));
        while (true)
        {
            foreach (GameClient client in clients) client.Pump();
            if (condition()) return;
            await Task.Delay(1, timeout.Token);
        }
    }

    private static async Task Settle(Func<bool> condition)
    {
        using var timeout = new CancellationTokenSource(TimeSpan.FromSeconds(30));
        while (!condition()) await Task.Delay(1, timeout.Token);
    }

    [Fact]
    public async Task Two_players_are_seated_by_their_tickets_as_themselves_with_rw_aggro_and_the_game_is_dealt()
    {
        await using Matchmaker matchmaker = Create();
        List<GameStage> seen = [];
        matchmaker.Changed += game =>
        {
            lock (seen) seen.Add(game.Stage);
            return Task.CompletedTask;
        };

        Seating hosted = await matchmaker.HostAsync(Alice);
        Assert.Null(hosted.Refusal);
        HostedGame game = hosted.Game!;
        Assert.Equal((GameStage.Open, 1), (game.Stage, hosted.Seat));

        Seating joined = await matchmaker.JoinAsync(game.Id, Bob);
        Assert.Equal((GameStage.Seated, 2), (game.Stage, joined.Seat));
        Assert.NotEqual(hosted.Ticket, joined.Ticket);

        GameClient alice = await Connect(game, hosted.Ticket!);
        GameClient bob = await Connect(game, joined.Ticket!);
        try
        {
            Assert.Equal((1, 2), (alice.Membership!.Seat, bob.Membership!.Seat));
            await Until(() => alice.View is not null && bob.View is not null, alice, bob);
            Assert.Equal(new[] { ("Alice", "rw-aggro"), ("Bob", "rw-aggro") },
                alice.Table!.Players.Select(player => (player.Name, player.Deck)));
            await Settle(() => game.Stage == GameStage.Playing);

            // Leaving mid-game abandons it, which the matchmaker reports as the game's ending.
            bob.Leave();
            await Settle(() => game.Stage == GameStage.Ended);
            Assert.Equal(GameEnding.Abandoned, game.Ending);
            Assert.Empty(game.Winners);
            lock (seen) Assert.Equal(new[] { GameStage.Seated, GameStage.Playing, GameStage.Ended }, seen);
        }
        finally
        {
            alice.Disconnect();
            bob.Disconnect();
        }
    }

    /// <summary>
    /// The server checks a deck against its table's format before seating anyone with it (DeckValidation): a forty-card deck
    /// offered for Standard is refused, saying why, and the table opened for it is closed again rather than left running.
    /// </summary>
    [Fact]
    public async Task A_deck_its_format_refuses_seats_nobody_and_leaves_no_table_open()
    {
        await using var matchmaker = new Matchmaker(new MatchmakerOptions
        {
            ContentRoot = RepoRoot.ContentRoot,
            Deck = RwAggro.Value,
            Formats = new Dictionary<string, IReadOnlyList<DeckList>> { ["standard"] = [RwAggro.Value] },
            Ports = FreePorts(1),
        });

        Seating refused = await matchmaker.HostAsync(Alice, "standard", "rw-aggro");
        Assert.Null(refused.Game);
        Assert.StartsWith("rw-aggro can't be played in Standard: A Standard deck is at least 60 cards", refused.Refusal, StringComparison.Ordinal);
        Assert.Empty(matchmaker.Games("standard"));
        Assert.Null(matchmaker.GameOf(Alice.Id));
        // The port is free again: a practice game, Limited, opens on it.
        Assert.NotNull((await matchmaker.HostAsync(Alice)).Game);
    }

    [Fact]
    public async Task A_constructed_games_players_each_sit_down_with_the_deck_they_chose_from_its_formats_list()
    {
        DeckList reanimator = DeckList.Load(DeckList.Find(Path.Combine(RepoRoot.Path, "decks"), "ub-reanimator"));
        await using var matchmaker = new Matchmaker(new MatchmakerOptions
        {
            ContentRoot = RepoRoot.ContentRoot,
            Deck = RwAggro.Value,
            Formats = new Dictionary<string, IReadOnlyList<DeckList>> { ["limited"] = [RwAggro.Value, reanimator] },
            Ports = FreePorts(2),
        });

        Assert.NotNull((await matchmaker.HostAsync(Alice, "modern", "rw-aggro")).Refusal);
        Assert.NotNull((await matchmaker.HostAsync(Alice, "limited", "legend-rule-test")).Refusal);
        Seating hosted = await matchmaker.HostAsync(Alice, "limited", "ub-reanimator");
        HostedGame game = hosted.Game!;
        Assert.Equal(("limited", GameStage.Open), (game.Format, game.Stage));
        Assert.Same(game, matchmaker.GameOf(Alice.Id));

        // A game hosted from Discord is no game of the format's.
        Seating discord = await matchmaker.HostAsync(Carol);
        Assert.Null(discord.Game!.Format);
        Assert.Equal([game], matchmaker.Games("limited"));

        // A seat wants a deck from the list; the host asking again gets their own seat back, deck or none.
        Assert.NotNull((await matchmaker.JoinAsync(game.Id, Bob)).Refusal);
        Assert.NotNull((await matchmaker.JoinAsync(game.Id, Bob, "nope")).Refusal);
        Assert.Equal(GameStage.Open, game.Stage);
        Assert.Equal(hosted.Ticket, (await matchmaker.JoinAsync(game.Id, Alice)).Ticket);
        Seating joined = await matchmaker.JoinAsync(game.Id, Bob, "rw-aggro");
        Assert.Equal((GameStage.Seated, 2), (game.Stage, joined.Seat));

        GameClient alice = await Connect(game, hosted.Ticket!);
        GameClient bob = await Connect(game, joined.Ticket!);
        try
        {
            await Until(() => alice.View is not null && bob.View is not null, alice, bob);
            Assert.Equal(new[] { ("Alice", "ub-reanimator"), ("Bob", "rw-aggro") },
                alice.Table!.Players.Select(player => (player.Name, player.Deck)));
        }
        finally
        {
            alice.Disconnect();
            bob.Disconnect();
        }
    }

    [Fact]
    public async Task A_player_is_in_one_game_at_a_time_and_a_full_game_takes_nobody_else()
    {
        await using Matchmaker matchmaker = Create();
        HostedGame game = (await matchmaker.HostAsync(Alice)).Game!;

        Seating twice = await matchmaker.HostAsync(Alice);
        Assert.NotNull(twice.Refusal);
        Assert.Same(game, twice.Game);

        await matchmaker.JoinAsync(game.Id, Bob);
        Assert.NotNull((await matchmaker.JoinAsync(game.Id, Carol)).Refusal);
        Assert.NotNull((await matchmaker.HostAsync(Bob)).Refusal);
        Assert.NotNull((await matchmaker.JoinAsync("nosuchgame", Carol)).Refusal);
    }

    [Fact]
    public async Task Pressing_join_again_hands_a_seated_player_their_own_ticket()
    {
        await using Matchmaker matchmaker = Create();
        Seating hosted = await matchmaker.HostAsync(Alice);
        Seating joined = await matchmaker.JoinAsync(hosted.Game!.Id, Bob);

        Seating host = await matchmaker.JoinAsync(hosted.Game.Id, Alice);
        Seating guest = await matchmaker.JoinAsync(hosted.Game.Id, Bob);
        Assert.Equal((true, 1, hosted.Ticket), (host.Again, host.Seat, host.Ticket));
        Assert.Equal((true, 2, joined.Ticket), (guest.Again, guest.Seat, guest.Ticket));
    }

    [Fact]
    public async Task Only_the_host_or_a_manager_may_cancel_and_cancelling_frees_the_table()
    {
        await using Matchmaker matchmaker = Create(tables: 1);
        HostedGame first = (await matchmaker.HostAsync(Alice)).Game!;
        Assert.Equal("Every table is in use. Try again when a game finishes.", (await matchmaker.HostAsync(Carol)).Refusal);

        Assert.NotNull(await matchmaker.CancelAsync(first.Id, Carol, moderator: false));
        Assert.Null(await matchmaker.CancelAsync(first.Id, Alice, moderator: false));
        Assert.Equal((GameEnding.Cancelled, Alice), (first.Ending, first.CancelledBy));
        Assert.NotNull((await matchmaker.JoinAsync(first.Id, Bob)).Refusal);

        HostedGame second = (await matchmaker.HostAsync(Carol)).Game!;
        Assert.Equal(first.Port, second.Port);
        Assert.Null(await matchmaker.CancelAsync(second.Id, Bob, moderator: true));
        Assert.Equal(Bob, second.CancelledBy);
    }

    [Fact]
    public async Task A_game_that_never_starts_expires_and_an_ended_one_closes_after_it_lingers()
    {
        var clock = new Clock();
        await using Matchmaker matchmaker = Create(tables: 1, clock);
        List<HostedGame> changed = [];
        matchmaker.Changed += game =>
        {
            lock (changed) changed.Add(game);
            return Task.CompletedTask;
        };

        HostedGame stale = (await matchmaker.HostAsync(Alice)).Game!;
        clock.Advance(TimeSpan.FromMinutes(29));
        await matchmaker.SweepAsync();
        Assert.Equal(GameStage.Open, stale.Stage);

        clock.Advance(TimeSpan.FromMinutes(1));
        await matchmaker.SweepAsync();
        Assert.Equal((GameStage.Ended, GameEnding.Expired), (stale.Stage, stale.Ending));
        lock (changed) Assert.Same(stale, Assert.Single(changed));
        Assert.Null(matchmaker.Find(stale.Id));
        HostedGame next = (await matchmaker.HostAsync(Bob)).Game!;
        Assert.Equal(stale.Port, next.Port);
    }

    [Fact]
    public async Task A_game_that_ends_in_play_keeps_its_table_until_the_result_has_lingered()
    {
        var clock = new Clock();
        await using Matchmaker matchmaker = Create(tables: 1, clock);
        Seating hosted = await matchmaker.HostAsync(Alice);
        HostedGame game = hosted.Game!;
        Seating joined = await matchmaker.JoinAsync(game.Id, Bob);
        GameClient alice = await Connect(game, hosted.Ticket!);
        GameClient bob = await Connect(game, joined.Ticket!);
        try
        {
            await Settle(() => game.Stage == GameStage.Playing);
            alice.Leave();
            await Settle(() => game.Stage == GameStage.Ended);

            // Ended, but Bob's client is still showing the result, so the table and its port stay.
            await matchmaker.SweepAsync();
            Assert.NotNull((await matchmaker.HostAsync(Carol)).Refusal);

            clock.Advance(TimeSpan.FromMinutes(2));
            await matchmaker.SweepAsync();
            Assert.Null(matchmaker.Find(game.Id));
            Assert.Equal(game.Port, (await matchmaker.HostAsync(Carol)).Game!.Port);
        }
        finally
        {
            alice.Disconnect();
            bob.Disconnect();
        }
    }

    /// <summary>
    /// A constructed game has the chess clocks every table has (an hour each): a player the game
    /// waits on who never acts runs out of time and loses, so nobody can hold the table open by sitting at it.
    /// </summary>
    [Fact]
    public async Task A_constructed_game_has_player_clocks_and_whoever_never_acts_runs_out_of_time()
    {
        var clock = new Clock();
        await using Matchmaker matchmaker = Create(tables: 1, clock);
        Seating hosted = await matchmaker.HostAsync(Alice);
        HostedGame game = hosted.Game!;
        Seating joined = await matchmaker.JoinAsync(game.Id, Bob);
        GameClient alice = await Connect(game, hosted.Ticket!);
        GameClient bob = await Connect(game, joined.Ticket!);
        try
        {
            await Settle(() => game.Stage == GameStage.Playing);
            await Settle(() =>
            {
                alice.Pump();
                return alice.Clock is { Clocks.Count: 2 } clocks && clocks.Clocks.All(each => each.LeftMs == 60 * 60 * 1000);
            });

            clock.Advance(TimeSpan.FromMinutes(61));
            await Settle(() => game.Stage == GameStage.Ended);
            Assert.Contains("ran out of time", game.Detail, StringComparison.Ordinal);
        }
        finally
        {
            alice.Disconnect();
            bob.Disconnect();
        }
    }

    [Fact]
    public void Discord_names_become_names_a_table_accepts()
    {
        Assert.Equal("Alice", Matchmaker.SeatName("  Alice\n"));
        Assert.Equal("Player", Matchmaker.SeatName("\u0007 "));
        Assert.Equal(new string('x', 32), Matchmaker.SeatName(new string('x', 40)));
        Assert.Equal(new string('x', 31), Matchmaker.SeatName(new string('x', 31) + "\U0001F600"));
    }

    [Fact]
    public void Hosting_needs_a_host_role_or_manage_server()
    {
        var anyone = new HostPolicy([]);
        Assert.False(anyone.MayHost([5], managesServer: false));
        Assert.True(anyone.MayHost([], managesServer: true));

        var organizers = new HostPolicy([7]);
        Assert.True(organizers.MayHost([5, 7], managesServer: false));
        Assert.False(organizers.MayHost([5], managesServer: false));
        Assert.Contains("<@&7>", organizers.Refusal, StringComparison.Ordinal);
    }

    [Fact]
    public async Task The_ticket_reply_carries_the_address_and_ticket_and_the_table_names_both_players()
    {
        await using Matchmaker matchmaker = Create();
        Seating hosted = await matchmaker.HostAsync(Alice);
        HostedGame game = hosted.Game!;
        // The ticket rides only in the reply's button link, never in its text.
        Assert.Contains("Open in Cantrip", GameMessages.Ticket(hosted), StringComparison.Ordinal);
        Assert.DoesNotContain(hosted.Ticket!, GameMessages.Ticket(hosted), StringComparison.Ordinal);
        Assert.Contains("Join", GameMessages.Table(game, "rw-aggro"), StringComparison.Ordinal);

        await matchmaker.JoinAsync(game.Id, Bob);
        Assert.Contains("**Alice** vs **Bob**", GameMessages.Table(game, "rw-aggro"), StringComparison.Ordinal);
        Assert.DoesNotContain("<@", GameMessages.Table(game, "rw-aggro"), StringComparison.Ordinal);
        await matchmaker.CancelAsync(game.Id, Alice, moderator: false);
        Assert.Contains("Cancelled by **Alice**", GameMessages.Table(game, "rw-aggro"), StringComparison.Ordinal);
    }

    [Fact]
    public void A_name_is_shown_as_written_and_cannot_format_or_mention()
    {
        Assert.Equal("**x\\_y\\*z\\*\\_**", GameMessages.Name(new DiscordPlayer(1, "x_y*z*_")));
        Assert.Equal("**\\@everyone \\<\\@1\\>**", GameMessages.Name(new DiscordPlayer(1, "@everyone <@1>")));
        Assert.Equal("**Plain Name**", GameMessages.Name(new DiscordPlayer(1, "Plain Name")));
    }
}
