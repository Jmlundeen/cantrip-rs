using Cantrip.Client;
using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// A booster draft of a pod of eight: drafted, built and played through its Swiss rounds, each game at a table of its
/// own. The whole event is played over real sockets by bot seats that follow their moves from table to table; the parts
/// that need a clock are driven without tables, on a clock the test moves.
/// </summary>
public sealed class BoosterPodTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-pod-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private static Cantrip.Datasets.ContentLibrary Content => ContentCache.Get(RepoRoot.ContentRoot);

    private static readonly IReadOnlyList<(string, string)> Eight =
        [("ann", "Ann"), ("ben", "Ben"), ("cat", "Cat"), ("dan", "Dan"), ("eve", "Eve"), ("fay", "Fay"), ("gus", "Gus"), ("hal", "Hal")];

    /// <summary>
    /// A cube of the RW Aggro list three times over: a hundred and twenty cards whose games the bots play cleanly, for the
    /// pod that is played out. What is tested is the pod, not every card of a real cube.
    /// </summary>
    private static IReadOnlyList<string> RwAggroThrice =>
        [.. Enumerable.Repeat(CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")), 3).SelectMany(cards => cards)];

    private static string LsvCube => Path.Combine(RepoRoot.Path, "docs", "cube-lists", "LSVCube.txt");

    private CreatedEvent Create(EventStore store, int packSize = 5, int minimumDeck = 40, IReadOnlyList<string>? cube = null, IReadOnlyList<string>? pickOrder = null) =>
        BoosterPod.Create(store, Content, new BoosterDraftSetup
        {
            Name = "Friday pod",
            Cube = cube is null ? "LSVCube" : "RW Aggro x3",
            CubeCards = cube ?? CubeList.Load(LsvCube),
            PickOrder = pickOrder ?? [],
            Packs = 3,
            PackSize = packSize,
            Rounds = 3,
            MinimumDeck = minimumDeck,
            Seed = 21,
            Players = Eight,
        });

    private sealed class Clock : TimeProvider
    {
        private DateTimeOffset _now = new(2026, 10, 1, 18, 0, 0, TimeSpan.Zero);
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan by) => _now += by;
    }

    [Fact]
    [Trait("Category", "WallClock")]
    public async Task Eight_bots_draft_build_and_play_three_swiss_rounds_each_game_at_its_own_table()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store, cube: RwAggroThrice);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, timing: new BoosterTiming
        {
            FirstRound = TimeSpan.FromMilliseconds(200),
            BetweenRounds = TimeSpan.FromMilliseconds(200),
            AfterGame = TimeSpan.FromMilliseconds(100),
        });

        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(RepoRoot.ContentRoot));
        List<EventSeat> seats = [];
        foreach ((string id, string name) in Eight)
        {
            seats.Add(await EventSeat.JoinAsync(host.Address, created.Tickets[id], name));
        }

        using var stop = new CancellationTokenSource(TimeSpan.FromMinutes(4));
        await Task.WhenAll(seats.Select(seat => Task.Run(async () =>
        {
            while (!seat.Finished && !stop.IsCancellationRequested)
            {
                if (!await seat.StepAsync(stop.Token))
                {
                    await Task.Delay(10, CancellationToken.None);
                }
            }
        })));

        Assert.Null(host.Fault);
        Assert.False(stop.IsCancellationRequested, $"The event stopped at: {pod.Detail}\n" + string.Join("\n", seats.Select(seat =>
            $"{seat.Client.Seat} at {seat.Address.Port}: moves {seat.Moves}, games {seat.GamesPlayed}, phase {seat.Client.Table?.Phase}, " +
            $"over {seat.Client.IsOver}, connected {seat.Client.IsConnected}, asked {seat.Client.Prompt?.Prompt?.GetType().Name}, answered {seat.LastAnswered}, " +
            $"deck {(seat.Client.DeckBuilding is { } deck ? $"submitted {deck.Submitted}" : "-")}, " +
            $"turn {seat.Client.View?.Turn} {seat.Client.View?.Step} priority {seat.Client.View?.PriorityPlayer} " +
            $"life {string.Join("/", seat.Client.View?.Players.Select(player => player.Life) ?? [])}, log {seat.Client.Log.Count}, " +
            $"error {seat.Client.Errors.LastOrDefault()?.Detail}" +
            (seat.Client.Table?.Phase == TablePhase.Playing ? "\n    " + string.Join("\n    ", seat.Client.Log.Select(line => $"{line.Turn} {line.Step}: {line.Text} [{string.Join(", ", line.References.Select(each => each.Name))}]")) : string.Empty))));

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(EventStage.Complete, record.Stage);
        Assert.Equal(12, record.Matches.Count);
        Assert.All(record.Matches, match => Assert.True(match.Complete));
        Assert.All(record.Matches, match => Assert.False(match.Forfeit));
        Assert.All(record.Matches, match => Assert.InRange(match.Games.Count, 2, 3));
        Assert.Equal(12, record.Matches.Select(match => string.Join("-", match.Players.Order())).Distinct().Count());

        // Every pick is in the log, each player's fifteen in the order they were made.
        BoosterDraft draft = store.LoadDraft<BoosterDraft>(created.Record.Id)!;
        Assert.Equal(120, draft.Log.Count);
        foreach (EventParticipant participant in record.Participants)
        {
            List<BoosterPick> mine = [.. draft.Log.Where(pick => pick.Seat == participant.Seat)];
            Assert.Equal(15, mine.Count);
            Assert.Equal(participant.Pool, mine.Select(pick => pick.Card));
            Assert.Equal([1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3], mine.Select(pick => pick.Pack));
        }

        PairingsMessage final = seats[0].Client.Pairings!;
        Assert.True(final.Final);
        Assert.Equal(8, final.Standings.Count);
        Assert.Equal(Enumerable.Range(1, 8), final.Standings.Select(standing => standing.Rank));
        Assert.All(seats, seat => Assert.InRange(seat.GamesPlayed, 6, 9));
        foreach (EventSeat seat in seats) seat.Dispose();
    }

    [Fact]
    [Trait("Category", "WallClock")]
    public async Task A_player_whose_match_clock_runs_out_while_the_game_waits_on_them_loses_the_whole_match()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, cube: RwAggroThrice);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(RepoRoot.ContentRoot, clock));

        // Drafted and built by the clocks, and the first round begun.
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        while (pod.Stage == EventStage.Drafting)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        clock.Advance(TimeSpan.FromSeconds(15));
        pod.Tick(clock.GetUtcNow());

        // Ann plays; Eve, her opponent, comes to the table and never answers anything.
        using var stop = new CancellationTokenSource(TimeSpan.FromMinutes(2));
        EventSeat ann = await EventSeat.JoinAsync(host.Address, created.Tickets["ann"], "Ann");
        Task annPlays = Task.Run(async () =>
        {
            while (!stop.IsCancellationRequested && ann.GamesPlayed == 0)
            {
                if (!await ann.StepAsync()) await Task.Delay(10, CancellationToken.None);
            }
        });

        var eve = new GameClient(await WebSocketTransport.ConnectAsync(host.Address));
        eve.Join("Eve", 0, created.Tickets["eve"]);
        while (eve.Moved is null && !stop.IsCancellationRequested)
        {
            eve.Pump();
            await Task.Delay(10);
        }

        TableMoveMessage moved = eve.Moved!;
        eve.Disconnect();
        eve = new GameClient(await WebSocketTransport.ConnectAsync(new Uri(moved.Server)));
        eve.Join("Eve", 0, moved.Ticket);

        // The game waits on Eve alone - her mulligan - and her clock alone runs.
        while (!stop.IsCancellationRequested && (eve.Prompt is null || eve.Clock?.Clocks is not [{ Running: false }, { Running: true }]))
        {
            eve.Pump();
            await Task.Delay(10);
        }

        Assert.Equal(25 * 60 * 1000, eve.Clock!.Clocks[1].LeftMs, tolerance: 1000);
        Assert.NotNull(eve.Prompt);

        clock.Advance(TimeSpan.FromMinutes(26));
        await annPlays;
        Assert.False(stop.IsCancellationRequested, "Ann's game never ended.");

        MatchRecord match = store.LoadEvent(created.Record.Id)!.Matches.Single(each => each.Round == 1 && each.Players.Contains("ann"));
        Assert.True(match.Complete);
        Assert.Equal(["ann"], match.Winners);
        Assert.Equal("eve", match.TimedOut);
        Assert.Equal(0, match.ClockMs["eve"]);
        Assert.Contains("Eve ran out of time", match.Detail);
        Assert.Single(match.Games);
        Assert.Equal(GameOutcomeView.TeamWon, ann.Client.Outcome!.Outcome);
        ann.Dispose();
        eve.Disconnect();
    }

    [Fact]
    public void Everyone_sees_how_many_packs_wait_for_each_seat_and_a_clock_that_runs_out_picks_for_its_player()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 15);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);

        var view = (DraftMessage)pod.ViewFor(1)!;
        Assert.Equal(("booster", 1, 1, 15, 1), (view.Format, view.PackNumber, view.PickNumber, view.Pile.Count, view.Direction));
        Assert.Equal(75_000, view.PickLeftMs);

        // Seat 1 picks; its pack waits behind seat 2's own, and everyone sees two packs at seat 2.
        Assert.Null(pod.Receive(1, new DraftPickMessage { Decision = 0, Take = true, Card = 0 }));
        view = (DraftMessage)pod.ViewFor(5)!;
        Assert.Equal([0, 2, 1, 1, 1, 1, 1, 1], view.Seats.Select(seat => seat.Queued));

        // A second click on the same pick is refused, not taken from the next pack.
        Assert.NotNull(pod.Receive(1, new DraftPickMessage { Decision = 0, Take = true, Card = 0 }));

        clock.Advance(TimeSpan.FromSeconds(75));
        pod.Tick(clock.GetUtcNow());
        Assert.All(Enumerable.Range(2, 7), seat => Assert.Single(pod.Picks, pick => pick.Seat == seat && pick.Auto));
        Assert.Equal("Pack 1 pick 1: ", ((DraftMessage)pod.ViewFor(2)!).Recent[0][..15]);
        Assert.EndsWith("(time ran out)", ((DraftMessage)pod.ViewFor(2)!).Recent[0]);
    }

    [Fact]
    public void Each_seat_is_told_its_packs_pick_order_and_the_clock_takes_the_best_of_it()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        List<string> order = [.. CubeList.PickOrder(LsvCube)];
        CreatedEvent created = Create(store, packSize: 15, pickOrder: order);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);

        var view = (DraftMessage)pod.ViewFor(1)!;
        List<string> names = [.. store.LoadDraft<BoosterDraft>(created.Record.Id)!.Current(1)!.Cards];
        Assert.Equal(15, view.PileRanks.Count);
        Assert.Equal(names.OrderBy(name => order.IndexOf(name)), names.Zip(view.PileRanks).OrderBy(pair => pair.Second).Select(pair => pair.First));

        clock.Advance(TimeSpan.FromSeconds(75));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(names.MinBy(name => order.IndexOf(name)), pod.Picks.Single(pick => pick.Seat == 1).Card);
    }

    [Fact]
    public void A_seat_with_Cogwork_Librarian_drafted_is_offered_a_second_card_and_the_Librarian_goes_into_the_pack()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        // A hundred and twenty cards for packs of five, so every one is dealt - the Librarian among them.
        CreatedEvent created = Create(store, cube: ["Cogwork Librarian", .. RwAggroThrice.Skip(1)]);
        BoosterDraft draft = store.LoadDraft<BoosterDraft>(created.Record.Id)!;
        Assert.Equal(["Cogwork Librarian"], draft.AdditionalPicks);

        // The position: the Librarian first in seat 1's pack, wherever it was dealt.
        List<string> holding = draft.Queues.Select(queue => queue[0].Cards).Concat(draft.Unopened.SelectMany(round => round))
            .First(cards => cards.Contains("Cogwork Librarian"));
        (holding[holding.IndexOf("Cogwork Librarian")], draft.Queues[0][0].Cards[0]) = (draft.Queues[0][0].Cards[0], "Cogwork Librarian");
        store.SaveDraft(created.Record.Id, draft);

        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        Assert.Null(pod.Receive(1, new DraftPickMessage { Decision = 0, Take = true, Card = 0 }));
        Assert.Null(pod.Receive(8, new DraftPickMessage { Decision = 0, Take = true, Card = 0 }));

        var view = (DraftMessage)pod.ViewFor(1)!;
        Assert.Equal(4, view.Pile.Count);
        Assert.Equal([0], view.AdditionalPicks);
        Assert.Equal("Cogwork Librarian", view.Pool[view.AdditionalPicks[0]].Name);
        Assert.Empty(((DraftMessage)pod.ViewFor(2)!).AdditionalPicks);

        // Seat 2 has none to spend; seat 1 takes two cards with its one.
        Assert.NotNull(pod.Receive(2, new DraftPickMessage { Decision = 0, Take = true, Card = 0, Also = [1] }));
        Assert.NotNull(pod.Receive(1, new DraftPickMessage { Decision = 1, Take = true, Card = 0, Also = [1, 2] }));
        Assert.Null(pod.Receive(1, new DraftPickMessage { Decision = 1, Take = true, Card = 3, Also = [1] }));

        view = (DraftMessage)pod.ViewFor(1)!;
        Assert.Equal(2, view.Pool.Count);
        Assert.DoesNotContain(view.Pool, card => card.Name == "Cogwork Librarian");
        Assert.EndsWith("(for [Cogwork Librarian])", view.Recent[^1]);
        Assert.Equal("Cogwork Librarian", view.RecentCards[^1][^1].Card?.Name);
        draft = store.LoadDraft<BoosterDraft>(created.Record.Id)!;
        Assert.Equal("Cogwork Librarian", draft.QueueOf(2)[^1].Cards[^1]);
        Assert.Equal(3, draft.QueueOf(2)[^1].Cards.Count);
    }

    [Fact]
    public void A_clock_that_runs_out_asks_a_player_who_is_there_for_the_card_chosen_before_taking_one_at_random()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 15);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        // Seats 1 to 3 have the table open; the rest are not there.
        for (int seat = 1; seat <= 3; seat++) pod.Connected(seat, true);
        Assert.False(((DraftMessage)pod.ViewFor(1)!).PickDue);

        int version = pod.Version;
        clock.Advance(TimeSpan.FromSeconds(75));
        pod.Tick(clock.GetUtcNow());

        // Nobody who is there has had a card taken for them: they are asked, and shown so.
        Assert.All(Enumerable.Range(4, 5), seat => Assert.Single(pod.Picks, pick => pick.Seat == seat && pick.Auto));
        Assert.DoesNotContain(pod.Picks, pick => pick.Seat <= 3);
        Assert.NotEqual(version, pod.Version);
        Assert.All(Enumerable.Range(1, 3), seat => Assert.True(((DraftMessage)pod.ViewFor(seat)!).PickDue));

        // Seat 1 had a card chosen and says which; seat 2 had none and says so; seat 3 says nothing.
        string chosen = ((DraftMessage)pod.ViewFor(1)!).Pile[4].CardId;
        Assert.Null(pod.Receive(1, new DraftPickMessage { Decision = 0, Take = true, Card = 4 }));
        Assert.Null(pod.Receive(2, new DraftPickMessage { Decision = 0, Take = false }));
        BoosterPick first = Assert.Single(pod.Picks, pick => pick.Seat == 1);
        Assert.False(first.Auto);
        Assert.Equal(chosen, first.Card);
        Assert.True(Assert.Single(pod.Picks, pick => pick.Seat == 2).Auto);
        Assert.False(((DraftMessage)pod.ViewFor(1)!).PickDue);

        // Seat 3 is waited on only a moment.
        clock.Advance(TimeSpan.FromSeconds(1));
        pod.Tick(clock.GetUtcNow());
        Assert.DoesNotContain(pod.Picks, pick => pick.Seat == 3);
        clock.Advance(TimeSpan.FromSeconds(1));
        pod.Tick(clock.GetUtcNow());
        Assert.True(Assert.Single(pod.Picks, pick => pick.Seat == 3).Auto);

        // A pick that takes nothing is only an answer to being asked.
        Assert.NotNull(pod.Receive(1, new DraftPickMessage { Decision = 1, Take = false }));
    }

    [Fact]
    public void A_packs_last_card_is_shown_on_a_short_clock_and_taken_without_asking()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 2);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        pod.Connected(1, true);

        // Seat 1 picks at once; the clock takes for everyone else, and seat 8 passes seat 1 the last card of its pack.
        Assert.Null(pod.Receive(1, new DraftPickMessage { Decision = 0, Take = true, Card = 0 }));
        clock.Advance(TimeSpan.FromSeconds(10));
        pod.Tick(clock.GetUtcNow());
        var view = (DraftMessage)pod.ViewFor(1)!;
        Assert.Single(view.Pile);
        Assert.False(view.PickDue);
        Assert.Equal(BoosterDraft.LastCardSeconds * 1000, view.PickLeftMs);

        // Its clock runs out and the card is taken, without asking the player who is there.
        clock.Advance(TimeSpan.FromSeconds(BoosterDraft.LastCardSeconds));
        pod.Tick(clock.GetUtcNow());
        view = (DraftMessage)pod.ViewFor(1)!;
        Assert.Equal(2, view.Pool.Count);
        Assert.True(pod.Picks.Single(pick => pick.Seat == 1 && pick.Pick == 2).Auto);
    }

    [Fact]
    public void Decks_not_submitted_in_time_are_made_up_with_basic_lands_and_the_first_round_is_paired_across_the_table()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 5);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);

        // Nobody picks: the clock drafts for everyone.
        while (pod.Stage == EventStage.Drafting)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        var building = (DeckBuildingMessage)pod.ViewFor(3)!;
        Assert.Equal(15, building.Pool.Count);
        Assert.Equal(600_000, building.LeftMs);

        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());

        Assert.Equal(EventStage.Playing, pod.Stage);
        var pairings = (PairingsMessage)pod.ViewFor(3)!;
        Assert.Equal((1, 3, 15_000), (pairings.Round, pairings.Rounds, pairings.StartsInMs));
        Assert.Equal([(1, 5), (2, 6), (3, 7), (4, 8)], pairings.Matches.Select(match => (match.FirstSeat, match.SecondSeat)));

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.All(record.Participants, participant =>
        {
            Assert.True(participant.Deck.Submitted);
            Assert.Equal(40, participant.Deck.Main.Count + participant.Deck.Basics.Values.Sum());
        });
    }

    [Fact]
    public void A_player_who_does_not_come_to_their_match_forfeits_it_and_is_not_paired_again()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 5);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        pod.Hosted(new Uri("ws://localhost:8999/game"));
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        while (pod.Stage == EventStage.Drafting)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        clock.Advance(TimeSpan.FromSeconds(15));
        pod.Tick(clock.GetUtcNow());

        IReadOnlyList<MatchGame> tables = pod.TablesToOpen();
        Assert.Equal(4, tables.Count);
        foreach ((MatchGame game, int i) in tables.Select((game, i) => (game, i)))
        {
            game.Opened(new Uri($"ws://localhost:{9000 + i}/game"));
        }

        // Ann's opponent, Eve, never comes; everyone else sits down.
        MatchGame annsTable = tables.Single(game => game.NameAt(1) == "Ann");
        annsTable.Seated(1);
        annsTable.Connected(1, true);
        foreach (MatchGame other in tables.Where(game => game != annsTable))
        {
            other.Seated(1);
            other.Seated(2);
        }

        clock.Advance(TimeSpan.FromMinutes(3));
        foreach (MatchGame game in tables) game.Tick(clock.GetUtcNow());

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        MatchRecord forfeited = record.Matches.Single(match => match.Players.Contains("ann"));
        Assert.True(forfeited.Forfeit);
        Assert.Equal(["ann"], forfeited.Winners);
        Assert.True(record.Participant("eve")!.Dropped);
        Assert.Contains("Eve was not there", forfeited.Detail);

        // Ann is sent back to the pod's table once the result has been shown.
        clock.Advance(TimeSpan.FromSeconds(1));
        annsTable.Tick(clock.GetUtcNow());
        Assert.Contains(annsTable.TakeVacated(), vacated => vacated.Seat == 1 && vacated.Farewell is TableMoveMessage);
    }

    /// <summary>A pod drafted and built by the clock, with its first round paired and not yet begun.</summary>
    private (EventStore Store, CreatedEvent Created, BoosterPod Pod, Clock Clock) Paired()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 5);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        pod.Hosted(new Uri("ws://localhost:8999/game"));
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        while (pod.Stage == EventStage.Drafting)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        return (store, created, pod, clock);
    }

    [Fact]
    public void A_player_who_drops_before_the_round_begins_is_left_out_of_it_and_the_round_is_paired_again()
    {
        (EventStore store, CreatedEvent created, BoosterPod pod, Clock clock) = Paired();
        clock.Advance(TimeSpan.FromSeconds(5));

        // Eve, seat 5, was to play Ann.
        Assert.Null(pod.Receive(5, new DropMessage()));

        var pairings = (PairingsMessage)pod.ViewFor(1)!;
        Assert.Equal((1, 10_000), (pairings.Round, pairings.StartsInMs));
        Assert.Equal(4, pairings.Matches.Count);
        Assert.DoesNotContain(pairings.Matches, match => match.FirstSeat == 5 || match.SecondSeat == 5);
        Assert.Single(pairings.Matches, match => match.SecondSeat == 0);
        Assert.True(pairings.Standings.Single(standing => standing.Seat == 5).Dropped);
        Assert.Equal(5, pairings.Standings[^1].Seat);

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.True(record.Participant("eve")!.Dropped);
        Assert.Equal(4, record.Matches.Count);
        Assert.Equal("You have already dropped.", pod.Receive(5, new DropMessage()));

        // The round begins when it was going to, with a table for each match but the bye.
        clock.Advance(TimeSpan.FromSeconds(10));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(3, pod.TablesToOpen().Count);
    }

    [Fact]
    public void A_player_who_drops_during_their_match_concedes_it_and_the_game_being_played_is_ended_against_them()
    {
        (EventStore store, CreatedEvent created, BoosterPod pod, Clock clock) = Paired();
        clock.Advance(TimeSpan.FromSeconds(15));
        pod.Tick(clock.GetUtcNow());
        IReadOnlyList<MatchGame> tables = pod.TablesToOpen();
        foreach ((MatchGame game, int i) in tables.Select((game, i) => (game, i)))
        {
            game.Opened(new Uri($"ws://localhost:{9000 + i}/game"));
            game.Seated(1);
            game.Seated(2);
        }

        // Ann and Eve's first game is dealt; Eve drops from the site, mid-game.
        MatchGame annsTable = tables.Single(game => game.NameAt(1) == "Ann");
        Assert.True(annsTable.ReadyToDeal);
        annsTable.Deal(["Ann", "Eve"], new CapturingRulesFactory());
        Assert.Equal("You are not in this event.", pod.Drop("zed"));
        Assert.Null(pod.Drop("eve"));
        Assert.True(pod.HasDropped("eve"));

        // The table ends the game against her, saying why, and its own end changes nothing more.
        Assert.Equal([2], annsTable.TakeForfeits());
        Assert.Equal("Eve dropped from the event.", annsTable.ForfeitReason(2));
        Assert.Equal(TablePhase.Finished, annsTable.Phase);
        annsTable.GameEnded(false, [1], "Eve dropped from the event.");

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        MatchRecord match = record.Matches.Single(each => each.Players.Contains("eve"));
        Assert.True(match.Complete);
        Assert.Equal(["ann"], match.Winners);
        Assert.Equal("Ann won: Eve dropped.", match.Detail);
        Assert.Equal(("won", "ann"), (match.Games.Single().Ending, match.Games.Single().Winners.Single()));
        Assert.True(record.Participant("eve")!.Dropped);
        var pairings = (PairingsMessage)pod.ViewFor(1)!;
        Assert.True(pairings.Standings.Single(standing => standing.Seat == 5).Dropped);
        Assert.Equal(5, pairings.Standings[^1].Seat);

        // Ann goes back to the pairings once the result has been shown.
        clock.Advance(TimeSpan.FromSeconds(10));
        annsTable.Tick(clock.GetUtcNow());
        Assert.Contains(annsTable.TakeVacated(), vacated => vacated.Seat == 1 && vacated.Farewell is TableMoveMessage);

        // The others play on; once they are done the next round is paired without Eve, and the odd player out has the bye.
        foreach (MatchGame other in tables.Where(game => game != annsTable))
        {
            other.Deal(["A", "B"], new CapturingRulesFactory());
            other.GameEnded(false, [1], "won");
        }

        foreach (MatchGame next in pod.TablesToOpen())
        {
            next.Opened(new Uri("ws://localhost:9100/game"));
            next.Deal(["A", "B"], new CapturingRulesFactory());
            next.GameEnded(false, [1], "won");
        }

        record = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(2, record.Round);
        List<MatchRecord> second = [.. record.Matches.Where(each => each.Round == 2)];
        Assert.DoesNotContain(second, each => each.Players.Contains("eve"));
        Assert.Single(second, each => each.Bye);
        Assert.Equal(7, second.Sum(each => each.Players.Count));
    }

    [Fact]
    public void A_player_who_drops_during_the_draft_is_picked_for_at_once_their_deck_submitted_and_never_paired()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store, packSize: 5);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, Content, clock);
        for (int seat = 1; seat <= 8; seat++) pod.Seated(seat);
        Assert.Equal(EventStage.Drafting, pod.Stage);

        // Cat drops: her first pick is made for her straight away, without her clock running down.
        Assert.Null(pod.Drop("cat"));
        Assert.Single(pod.Picks, pick => pick.Seat == 3 && pick.Auto);
        Assert.DoesNotContain(pod.Picks, pick => pick.Seat != 3);

        // Every pack that comes to her is picked from as it arrives; the rest are drafted by the clock.
        while (pod.Stage == EventStage.Drafting)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(15, record.Participant("cat")!.Pool.Count);

        // Dan drops while decks are built: his is submitted as it stands, and nobody is paired with either.
        Assert.Equal(EventStage.DeckBuilding, pod.Stage);
        Assert.Null(pod.Drop("dan"));
        record = store.LoadEvent(created.Record.Id)!;
        Assert.True(record.Participant("dan")!.Deck.Submitted);
        Assert.True(record.Participant("cat")!.Deck.Submitted);
        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());

        var pairings = (PairingsMessage)pod.ViewFor(1)!;
        Assert.Equal(3, pairings.Matches.Count);
        Assert.DoesNotContain(pairings.Matches, match => match.FirstSeat is 3 or 4 || match.SecondSeat is 3 or 4);
        Assert.Equal([3, 4], pairings.Standings.TakeLast(2).Select(standing => standing.Seat).Order());
    }

    [Fact]
    public void When_drops_leave_one_player_the_event_ends()
    {
        (_, _, BoosterPod pod, _) = Paired();
        for (int seat = 2; seat <= 8; seat++)
        {
            Assert.Null(pod.Receive(seat, new DropMessage()));
        }

        var standings = (PairingsMessage)pod.ViewFor(1)!;
        Assert.True(standings.Final);
        Assert.Equal(EventStage.Complete, pod.Stage);
        Assert.Equal(1, standings.Standings[0].Seat);
        Assert.Contains("over", pod.Receive(1, new DropMessage()));
    }
}
