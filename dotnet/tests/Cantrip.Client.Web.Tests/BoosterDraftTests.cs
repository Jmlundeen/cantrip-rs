using Cantrip.Protocol;
using Cantrip.Server;
using Cantrip.Server.Events;
using Cantrip.Server.Host;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// A booster draft of a pod of eight in the page, the other seven seats bots that follow their seats from table to
/// table: the table seen from above with everyone's packs waiting in front of them, a card double-clicked out of the
/// pack, one dragged from it into a column of the pool, deck building on its clock, and the pairings the round begins
/// from. Another drafts a Cogwork Librarian and puts it into a pack for two cards. A second pod is played out by bots alone, and the page opened on its final standings. Pictured at each step,
/// under TestResults\web.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class BoosterDraftTests(WebClient web) : IDisposable
{
    private readonly string _data = Path.Combine(Path.GetTempPath(), "cantrip-web-booster-" + Guid.NewGuid().ToString("N"));

    public void Dispose()
    {
        if (Directory.Exists(_data))
        {
            Directory.Delete(_data, recursive: true);
        }
    }

    private static readonly IReadOnlyList<(string, string)> Pod =
        [("you", "You"), ("ben", "Ben"), ("cat", "Cat"), ("dan", "Dan"), ("eve", "Eve"), ("fay", "Fay"), ("gus", "Gus"), ("hal", "Hal")];

    /// <summary>
    /// The pod drafts LSVCube; a pod played out by bots drafts the RW Aggro list three times over, whose games they play
    /// cleanly - what is pictured is the standings, not every card of a cube.
    /// </summary>
    private CreatedEvent Create(EventStore store, bool playedOut = false) => BoosterPod.Create(store, ContentCache.Get(WebClient.Content), new BoosterDraftSetup
    {
        Name = "Friday pod",
        Cube = playedOut ? "RW Aggro x3" : "LSVCube",
        CubeCards = playedOut
            ? [.. Enumerable.Repeat(CubeList.Load(Path.Combine(WebClient.Root, "decks", "rw-aggro.txt")), 3).SelectMany(cards => cards)]
            : CubeList.Load(Path.Combine(WebClient.Root, "docs", "cube-lists", "LSVCube.txt")),
        Packs = 3,
        PackSize = 5,
        Rounds = 3,
        Seed = 5,
        Players = Pod,
    });

    /// <summary>Seats a bot in each of the pod's seats but the page's, each playing until the test is over.</summary>
    private static async Task<(List<EventSeat> Seats, Task Playing)> Bots(Uri address, CreatedEvent created, IEnumerable<string> who, CancellationToken stop)
    {
        List<EventSeat> seats = [];
        foreach (string id in who)
        {
            seats.Add(await EventSeat.JoinAsync(address, created.Tickets[id], Pod.First(player => player.Item1 == id).Item2));
        }

        Task playing = Task.WhenAll(seats.Select(seat => Task.Run(async () =>
        {
            while (!stop.IsCancellationRequested && !seat.Finished)
            {
                if (!await seat.StepAsync(CancellationToken.None))
                {
                    await Task.Delay(15, CancellationToken.None);
                }
            }
        })));
        return (seats, playing);
    }

    [Fact]
    public async Task A_pick_is_double_clicked_from_the_pack_and_the_round_is_paired_once_decks_are_in()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), timing: new BoosterTiming
        {
            SecondsPerCard = 4,
            FirstRound = TimeSpan.FromMinutes(5),
        });

        // Everyone's picture but Hal's, who keeps the stand-in.
        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(WebClient.Content, portraits: TestPortraits.AllBut("hal")));
        using var stop = new CancellationTokenSource();
        (List<EventSeat> bots, Task playing) = await Bots(host.Address, created, Pod.Skip(1).Select(player => player.Item1), stop.Token);

        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(host.Address.AbsoluteUri)}#ticket={created.Tickets["you"]}", WebClient.Pictures(nameof(BoosterDraftTests)));

        // The first pack: the table with a pack in front of everyone, and the pack's five cards to pick from.
        TableState state = await page.SettledAsync(state => state.Phase == "Drafting" && state.Controls.Any(control => control.Label == "Pick"));
        await page.PictureAsync("booster-first-pick");
        ControlState pick = state.Controls.Single(control => control.Label == "Pick");
        List<CardState> pack = [.. state.Cards.Where(card => card.Visible && card.Row == "pack")];
        Assert.Equal(5, pack.Count);
        Assert.True(pick.Y > pack.Max(card => card.Y + (card.H / 2)), "Pick is below the pack.");

        // A double-click takes the card: it flies into the pool, and the pack goes on to the next seat.
        int flown = state.Flown;
        CardState taken = pack[2];
        await page.Page.Mouse.DblClickAsync(taken.X, taken.Y);
        state = await page.SettledAsync(state => state.Flown > flown);
        await page.PictureAsync("booster-picked");
        string pickedId = pod.Picks.First(entry => entry.Seat == 1 && !entry.Auto).Card;
        Assert.Equal(taken.Name, ContentCache.Get(WebClient.Content).Cards.InOrder().First(card => card.Id.Value == pickedId).Name);

        // The table and its log of picks are put away until asked for.
        await page.ClickAsync("Show draft table", "button");
        await page.PictureAsync("booster-draft-table");

        // The card just taken, named in the log of picks, reads as the card does: its tooltip, and held up from its name.
        state = await page.SettledAsync(state => state.Mentions?.Any(mention => mention.Visible && mention.Name == taken.Name) == true);
        MentionState named = state.Mentions!.First(mention => mention.Visible && mention.Name == taken.Name);
        await page.RestAsync(named.X, named.Y);
        Assert.StartsWith(taken.Name, (await page.StateAsync())!.Tooltip);
        await page.PictureAsync("booster-pick-named-tooltip");
        await page.HoldUpAsync(named);
        await Task.Delay(400);
        Assert.Equal(1, (await page.StateAsync())!.Held);
        await page.PictureAsync("booster-pick-named-held-up");
        await page.PutDownAsync();
        await Task.Delay(400);
        await page.ClickAsync("Hide draft table", "button");

        // The next pick dragged out of the pack onto the card just taken: it is picked, and goes in that card's column.
        state = await page.SettledAsync(state => state.Cards.Count(card => card.Row == "pack") == 4);
        CardState first = state.Cards.Single(card => card.Row == "pool" && card.Name == taken.Name);
        CardState dragged = state.Cards.First(card => card.Row == "pack");
        // Let go anywhere but the pool, it is not picked: it flies back to its place in the pack.
        flown = state.Flown;
        await page.Page.Mouse.MoveAsync(dragged.X, dragged.Y);
        await page.Page.Mouse.DownAsync();
        await page.Page.Mouse.MoveAsync(dragged.X + 20, dragged.Y + 20, new() { Steps = 4 });
        await page.Page.Mouse.MoveAsync(dragged.X + 40, 12, new() { Steps = 8 });
        await page.Page.Mouse.UpAsync();
        state = await page.SettledAsync(state => state.Flown > flown);
        CardState back = state.Cards.Single(card => card.Row == "pack" && card.Name == dragged.Name);
        Assert.Equal((dragged.X, dragged.Y), (back.X, back.Y));
        Assert.True(back.FlewFrom is { } returned && MathF.Abs(returned.Y - 12) < 30, $"{dragged.Name} flew back from {back.FlewFrom}, not from where it was let go.");
        Assert.DoesNotContain(state.Cards, card => card.Row == "pool" && card.Name == dragged.Name);

        flown = state.Flown;
        int pooled = state.Cards.Count(card => card.Row == "pool");
        await page.Page.Mouse.MoveAsync(dragged.X, dragged.Y);
        await page.Page.Mouse.DownAsync();
        await page.Page.Mouse.MoveAsync(dragged.X + 20, dragged.Y + 20, new() { Steps = 4 });
        (float X, float Y) letGo = (first.X, first.Y + (first.H / 2) - 4);
        await page.Page.Mouse.MoveAsync(letGo.X, letGo.Y, new() { Steps = 12 });
        await page.Page.Mouse.UpAsync();

        // Until the server answers, it waits where it was let go: never back in the pack, to fly from there.
        DateTime answered = DateTime.UtcNow + TimeSpan.FromSeconds(10);
        while (true)
        {
            TableState? now = await page.StateAsync();
            if (now is not null && now.Cards.Count(card => card.Row == "pool") > pooled)
            {
                break;
            }

            Assert.False(now?.Cards.Any(card => card.Row == "pack" && card.Name == dragged.Name), $"{dragged.Name} went back to the pack while its pick was answered.");
            Assert.True(DateTime.UtcNow < answered, $"{dragged.Name} was not taken.");
        }

        state = await page.SettledAsync(state => state.Flown > flown && state.Cards.Count(card => card.Row == "pool") > pooled);
        await page.PictureAsync("booster-dragged-pick");
        CardState landed = state.Cards.Single(card => card.Row == "pool" && card.Name == dragged.Name);
        Assert.Equal(state.Cards.Single(card => card.Row == "pool" && card.Name == taken.Name).X, landed.X, 1f);

        // It flew from where it was let go, not from where it was in the pack.
        Assert.True(landed.FlewFrom is { } flewFrom && MathF.Abs(flewFrom.X - letGo.X) < 30 && MathF.Abs(flewFrom.Y - letGo.Y) < 30,
            $"The dragged pick flew from {landed.FlewFrom}, not from where it was let go, {letGo}; it was at {(dragged.X, dragged.Y)} in the pack.");

        // Every pick after that by clicking a card and then Pick, until the draft is over.
        DateTime giveUp = DateTime.UtcNow + TimeSpan.FromMinutes(3);
        while (state.Phase == "Drafting" && DateTime.UtcNow < giveUp)
        {
            if (state.Controls.Any(control => control.Label == "Pick")
                && state.Cards.FirstOrDefault(card => card.Visible && card.Row == "pack") is { } card)
            {
                await page.Page.Mouse.ClickAsync(card.X, card.Y);
                await Task.Delay(150);
                if ((await page.StateAsync())?.Controls.FirstOrDefault(control => control.Label.StartsWith("Pick ", StringComparison.Ordinal)) is { } confirm)
                {
                    await page.Page.Mouse.ClickAsync(confirm.X, confirm.Y);
                }
            }

            await Task.Delay(150);
            state = await page.StateAsync() ?? state;
        }

        // Deck building, on its clock: everything drafted is in the deck; seventeen lands and in.
        state = await page.SettledAsync(state => state.Phase == "DeckBuilding");
        await page.PictureAsync("booster-deck-building");

        // The card dragged into a column is still in it, beside the card it was put with, not in a column of its own cost.
        Assert.Equal(
            state.Cards.Single(card => card.Row == "pool" && card.Name == taken.Name).X,
            state.Cards.Single(card => card.Row == "pool" && card.Name == dragged.Name).X,
            1f);
        for (int i = 0; i < 25; i++)
        {
            await page.ClickAsync("Add Island", "button");
        }

        state = await page.SettledAsync(state => state.Controls.Any(control => control.Label == "Submit deck (40)"));
        await page.ClickAsync("Submit deck (40)", "button");

        // Every deck in: the first round's pairings, across the table, with the clock on the round.
        state = await page.SettledAsync(state => state.Phase == "Pairings" && state.Pairings is { Round: 1 });
        await page.PictureAsync("pairings-round-1");
        Assert.Equal(4, state.Pairings!.Matches.Count);
        Assert.StartsWith("1: You 0-0 Eve", state.Pairings.Matches[0], StringComparison.Ordinal);
        Assert.True(state.Pairings.StartsIn > 0);

        await stop.CancelAsync();
        await playing;
        foreach (EventSeat bot in bots) bot.Dispose();
        Assert.Empty(page.Errors);
    }

    [Fact]
    public async Task Cogwork_Librarian_in_the_pool_is_put_into_a_pack_for_two_cards_picked_together()
    {
        var store = new EventStore(_data);
        // Packs of five from a hundred and twenty cards, so every card is dealt - the Librarian among them - and the
        // position forced: the Librarian first in this seat's pack.
        CreatedEvent created = BoosterPod.Create(store, ContentCache.Get(WebClient.Content), new BoosterDraftSetup
        {
            Name = "Friday pod",
            Cube = "RW Aggro x3 and a Librarian",
            CubeCards = ["Cogwork Librarian", .. Enumerable.Repeat(CubeList.Load(Path.Combine(WebClient.Root, "decks", "rw-aggro.txt")), 3).SelectMany(cards => cards).Skip(1)],
            PackSize = 5,
            Seed = 5,
            Players = Pod,
        });
        BoosterDraft draft = store.LoadDraft<BoosterDraft>(created.Record.Id)!;
        List<string> holding = draft.Queues.Select(queue => queue[0].Cards).Concat(draft.Unopened.SelectMany(round => round))
            .First(cards => cards.Contains("Cogwork Librarian"));
        (holding[holding.IndexOf("Cogwork Librarian")], draft.Queues[0][0].Cards[0]) = (draft.Queues[0][0].Cards[0], "Cogwork Librarian");
        store.SaveDraft(created.Record.Id, draft);

        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), timing: new BoosterTiming { SecondsPerCard = 30 });
        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(WebClient.Content));
        using var stop = new CancellationTokenSource();
        (List<EventSeat> bots, Task playing) = await Bots(host.Address, created, Pod.Skip(1).Select(player => player.Item1), stop.Token);

        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(host.Address.AbsoluteUri)}#ticket={created.Tickets["you"]}", WebClient.Pictures(nameof(BoosterDraftTests)));

        // The Librarian taken first; nothing to spend it on yet.
        TableState state = await page.SettledAsync(state => state.Cards.Any(card => card.Visible && card.Row == "pack" && card.Name == "Cogwork Librarian"));
        Assert.DoesNotContain(state.Controls, control => control.Kind == "toggle" && control.Label.Contains("Cogwork Librarian", StringComparison.Ordinal));
        CardState librarian = state.Cards.Single(card => card.Visible && card.Row == "pack" && card.Name == "Cogwork Librarian");
        await page.Page.Mouse.DblClickAsync(librarian.X, librarian.Y);

        // The next pack, with the Librarian in the pool to put into it.
        state = await page.SettledAsync(state => state.Cards.Count(card => card.Visible && card.Row == "pack") == 4
            && state.Controls.Any(control => control.Kind == "toggle" && control.Label.Contains("Cogwork Librarian", StringComparison.Ordinal)));
        await page.PictureAsync("booster-librarian-offered");
        await page.ClickAsync("Use Cogwork Librarian for a second card", "toggle");

        // Two cards chosen, and both picked with one press.
        List<CardState> pack = [.. (await page.SettledAsync()).Cards.Where(card => card.Visible && card.Row == "pack")];
        await page.Page.Mouse.ClickAsync(pack[0].X, pack[0].Y);
        await Task.Delay(200);
        await page.Page.Mouse.ClickAsync(pack[2].X, pack[2].Y);
        await page.SettledAsync(state => state.Controls.Any(control => control.Label == "Pick 2 cards"));
        await page.PictureAsync("booster-librarian-two-chosen");
        await page.ClickAsync("Pick 2 cards", "button");

        // Both cards are in the pool and the Librarian is gone from it, into the pack. (The bots draft faster than this
        // seat, so the last cards of their packs come to it as well.)
        state = await page.SettledAsync(state => state.Cards.All(card => card.Row != "pool" || card.Name != "Cogwork Librarian"));
        await page.PictureAsync("booster-librarian-spent");
        List<string> pool = [.. state.Cards.Where(card => card.Row == "pool").Select(card => card.Name)];
        Assert.Contains(pack[0].Name, pool);
        Assert.Contains(pack[2].Name, pool);
        Assert.Contains(pod.Picks, pick => pick.Seat == 1 && pick.Card == pack[2].Name && pick.Spent == "Cogwork Librarian");

        await stop.CancelAsync();
        await playing;
        foreach (EventSeat bot in bots) bot.Dispose();
        Assert.Empty(page.Errors);
    }

    [Fact]
    public async Task A_match_game_shows_each_players_match_clock_in_their_box()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store, playedOut: true);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), timing: new BoosterTiming
        {
            FirstRound = TimeSpan.FromMilliseconds(100),
        });
        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(WebClient.Content));
        DraftAndBuild(pod);

        // You come to the pod and are sent on to your game, against Eve, a bot.
        using var stop = new CancellationTokenSource();
        (List<EventSeat> bots, Task playing) = await Bots(host.Address, created, ["eve"], stop.Token);
        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(host.Address.AbsoluteUri)}#ticket={created.Tickets["you"]}", WebClient.Pictures(nameof(BoosterDraftTests)));
        await page.SettledAsync(state => state.Phase == "Playing" && state.Prompt.Length > 0);
        await page.PictureAsync("match-clocks");

        await stop.CancelAsync();
        await playing;
        foreach (EventSeat bot in bots) bot.Dispose();
        Assert.Empty(page.Errors);
    }

    /// <summary>
    /// Drafts the pod and builds every deck straight through the pod, as fast as it can go: each seat takes the first card
    /// of whatever pack is in front of it, and submits all it drafted with basic lands to make forty.
    /// </summary>
    private static void DraftAndBuild(BoosterPod pod)
    {
        for (int seat = 1; seat <= pod.Seats; seat++) pod.Seated(seat);
        while (pod.Stage == EventStage.Drafting)
        {
            for (int seat = 1; seat <= pod.Seats; seat++)
            {
                if (pod.ViewFor(seat) is DraftMessage { Pile.Count: > 0 } draft)
                {
                    Assert.Null(pod.Receive(seat, new DraftPickMessage { Decision = draft.Decision, Take = true, Card = 0 }));
                }
            }
        }

        for (int seat = 1; seat <= pod.Seats && pod.Stage == EventStage.DeckBuilding; seat++)
        {
            var deck = (DeckBuildingMessage)pod.ViewFor(seat)!;
            Assert.Null(pod.Receive(seat, new BuildDeckMessage
            {
                Change = 1,
                Main = [.. Enumerable.Range(0, deck.Pool.Count)],
                Basics = [new BasicLandCount { Name = "Plains", Count = 40 - deck.Pool.Count }],
                Submit = true,
            }));
        }
    }

    [Fact]
    public async Task A_finished_pod_shows_its_final_standings()
    {
        var store = new EventStore(_data);
        CreatedEvent created = Create(store, playedOut: true);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), timing: new BoosterTiming
        {
            FirstRound = TimeSpan.FromMilliseconds(200),
            BetweenRounds = TimeSpan.FromMilliseconds(200),
            AfterGame = TimeSpan.FromMilliseconds(100),
        });
        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(WebClient.Content));
        using var stop = new CancellationTokenSource(TimeSpan.FromMinutes(4));
        (List<EventSeat> bots, Task playing) = await Bots(host.Address, created, Pod.Select(player => player.Item1), stop.Token);
        await playing;
        Assert.Equal(EventStage.Complete, pod.Stage);

        // Coming back to the pod after it is over: the final standings.
        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(host.Address.AbsoluteUri)}#ticket={pod.IssueTicket("you")}", WebClient.Pictures(nameof(BoosterDraftTests)));
        TableState state = await page.SettledAsync(state => state.Phase == "Pairings" && state.Pairings is { Final: true });
        await page.PictureAsync("final-standings");
        Assert.Equal(3, state.Pairings!.Round);

        foreach (EventSeat bot in bots) bot.Dispose();
        Assert.Empty(page.Errors);
    }

    /// <summary>A clock a test moves on by hand, from now.</summary>
    private sealed class Clock : TimeProvider
    {
        private DateTimeOffset _now = TimeProvider.System.GetUtcNow();
        public override DateTimeOffset GetUtcNow() => _now;
        public void Advance(TimeSpan by) => _now += by;
    }

    [Fact]
    public async Task A_player_drops_from_the_standings_page_and_the_round_is_paired_again_without_them()
    {
        // Drafted and built by the pod's own clocks, moved on by hand, to a first round that begins in an hour.
        var store = new EventStore(_data);
        var clock = new Clock();
        CreatedEvent created = Create(store);
        BoosterPod pod = BoosterPod.Open(store, created.Record.Id, ContentCache.Get(WebClient.Content), clock, new BoosterTiming { FirstRound = TimeSpan.FromHours(1) });
        for (int seat = 1; seat <= Pod.Count; seat++) pod.Seated(seat);
        while (pod.Stage != EventStage.Playing)
        {
            clock.Advance(TimeSpan.FromSeconds(30));
            pod.Tick(clock.GetUtcNow());
        }

        await using PodHost host = await PodHost.StartAsync(pod, PodHost.Loopback(WebClient.Content));
        await using TablePage page = await TablePage.OpenAsync(
            web, $"server={Uri.EscapeDataString(host.Address.AbsoluteUri)}#ticket={pod.IssueTicket("you")}", WebClient.Pictures(nameof(BoosterDraftTests)));
        TableState state = await page.SettledAsync(state => state.Phase == "Pairings" && state.Pairings is { Round: 1 });
        Assert.Contains(state.Pairings!.Matches, match => match.Contains("You", StringComparison.Ordinal));

        await page.ClickAsync("Drop", "button");
        await page.SettledAsync(state => state.Controls.Any(control => control.Label == "Yes, drop"));
        await page.PictureAsync("drop-asked-again");
        await page.ClickAsync("Yes, drop", "button");

        // Paired again without this seat - three matches and a bye - and nothing left to drop from.
        state = await page.SettledAsync(state => state.Pairings is { } pairings && !pairings.Matches.Any(match => match.Contains("You", StringComparison.Ordinal)));
        await page.PictureAsync("dropped");
        Assert.Equal(4, state.Pairings!.Matches.Count);
        Assert.DoesNotContain(state.Controls, control => control.Label is "Drop" or "Yes, drop");
        Assert.True(store.LoadEvent(created.Record.Id)!.Participant("you")!.Dropped);
        Assert.Empty(page.Errors);
    }
}
