using Cantrip.Protocol;
using Cantrip.Server.Events;
using Cantrip.Server.Rules;
using Cantrip.TestKit;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// A draft event as its table drives it, without the table: made and saved in a data directory, seated by its tickets,
/// drafted, built and dealt, with everything saved as it happens so the event can be opened again anywhere it stopped.
/// The cube is the RW Aggro list, every card of which is loaded.
/// </summary>
public sealed class DraftEventTests : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-events-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private static Cantrip.Datasets.ContentLibrary Content => ContentCache.Get(RepoRoot.ContentRoot);

    private CreatedEvent Create(EventStore store, int cards = 20, IReadOnlyList<string>? cube = null, IReadOnlyList<string>? pickOrder = null) =>
        DraftPod.CreateWinston(store, Content, new WinstonDraftSetup
        {
            Name = "Thursday draft",
            Cube = "rw-aggro",
            CubeCards = cube ?? CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
            PickOrder = pickOrder ?? [],
            Cards = cards,
            MinimumDeck = 15,
            Seed = 11,
            Players = [("alice", "Alice"), ("bob", "Bob")],
        });

    /// <summary>Plays the draft out: each seat takes whatever it is offered.</summary>
    private static void DraftOut(DraftPod pod)
    {
        while (pod.ViewFor(1) is DraftMessage draft)
        {
            Assert.Null(pod.Receive(draft.Active, new DraftPickMessage { Decision = draft.Decision, Take = true }));
        }
    }

    [Fact]
    public void A_new_event_is_saved_with_its_players_who_each_have_it_on_their_record()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store);

        EventRecord saved = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(EventKinds.WinstonDraft, saved.Kind);
        Assert.Equal(EventStage.Seating, saved.Stage);
        Assert.Equal(20, saved.Draft!.Cards);
        Assert.Equal(["alice", "bob"], saved.Participants.Select(participant => participant.PlayerId));
        Assert.NotNull(store.LoadDraft<WinstonDraft>(created.Record.Id));
        Assert.Contains(store.LoadPlayer("alice")!.Events, entry => entry.EventId == created.Record.Id);
        Assert.Equal([created.Record.Id], store.LiveEvents().Select(record => record.Id));
    }

    [Fact]
    public void A_card_banned_in_the_format_is_left_out_of_the_cube_and_not_counted_as_missing()
    {
        var store = new EventStore(_data);
        IReadOnlyList<string> aggro = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt"));
        List<string> cube = [.. aggro, "Cogwork Librarian"];
        Assert.True(Banlist.IsBanned(DraftFormats.Winston, "Cogwork Librarian"));
        Assert.True(Banlist.IsBanned(DraftFormats.Grid, "cogwork librarian"));
        Assert.False(Banlist.IsBanned(DraftFormats.Booster, "Cogwork Librarian"));
        Assert.Equal(aggro.Count, DraftPod.Playable(Content, cube, DraftFormats.Winston));
        Assert.Equal(aggro.Count + 1, DraftPod.Playable(Content, cube, DraftFormats.Booster));

        CreatedEvent created = Create(store, cards: 0, cube: cube);
        Assert.Empty(created.Missing);
        Assert.Equal(aggro.Count, created.Record.Draft!.Cards);
        WinstonDraft draft = store.LoadDraft<WinstonDraft>(created.Record.Id)!;
        Assert.DoesNotContain("Cogwork Librarian", draft.Stack.Concat(draft.Piles.SelectMany(pile => pile)));
    }

    [Fact]
    public void Only_a_hash_of_each_ticket_is_kept_and_the_ticket_seats_its_player()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content);

        string saved = File.ReadAllText(Path.Combine(_data, "events", created.Record.Id, "event.json"));
        Assert.DoesNotContain(created.Tickets["alice"], saved, StringComparison.OrdinalIgnoreCase);
        Assert.Equal(1, pod.SeatFor(created.Tickets["alice"]));
        Assert.Equal(2, pod.SeatFor(created.Tickets["bob"]));
        Assert.Equal(0, pod.SeatFor(new string('0', 64)));

        // A fresh ticket for a lost link works, and so does the old one.
        string again = pod.IssueTicket("alice");
        Assert.Equal(1, pod.SeatFor(again));
        Assert.Equal(1, pod.SeatFor(created.Tickets["alice"]));
    }

    [Fact]
    public void A_cube_card_that_is_not_loaded_is_left_out_and_said_so()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store, cards: 3, cube: ["Lightning Bolt", "Chain Lightning", "Not A Card Anyone Printed", "Dismember"]);

        Assert.Equal(["Not A Card Anyone Printed"], created.Missing);
        Assert.Equal(3, created.Record.Draft!.Cards);
    }

    [Fact]
    public void The_draft_starts_when_everyone_has_sat_down_and_shows_the_pile_only_to_the_seat_looking_at_it()
    {
        var store = new EventStore(_data);
        DraftPod pod = DraftPod.Open(store, Create(store).Record.Id, Content);

        pod.Seated(1);
        Assert.Equal(TablePhase.Setup, pod.Phase);
        Assert.Null(pod.ViewFor(1));
        pod.Seated(2);
        Assert.Equal(TablePhase.Drafting, pod.Phase);

        var draft = (DraftMessage)pod.ViewFor(1)!;
        int looking = draft.Active;
        int other = 3 - looking;
        Assert.Single(((DraftMessage)pod.ViewFor(looking)!).Pile);
        Assert.Empty(((DraftMessage)pod.ViewFor(other)!).Pile);
        Assert.Equal([1, 1, 1], draft.Piles);
        Assert.Equal(17, draft.Stack);
        Assert.NotNull(pod.Receive(other, new DraftPickMessage { Decision = draft.Decision, Take = true }));
    }

    [Fact]
    public void A_winston_draft_ranks_by_the_pick_order_only_the_pile_its_seat_is_looking_at()
    {
        var store = new EventStore(_data);
        // The cube's cards worst first, and a name no card has, which is left out.
        List<string> order = ["Not A Card Anyone Printed", .. CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")).Reverse().Distinct()];
        string id = Create(store, pickOrder: order).Record.Id;
        DraftPod pod = DraftPod.Open(store, id, Content);
        pod.Seated(1);
        pod.Seated(2);

        // A card's rank is how many of the draft's own cards the order puts ahead of it.
        WinstonDraft saved = store.LoadDraft<WinstonDraft>(id)!;
        Assert.Equal(saved.PickOrder.Count, saved.PickOrder.Distinct().Count());
        HashSet<string> dealt = [.. saved.Stack.Concat(saved.Piles.SelectMany(pile => pile)).Select(card => Content.Cards.InOrder().First(known => known.Id.Value == card).Name)];
        int Expected(string name) => order.Take(order.IndexOf(name)).Count(dealt.Contains);

        var draft = (DraftMessage)pod.ViewFor(1)!;
        int looking = draft.Active;
        int other = 3 - looking;
        for (int pass = 0; pass < 3; pass++)
        {
            var mine = (DraftMessage)pod.ViewFor(looking)!;
            var theirs = (DraftMessage)pod.ViewFor(other)!;
            Assert.NotEmpty(mine.Pile);
            Assert.Equal(mine.Pile.Select(card => Expected(card.Name)), mine.PileRanks);
            // Nothing the other seat may not see is ranked for it: not the pile, the other piles or the stack.
            Assert.Empty(theirs.Pile);
            Assert.Empty(theirs.PileRanks);
            Assert.Null(pod.Receive(looking, new DraftPickMessage { Decision = mine.Decision, Take = false }));
            if (((DraftMessage)pod.ViewFor(1)!).Active != looking)
            {
                break;
            }
        }
    }

    [Fact]
    public void A_winston_draft_of_a_cube_with_no_pick_order_ranks_nothing()
    {
        var store = new EventStore(_data);
        DraftPod pod = DraftPod.Open(store, Create(store).Record.Id, Content);
        pod.Seated(1);
        pod.Seated(2);

        var draft = (DraftMessage)pod.ViewFor(1)!;
        var mine = (DraftMessage)pod.ViewFor(draft.Active)!;
        Assert.NotEmpty(mine.Pile);
        Assert.Empty(mine.PileRanks);
    }

    [Fact]
    public void A_draft_picks_up_where_it_was_when_the_event_is_opened_again()
    {
        var store = new EventStore(_data);
        string id = Create(store).Record.Id;
        DraftPod pod = DraftPod.Open(store, id, Content);
        pod.Seated(1);
        pod.Seated(2);
        for (int i = 0; i < 5; i++)
        {
            var draft = (DraftMessage)pod.ViewFor(1)!;
            Assert.Null(pod.Receive(draft.Active, new DraftPickMessage { Decision = draft.Decision, Take = i % 2 == 0 }));
        }

        var before = (DraftMessage)pod.ViewFor(1)!;
        DraftPod reopened = DraftPod.Open(store, id, Content);
        var after = (DraftMessage)reopened.ViewFor(1)!;

        Assert.Equal(before.Decision, after.Decision);
        Assert.Equal(before.Active, after.Active);
        Assert.Equal(before.Piles, after.Piles);
        Assert.Equal(before.Pool.Select(card => card.CardId), after.Pool.Select(card => card.CardId));
    }

    [Fact]
    public void After_the_draft_each_deck_starts_as_the_whole_pool_and_is_checked_as_it_is_built()
    {
        var store = new EventStore(_data);
        DraftPod pod = DraftPod.Open(store, Create(store).Record.Id, Content);
        pod.Seated(1);
        pod.Seated(2);
        DraftOut(pod);

        Assert.Equal(TablePhase.DeckBuilding, pod.Phase);
        var building = (DeckBuildingMessage)pod.ViewFor(1)!;
        Assert.Equal(Enumerable.Range(0, building.Pool.Count), building.Main);
        Assert.Equal(DraftPod.BasicLands, building.Basics.Select(land => land.Name));

        int change = 0;
        BuildDeckMessage Deck(List<int> main, int mountains, bool submit) => new()
        {
            Change = ++change,
            Main = main,
            Basics = [new BasicLandCount { Name = "Mountain", Count = mountains }],
            Submit = submit,
        };

        List<int> all = [.. building.Main];
        Assert.NotNull(pod.Receive(1, Deck([building.Pool.Count], 0, submit: false)));
        Assert.NotNull(pod.Receive(1, Deck([0, 0], 0, submit: false)));
        Assert.NotNull(pod.Receive(1, new BuildDeckMessage { Change = ++change, Basics = [new BasicLandCount { Name = "Wastes", Count = 1 }] }));
        Assert.NotNull(pod.Receive(1, Deck(all, 15 - all.Count - 1, submit: true)));

        // A refused change still comes back, so the client knows to show the deck as the server has it.
        Assert.Equal(change, ((DeckBuildingMessage)pod.ViewFor(1)!).Change);
        Assert.Equal(all, ((DeckBuildingMessage)pod.ViewFor(1)!).Main);

        Assert.Null(pod.Receive(1, Deck(all[1..], 20, submit: false)));
        Assert.Null(pod.Receive(1, Deck(all[1..], 20, submit: true)));

        var built = (DeckBuildingMessage)pod.ViewFor(1)!;
        Assert.Equal(change, built.Change);
        Assert.Equal(0, ((DeckBuildingMessage)pod.ViewFor(2)!).Change);
        Assert.True(built.Submitted);
        Assert.Equal(all[1..], built.Main);
        Assert.Equal(20, built.Basics.Single(land => land.Name == "Mountain").Count);
        Assert.False(pod.ReadyToDeal);
    }

    [Fact]
    public void Everyone_submitted_deals_the_decks_built_and_the_result_is_kept()
    {
        var store = new EventStore(_data);
        string id = Create(store).Record.Id;
        DraftPod pod = DraftPod.Open(store, id, Content);
        pod.Seated(1);
        pod.Seated(2);
        DraftOut(pod);

        foreach (int seat in (int[])[1, 2])
        {
            var building = (DeckBuildingMessage)pod.ViewFor(seat)!;
            Assert.Null(pod.Receive(seat, new BuildDeckMessage
            {
                Change = 1,
                Main = building.Main[1..],
                Basics = [new BasicLandCount { Name = "Plains", Count = 8 }, new BasicLandCount { Name = "Mountain", Count = 7 }],
                Submit = true,
            }));
        }

        Assert.True(pod.ReadyToDeal);
        var dealt = new CapturingRulesFactory();
        _ = pod.Deal(["Alice", "Bob"], dealt);
        EventRecord playing = store.LoadEvent(id)!;
        Assert.Equal(EventStage.Playing, playing.Stage);
        Assert.Equal(1, playing.Round);

        foreach (EventParticipant participant in playing.Participants)
        {
            RulesSeat seat = dealt.Seats[participant.Seat - 1];
            Assert.Equal(participant.Pool.Count - 1 + 15 + 1, seat.Deck.Count + seat.Sideboard.Count);
        }

        // Cut off by a restart, the round is dealt again from its seed.
        DraftPod reopened = DraftPod.Open(store, id, Content);
        Assert.True(reopened.ReadyToDeal);
        Assert.Equal(playing.Matches[0].Games[0].Seed, store.LoadEvent(id)!.Matches[0].Games[0].Seed);

        pod.GameEnded(abandoned: false, [2], string.Empty);
        EventRecord over = store.LoadEvent(id)!;
        Assert.Equal(EventStage.Complete, over.Stage);
        Assert.Equal(["bob"], over.Matches[0].Winners);
        Assert.Equal("won", over.Matches[0].Games[0].Ending);
        Assert.Empty(store.LiveEvents());
    }

    [Fact]
    public void A_pick_saved_to_the_draft_but_not_yet_to_the_event_is_taken_up_when_the_event_is_opened_again()
    {
        var store = new EventStore(_data);
        string id = Create(store).Record.Id;
        DraftPod pod = DraftPod.Open(store, id, Content);
        pod.Seated(1);
        pod.Seated(2);
        string eventFile = Path.Combine(_data, "events", id, "event.json");
        string before = File.ReadAllText(eventFile);

        var draft = (DraftMessage)pod.ViewFor(1)!;
        Assert.Null(pod.Receive(draft.Active, new DraftPickMessage { Decision = draft.Decision, Take = true }));
        List<string> taken = [.. ((DraftMessage)pod.ViewFor(draft.Active)!).Pool.Select(card => card.CardId)];

        // As if the process stopped between the draft's save and the event's.
        File.WriteAllText(eventFile, before);
        DraftPod reopened = DraftPod.Open(store, id, Content);

        Assert.Equal(taken, store.LoadEvent(id)!.AtSeat(draft.Active)!.Pool);
        Assert.Equal(taken, ((DraftMessage)reopened.ViewFor(draft.Active)!).Pool.Select(card => card.CardId));
    }

    [Fact]
    public void A_game_that_cannot_be_dealt_sends_everyone_back_to_building_with_nothing_recorded()
    {
        var store = new EventStore(_data);
        string id = Create(store).Record.Id;
        DraftPod pod = DraftPod.Open(store, id, Content);
        pod.Seated(1);
        pod.Seated(2);
        DraftOut(pod);
        foreach (int seat in (int[])[1, 2])
        {
            var building = (DeckBuildingMessage)pod.ViewFor(seat)!;
            Assert.Null(pod.Receive(seat, new BuildDeckMessage
            {
                Change = 1, Main = building.Main, Basics = [new BasicLandCount { Name = "Plains", Count = 30 }], Submit = true,
            }));
        }

        // A card the pool names that is no longer loaded: the content changed under a saved event.
        EventRecord saved = store.LoadEvent(id)!;
        saved.Participants[0].Pool[0] = "mtg/no-such-card";
        store.SaveEvent(saved);
        DraftPod reopened = DraftPod.Open(store, id, Content);

        Assert.True(reopened.ReadyToDeal);
        Assert.ThrowsAny<Exception>(() => reopened.Deal(["Alice", "Bob"], new CapturingRulesFactory(card => !card.Contains("no-such-card", StringComparison.Ordinal))));
        reopened.DealFailed("a card is missing");

        EventRecord after = store.LoadEvent(id)!;
        Assert.Equal(EventStage.DeckBuilding, after.Stage);
        Assert.Equal(0, after.Round);
        Assert.Empty(after.Matches);
        Assert.All(after.Participants, participant => Assert.False(participant.Deck.Submitted));
        Assert.False(reopened.ReadyToDeal);
    }

    private CreatedEvent CreateGrid(EventStore store, int packs = 2) =>
        DraftPod.CreateGrid(store, Content, new GridDraftSetup
        {
            Name = "Grid night",
            Cube = "rw-aggro",
            CubeCards = CubeList.Load(Path.Combine(RepoRoot.Path, "decks", "rw-aggro.txt")),
            PickOrder = ["Lightning Bolt"],
            Packs = packs,
            MinimumDeck = 5,
            Seed = 11,
            Players = [("alice", "Alice"), ("bob", "Bob")],
        });

    [Fact]
    public void A_grid_draft_shows_both_seats_the_grid_and_takes_a_row_or_a_column_each()
    {
        var store = new EventStore(_data);
        CreatedEvent created = CreateGrid(store);
        Assert.Equal(EventKinds.GridDraft, created.Record.Kind);
        Assert.Equal(18, created.Record.Draft!.Cards);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content);
        pod.Seated(1);
        pod.Seated(2);

        var first = (DraftMessage)pod.ViewFor(1)!;
        int picking = first.Active;
        int other = 3 - picking;
        Assert.Equal(DraftFormats.Grid, first.Format);
        Assert.Equal((1, 2, 1), (first.PackNumber, first.Packs, first.PickNumber));
        Assert.Equal(9, first.Pile.Count);
        Assert.Equal(first.Pile.Select(card => card.CardId), ((DraftMessage)pod.ViewFor(other)!).Pile.Select(card => card.CardId));
        Assert.Equal(9, first.PileRanks.Count);

        Assert.NotNull(pod.Receive(other, new DraftPickMessage { Decision = first.Decision, Take = true, Card = 0 }));
        Assert.NotNull(pod.Receive(picking, new DraftPickMessage { Decision = first.Decision, Take = false, Card = 6 }));
        Assert.Null(pod.Receive(picking, new DraftPickMessage { Decision = first.Decision, Take = true, Card = 1 }));

        var second = (DraftMessage)pod.ViewFor(other)!;
        Assert.Equal(other, second.Active);
        Assert.Equal(2, second.PickNumber);
        Assert.Equal(-1, first.LastLine);
        Assert.Equal(1, second.LastLine);
        Assert.Equal([false, false, false], second.Pile.Skip(3).Take(3).Select(card => card.Known));
        Assert.Equal(first.Pile.Skip(3).Take(3).Select(card => card.CardId), ((DraftMessage)pod.ViewFor(picking)!).Pool.Select(card => card.CardId));
        Assert.Contains("took row 2 of grid 1", second.Recent.Single(), StringComparison.Ordinal);
        Assert.Equal(first.Pile.Skip(3).Take(3).Select(card => $"[{card.Name}]"),
            second.RecentCards.Single().Select(named => $"[{named.Card?.Name}]"));
        Assert.All(second.RecentCards.Single(), named => Assert.Contains($"[{named.Name}]", second.Recent.Single(), StringComparison.Ordinal));

        // Column 1 has two cards left; the second grid is the other seat's to pick from first.
        Assert.Null(pod.Receive(other, new DraftPickMessage { Decision = second.Decision, Take = true, Card = 3 }));
        var next = (DraftMessage)pod.ViewFor(1)!;
        Assert.Equal((2, 1, other), (next.PackNumber, next.PickNumber, next.Active));
        Assert.Equal(2, next.Seats.Single(seat => seat.Seat == other).Drafted);

        // The draft goes on after a restart from the same pick.
        DraftPod reopened = DraftPod.Open(store, created.Record.Id, Content);
        var resumed = (DraftMessage)reopened.ViewFor(1)!;
        Assert.Equal(next.Decision, resumed.Decision);
        Assert.Equal(next.Pile.Select(card => card.CardId), resumed.Pile.Select(card => card.CardId));

        Assert.Null(reopened.Receive(other, new DraftPickMessage { Decision = next.Decision, Take = true, Card = 2 }));
        var last = (DraftMessage)reopened.ViewFor(1)!;
        Assert.Null(reopened.Receive(picking, new DraftPickMessage { Decision = last.Decision, Take = true, Card = 5 }));

        Assert.Equal(TablePhase.DeckBuilding, reopened.Phase);
        EventRecord built = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(5, built.AtSeat(picking)!.Pool.Count);
        Assert.Equal(5, built.AtSeat(other)!.Pool.Count);
    }

    private sealed class Clock : TimeProvider
    {
        private DateTimeOffset _now = new(2026, 10, 1, 18, 0, 0, TimeSpan.Zero);
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan by) => _now += by;
    }

    [Fact]
    public void A_grid_pick_whose_clock_runs_out_is_taken_for_its_player_and_the_next_pick_gets_a_full_clock()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = CreateGrid(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { GridPick = TimeSpan.FromSeconds(30) });
        Assert.Equal(0, ((DraftMessage?)pod.ViewFor(1))?.PickLeftMs ?? 0);
        pod.Seated(1);
        pod.Seated(2);

        var first = (DraftMessage)pod.ViewFor(1)!;
        int picking = first.Active;
        Assert.Equal(30_000, first.PickLeftMs);
        Assert.Equal(30_000, ((DraftMessage)pod.ViewFor(3 - picking)!).PickLeftMs);

        clock.Advance(TimeSpan.FromSeconds(29));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(0, pod.Version);
        Assert.Equal(1_000, ((DraftMessage)pod.ViewFor(1)!).PickLeftMs);

        clock.Advance(TimeSpan.FromSeconds(1));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(1, pod.Version);
        var after = (DraftMessage)pod.ViewFor(picking)!;
        Assert.Equal((first.Decision + 1, 3 - picking), (after.Decision, after.Active));
        Assert.Equal(30_000, after.PickLeftMs);
        Assert.Contains("(time ran out)", after.Recent.Single(), StringComparison.Ordinal);
        Assert.Equal(3, store.LoadEvent(created.Record.Id)!.AtSeat(picking)!.Pool.Count);

        // A pick made in time starts the next clock from when it was made.
        clock.Advance(TimeSpan.FromSeconds(10));
        Assert.Null(pod.Receive(3 - picking, new DraftPickMessage { Decision = after.Decision, Take = true, Card = after.LastLine == 0 ? 1 : 0 }));
        Assert.Equal(30_000, ((DraftMessage)pod.ViewFor(1)!).PickLeftMs);
    }

    [Fact]
    public void A_grid_pick_whose_clock_runs_out_takes_the_line_its_player_had_chosen()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = CreateGrid(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { GridPick = TimeSpan.FromSeconds(30) });
        pod.Seated(1);
        pod.Seated(2);

        var first = (DraftMessage)pod.ViewFor(1)!;
        int picking = first.Active;

        // Only the active player's choice counts, and only for the pick it names.
        Assert.NotNull(pod.Receive(3 - picking, new DraftPickMessage { Decision = first.Decision, Take = false, Card = 5 }));
        Assert.NotNull(pod.Receive(picking, new DraftPickMessage { Decision = first.Decision + 1, Take = false, Card = 5 }));
        Assert.Null(pod.Receive(picking, new DraftPickMessage { Decision = first.Decision, Take = false, Card = 5 }));
        Assert.Equal(0, pod.Version);

        clock.Advance(TimeSpan.FromSeconds(30));
        pod.Tick(clock.GetUtcNow());
        var after = (DraftMessage)pod.ViewFor(picking)!;
        Assert.Equal(5, after.LastLine);
        Assert.Equal(first.Decision + 1, after.Decision);
    }

    [Fact]
    public void A_winston_turn_whose_clock_runs_out_is_played_out_for_its_player()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { WinstonTurn = TimeSpan.FromSeconds(60) });
        pod.Seated(1);
        pod.Seated(2);

        var first = (DraftMessage)pod.ViewFor(1)!;
        int looking = first.Active;

        // Passing a pile is part of the turn: the clock does not start again for the next pile.
        Assert.Null(pod.Receive(looking, new DraftPickMessage { Decision = first.Decision, Take = false }));
        clock.Advance(TimeSpan.FromSeconds(59));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(looking, ((DraftMessage)pod.ViewFor(1)!).Active);

        clock.Advance(TimeSpan.FromSeconds(1));
        pod.Tick(clock.GetUtcNow());
        var after = (DraftMessage)pod.ViewFor(looking)!;
        Assert.Equal(3 - looking, after.Active);
        Assert.Equal(60_000, after.PickLeftMs);
        Assert.Contains(after.Recent, line => line.Contains("(time ran out)", StringComparison.Ordinal));
        Assert.NotEmpty(store.LoadEvent(created.Record.Id)!.AtSeat(looking)!.Pool);
    }

    [Fact]
    public void Players_who_do_not_take_their_seats_in_time_call_the_event_off()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { Seating = TimeSpan.FromMinutes(10) });
        pod.Seated(1);

        clock.Advance(TimeSpan.FromMinutes(9));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal(EventStage.Seating, pod.Stage);

        // Opened again after a restart, the seating clock is where it was, and goes on running.
        DraftPod reopened = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { Seating = TimeSpan.FromMinutes(10) });
        clock.Advance(TimeSpan.FromMinutes(1));
        reopened.Tick(clock.GetUtcNow());
        Assert.Equal(EventStage.Cancelled, reopened.Stage);
        Assert.Equal(TablePhase.Finished, reopened.Phase);
        Assert.Equal("Bob did not take their seat in time.", reopened.Detail);
        Assert.Equal(1, reopened.Version);
    }

    [Fact]
    public void Deck_buildings_clock_submits_every_deck_as_it_stands_and_the_game_is_dealt()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { DeckBuilding = TimeSpan.FromMinutes(10) });
        pod.Seated(1);
        pod.Seated(2);
        DraftOut(pod);
        Assert.Equal(600_000, ((DeckBuildingMessage)pod.ViewFor(1)!).LeftMs);

        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());

        Assert.True(pod.ReadyToDeal);
        EventRecord built = store.LoadEvent(created.Record.Id)!;
        Assert.All(built.Participants, participant => Assert.True(participant.Deck.Submitted));
        Assert.All(built.Participants, participant => Assert.True(participant.Deck.Main.Count + participant.Deck.Basics.Values.Sum() >= 15));
    }

    [Fact]
    public void Decks_being_built_in_an_event_saved_without_a_deck_building_clock_get_one_when_it_is_opened_again()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock);
        pod.Seated(1);
        pod.Seated(2);
        DraftOut(pod);
        EventRecord saved = store.LoadEvent(created.Record.Id)!;
        saved.Deadline = null;
        store.SaveEvent(saved);

        DraftPod reopened = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { DeckBuilding = TimeSpan.FromMinutes(10) });
        clock.Advance(TimeSpan.FromMinutes(10));
        reopened.Tick(clock.GetUtcNow());

        Assert.True(reopened.ReadyToDeal);
    }

    [Fact]
    public void The_game_has_a_match_clock_and_a_player_not_at_the_table_forfeits_it()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        var timing = new DraftTiming { NoShow = TimeSpan.FromMinutes(3), Away = TimeSpan.FromMinutes(5), MatchClock = TimeSpan.FromMinutes(25) };
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, timing);
        pod.Seated(1);
        pod.Seated(2);
        pod.Connected(1, true);
        DraftOut(pod);
        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        pod.Deal(["Alice", "Bob"], new CapturingRulesFactory());

        Assert.Equal(TimeSpan.FromMinutes(25), pod.ClockAtDeal(1));
        pod.ClockUsed(2, TimeSpan.FromMinutes(20));
        Assert.Equal(TimeSpan.FromMinutes(20), pod.ClockAtDeal(2));

        // Bob was not at the table as it was dealt: he has three minutes to come.
        clock.Advance(TimeSpan.FromMinutes(2));
        pod.Tick(clock.GetUtcNow());
        Assert.Empty(pod.TakeForfeits());
        clock.Advance(TimeSpan.FromMinutes(1));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal([2], pod.TakeForfeits());
        Assert.Empty(pod.TakeForfeits());

        pod.GameEnded(false, [1], "Bob was not at the table.");
        EventRecord ended = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(EventStage.Complete, ended.Stage);
        MatchRecord match = ended.Matches.Single();
        Assert.True(match.Forfeit);
        Assert.Equal(["alice"], match.Winners);
        Assert.Equal(20 * 60_000, match.ClockMs["bob"]);
    }

    [Fact]
    public void A_player_gone_from_the_game_too_long_forfeits_it()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock, new DraftTiming { Away = TimeSpan.FromMinutes(5) });
        pod.Seated(1);
        pod.Seated(2);
        pod.Connected(1, true);
        pod.Connected(2, true);
        DraftOut(pod);
        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        pod.Deal(["Alice", "Bob"], new CapturingRulesFactory());

        // Gone a while and back again: the time away starts over when she leaves again.
        pod.Connected(1, false);
        clock.Advance(TimeSpan.FromMinutes(4));
        pod.Connected(1, true);
        pod.Connected(1, false);
        clock.Advance(TimeSpan.FromMinutes(4));
        pod.Tick(clock.GetUtcNow());
        Assert.Empty(pod.TakeForfeits());

        clock.Advance(TimeSpan.FromMinutes(1));
        pod.Tick(clock.GetUtcNow());
        Assert.Equal([1], pod.TakeForfeits());
    }

    [Fact]
    public void A_player_who_drops_before_the_game_calls_the_draft_off_and_it_is_over_for_both()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock);
        pod.Seated(1);
        pod.Seated(2);
        Assert.Equal(EventStage.Drafting, pod.Stage);
        int version = pod.Version;

        Assert.Equal("You are not in this draft.", pod.Drop("carol"));
        Assert.Null(pod.Drop("bob"));

        // Over for both, and the table is told: it shows the event finished.
        Assert.Equal(EventStage.Cancelled, pod.Stage);
        Assert.Equal(TablePhase.Finished, pod.Phase);
        Assert.NotEqual(version, pod.Version);
        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.Equal("Bob dropped from the draft.", record.Detail);
        Assert.True(record.Participant("bob")!.Dropped);
        Assert.True(pod.HasDropped("bob"));
        Assert.Empty(store.LiveEvents());
        Assert.Contains("over", pod.Drop("alice"));

        // As each player's own list has it.
        PlayerEventSummary bob = Assert.Single(PlayerEvents.Of(store, "bob"));
        Assert.Equal(("Called off", false, true), (bob.Outcome, bob.Live, bob.Dropped));
        Assert.Equal("Called off", PlayerEvents.Of(store, "alice").Single().Outcome);
    }

    [Fact]
    public void A_player_who_drops_during_the_game_forfeits_it_and_the_other_wins_the_draft()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        DraftPod pod = DraftPod.Open(store, created.Record.Id, Content, clock);
        pod.Seated(1);
        pod.Seated(2);
        pod.Connected(1, true);
        pod.Connected(2, true);
        DraftOut(pod);
        clock.Advance(TimeSpan.FromMinutes(10));
        pod.Tick(clock.GetUtcNow());
        pod.Deal(["Alice", "Bob"], new CapturingRulesFactory());
        Assert.Equal("Playing", PlayerEvents.Of(store, "alice").Single().Outcome);

        // Alice drops: the table is to end the game against her, saying why, and records what it did.
        Assert.Null(pod.Drop("alice"));
        Assert.Equal([1], pod.TakeForfeits());
        Assert.Equal("Alice dropped from the draft.", pod.ForfeitReason(1));
        Assert.Null(pod.ForfeitReason(2));
        Assert.Equal("You have already dropped.", pod.Drop("alice"));
        pod.GameEnded(false, [2], "Alice dropped from the draft.");

        EventRecord record = store.LoadEvent(created.Record.Id)!;
        Assert.Equal(EventStage.Complete, record.Stage);
        MatchRecord match = record.Matches.Single();
        Assert.Equal(["bob"], match.Winners);
        Assert.True(match.Forfeit);
        Assert.Equal("Bob won round 1. Alice dropped from the draft.", record.Detail);

        PlayerEventSummary bob = PlayerEvents.Of(store, "bob").Single();
        Assert.Equal(("Won", 1, 0), (bob.Outcome, bob.Wins, bob.Losses));
        Assert.Equal(["Alice"], bob.Opponents.Select(opponent => opponent.Name));
        Assert.Equal("Dropped", PlayerEvents.Of(store, "alice").Single().Outcome);
    }

    [Fact]
    public void A_players_events_list_the_ones_being_played_first_then_the_latest_over()
    {
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent first = Create(store);
        DraftPod.Open(store, first.Record.Id, Content, clock).Cancel("Called off by the host.");
        CreatedEvent second = Create(store);

        IReadOnlyList<PlayerEventSummary> alice = PlayerEvents.Of(store, "alice");
        Assert.Equal([second.Record.Id, first.Record.Id], alice.Select(each => each.Id));
        Assert.Equal(("Taking seats", true), (alice[0].Outcome, alice[0].Live));
        Assert.Equal("winston", alice[0].Format);
        Assert.Equal("rw-aggro", alice[0].Cube);

        // One the host no longer runs is not being played; and only as many over as asked for.
        PlayerEventSummary stopped = PlayerEvents.Of(store, "alice", hosted: _ => false).First(each => each.Id == second.Record.Id);
        Assert.Equal(("Not finished", false), (stopped.Outcome, stopped.Live));
        Assert.Equal([second.Record.Id], PlayerEvents.Of(store, "alice", recent: 0).Select(each => each.Id));
        Assert.Empty(PlayerEvents.Of(store, "nobody"));
        Assert.Equal(["1st", "2nd", "3rd", "4th", "11th", "12th", "13th", "21st", "102nd"], new[] { 1, 2, 3, 4, 11, 12, 13, 21, 102 }.Select(PlayerEvents.Ordinal));
    }

    [Fact]
    public void An_id_that_could_reach_outside_the_data_directory_is_refused()
    {
        var store = new EventStore(_data);

        Assert.False(EventStore.ValidId("../escape"));
        Assert.False(EventStore.ValidId("a/b"));
        Assert.False(EventStore.ValidId("Upper"));
        Assert.False(EventStore.ValidId("abc."));
        Assert.False(EventStore.ValidId("con"));
        Assert.False(EventStore.ValidId("nul.json"));
        Assert.True(EventStore.ValidId(EventStore.ToId("CON")));
        Assert.Throws<ArgumentException>(() => store.LoadEvent("..\\escape"));
        Assert.Equal("discord-1234", EventStore.ToId("Discord 1234"));
    }
}
