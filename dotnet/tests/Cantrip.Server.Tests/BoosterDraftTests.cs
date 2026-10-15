using Cantrip.Core.Random;
using Cantrip.Server.Events;
using Xunit;

namespace Cantrip.Server.Tests;

public sealed class BoosterDraftTests
{
    private static readonly DateTimeOffset Start = new(2026, 10, 1, 12, 0, 0, TimeSpan.Zero);

    private static List<string> Cards(int count) => [.. Enumerable.Range(1, count).Select(i => $"card-{i:D3}")];

    private static BoosterDraft Deal(int seats = 8, int packs = 3, int packSize = 15) =>
        BoosterDraft.Start(Cards(seats * packs * packSize + 20), seats, packs, packSize, new Pcg32Random(7), 7, Start);

    [Fact]
    public void Every_seat_opens_a_pack_of_fifteen_with_its_clock_running()
    {
        BoosterDraft draft = Deal();

        Assert.Equal(1, draft.Pack);
        Assert.Equal(2, draft.Unopened.Count);
        for (int seat = 1; seat <= 8; seat++)
        {
            Assert.Equal(15, draft.Current(seat)!.Cards.Count);
            Assert.Equal(seat, draft.Current(seat)!.Opener);
            Assert.Equal(TimeSpan.FromSeconds(75), draft.Left(seat, Start));
        }

        // 360 different cards dealt, none twice.
        List<string> dealt = [.. draft.Queues.SelectMany(queue => queue.SelectMany(pack => pack.Cards)), .. draft.Unopened.SelectMany(round => round.SelectMany(pack => pack))];
        Assert.Equal(360, dealt.Distinct().Count());
    }

    [Fact]
    public void A_pick_passes_the_rest_of_the_pack_left_in_pack_one_and_it_waits_behind_the_pack_being_picked()
    {
        BoosterDraft draft = Deal();
        string taken = draft.Current(1)!.Cards[3];

        Assert.Equal(taken, draft.Take(1, 0, 3, Start + TimeSpan.FromSeconds(10)));

        Assert.Equal([taken], draft.PoolOf(1));
        Assert.Null(draft.Current(1));
        Assert.Null(draft.Left(1, Start));
        Assert.Equal(2, draft.QueueOf(2).Count);
        Assert.Equal(14, draft.QueueOf(2)[1].Cards.Count);
        Assert.Equal(1, draft.QueueOf(2)[1].Opener);

        // Seat 2 is still on its own pack's clock; seat 1's pack starts its clock only when it reaches the front.
        Assert.Equal(TimeSpan.FromSeconds(75), draft.Left(2, Start));
        draft.Take(2, 0, 0, Start + TimeSpan.FromSeconds(20));
        Assert.Equal(14, draft.Current(2)!.Cards.Count);
        Assert.Equal(TimeSpan.FromSeconds(70), draft.Left(2, Start + TimeSpan.FromSeconds(20)));
    }

    [Fact]
    public void A_pick_naming_a_pack_that_has_gone_on_is_refused()
    {
        BoosterDraft draft = Deal();
        draft.Take(1, 0, 0, Start);

        Assert.Throws<InvalidOperationException>(() => draft.Take(1, 0, 0, Start));
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, 1, 0, Start));
    }

    [Fact]
    public void A_clock_that_runs_out_takes_a_card_at_random_and_says_so_in_the_log()
    {
        BoosterDraft draft = Deal();
        List<string> pack = [.. draft.Current(3)!.Cards];

        Assert.False(draft.Tick(Start + TimeSpan.FromSeconds(74)));
        Assert.True(draft.Tick(Start + TimeSpan.FromSeconds(75)));

        // Every seat's clock ran out together, so every seat took one.
        Assert.All(Enumerable.Range(1, 8), seat => Assert.Single(draft.PoolOf(seat)));
        BoosterPick pick = draft.Log.Single(entry => entry.Seat == 3);
        Assert.True(pick.Auto);
        Assert.Equal(pack, pick.Offered);
        Assert.Contains(pick.Card, pack);
        Assert.Equal((1, 1, 3), (pick.Pack, pick.Pick, pick.Opener));
    }

    [Fact]
    public void A_clock_that_runs_out_takes_the_best_card_in_the_pack_by_the_pick_order()
    {
        // The cube's best cards are its highest numbered ones; a few it does not rank at all.
        List<string> cube = Cards(380);
        List<string> order = [.. cube.AsEnumerable().Reverse().Where(card => !card.EndsWith('7'))];
        BoosterDraft draft = BoosterDraft.Start(cube, 8, 3, 15, new Pcg32Random(7), 7, Start, pickOrder: order);

        // Only the cards dealt are kept, still best first.
        List<string> dealt = [.. draft.Queues.SelectMany(queue => queue.SelectMany(pack => pack.Cards)), .. draft.Unopened.SelectMany(round => round.SelectMany(pack => pack))];
        Assert.Equal(dealt.Where(card => !card.EndsWith('7')).OrderDescending(StringComparer.Ordinal), draft.PickOrder);

        List<List<string>> packs = [.. Enumerable.Range(1, 8).Select(seat => draft.Current(seat)!.Cards.ToList())];
        Assert.True(draft.Tick(Start + TimeSpan.FromSeconds(75)));
        for (int seat = 1; seat <= 8; seat++)
        {
            Assert.Equal(packs[seat - 1].Where(card => !card.EndsWith('7')).Max(StringComparer.Ordinal), draft.PoolOf(seat).Single());
        }

        // A card the order does not name ranks after every card it does.
        Assert.Equal(draft.PickOrder.Count, draft.Rank("card-007"));
        Assert.Equal(0, draft.Rank(draft.PickOrder[0]));
    }

    [Fact]
    public void Packs_go_left_then_right_then_left_and_the_next_pack_opens_when_every_pack_is_empty()
    {
        BoosterDraft draft = Deal(seats: 4, packs: 3, packSize: 5);
        Assert.Equal(1, draft.Direction);
        Assert.Equal(2, draft.PassesTo(1));
        Assert.Equal(1, draft.PassesTo(4));

        DrainPack(draft);
        Assert.Equal(2, draft.Pack);
        Assert.Equal(-1, draft.Direction);
        Assert.Equal(4, draft.PassesTo(1));

        // Seat 1's second pack goes to seat 4.
        draft.Take(1, draft.Picks[0], 0, Start);
        Assert.Equal(1, draft.QueueOf(4)[^1].Opener);

        DrainPack(draft);
        Assert.Equal(3, draft.Pack);
        Assert.Equal(1, draft.Direction);

        DrainPack(draft);
        Assert.True(draft.IsOver);
        Assert.All(Enumerable.Range(1, 4), seat => Assert.Equal(15, draft.PoolOf(seat).Count));
        Assert.Equal(60, draft.Log.Count);
    }

    [Fact]
    public void The_last_card_of_a_pack_waits_in_front_of_its_seat_on_a_short_clock()
    {
        BoosterDraft draft = Deal(seats: 2, packs: 1, packSize: 3);

        draft.Take(1, 0, 0, Start);
        draft.Take(2, 0, 0, Start);
        // Each seat now has the other's pack, two cards in it; taking one passes the last on, behind the pack seat 2 has.
        DateTimeOffset passed = Start + TimeSpan.FromSeconds(3);
        draft.Take(1, 1, 0, passed);
        Assert.Single(draft.PoolOf(2));
        Assert.Single(draft.QueueOf(2)[^1].Cards);

        // Seat 2 picks from the pack in front of it, which passes seat 1 its last card, and its own comes up: each on a
        // two-second clock.
        DateTimeOffset reached = passed + TimeSpan.FromSeconds(4);
        draft.Take(2, 1, 0, reached);
        Assert.Single(draft.Current(2)!.Cards);
        Assert.Single(draft.Current(1)!.Cards);
        Assert.Equal(TimeSpan.FromSeconds(BoosterDraft.LastCardSeconds), draft.Left(1, reached));
        Assert.Equal(TimeSpan.FromSeconds(BoosterDraft.LastCardSeconds), draft.Left(2, reached));

        Assert.False(draft.Tick(reached + TimeSpan.FromSeconds(1.9)));
        Assert.True(draft.Tick(reached + TimeSpan.FromSeconds(2)));
        BoosterPick last = draft.Log.Single(pick => pick.Seat == 2 && pick.Pick == 3);
        Assert.True(last.Auto);
        Assert.Single(last.Offered);
        Assert.True(draft.IsOver);
        Assert.Equal(3, draft.PoolOf(1).Count);
        Assert.Equal(3, draft.PoolOf(2).Count);
    }

    [Fact]
    public void A_slow_seat_has_packs_pile_up_in_front_of_it()
    {
        BoosterDraft draft = Deal();

        // Everyone but seat 2 picks as fast as they can, three times round.
        for (int round = 0; round < 3; round++)
        {
            for (int seat = 1; seat <= 8; seat++)
            {
                if (seat != 2 && draft.Current(seat) is not null)
                {
                    draft.Take(seat, draft.Picks[seat - 1], 0, Start);
                }
            }
        }

        // Its own pack, and one from seat 1 for every time round.
        Assert.Equal(4, draft.QueueOf(2).Count);
        Assert.Equal(15, draft.Current(2)!.Cards.Count);
    }

    [Fact]
    public void Resuming_after_a_restart_starts_every_clock_again()
    {
        BoosterDraft draft = Deal();
        DateTimeOffset later = Start + TimeSpan.FromHours(1);

        draft.Resume(later);

        Assert.False(draft.Tick(later + TimeSpan.FromSeconds(74)));
        Assert.Equal(TimeSpan.FromSeconds(75), draft.Left(5, later));
    }

    /// <summary>
    /// A table of eight whose seat 1 has drafted a card with Cogwork Librarian's draft ability with its first pick, and
    /// has seat 8's pack of fourteen in front of it: the first card of each pack is made the Librarian.
    /// </summary>
    private static BoosterDraft LibrarianInSeatOnesPool(int packSize = 15)
    {
        BoosterDraft draft = Deal(packSize: packSize);
        string librarian = draft.Current(1)!.Cards[0];
        draft.AdditionalPicks = [librarian];
        Assert.Empty(draft.Spendable(1));

        draft.Take(1, 0, 0, Start);
        draft.Take(8, 0, 0, Start);
        Assert.Equal([librarian], draft.PoolOf(1));
        Assert.Equal(packSize - 1, draft.Current(1)!.Cards.Count);
        return draft;
    }

    [Fact]
    public void Cogwork_Librarian_in_a_pool_takes_a_second_card_and_goes_into_the_pack_in_their_place()
    {
        BoosterDraft draft = LibrarianInSeatOnesPool();
        string librarian = draft.AdditionalPicks[0];
        Assert.Equal([0], draft.Spendable(1));
        List<string> pack = [.. draft.Current(1)!.Cards];

        Assert.Equal([pack[5], pack[2]], draft.Take(1, 1, [5, 2], Start));

        // The two cards join the pool and the Librarian leaves it, for the pack, which goes on one card smaller as
        // after any pick.
        Assert.Equal([pack[5], pack[2]], draft.PoolOf(1));
        Assert.Equal(2, draft.Picks[0]);
        BoosterPack passed = draft.QueueOf(2)[^1];
        Assert.Equal(8, passed.Opener);
        Assert.Equal(13, passed.Cards.Count);
        Assert.Equal(librarian, passed.Cards[^1]);
        Assert.DoesNotContain(pack[5], passed.Cards);
        Assert.DoesNotContain(pack[2], passed.Cards);

        // Both cards are pick 2 of the pack, and the second says what was spent for it.
        BoosterPick[] picks = [.. draft.Log.TakeLast(2)];
        Assert.Equal((pack[5], 2, string.Empty), (picks[0].Card, picks[0].Pick, picks[0].Spent));
        Assert.Equal((pack[2], 2, librarian), (picks[1].Card, picks[1].Pick, picks[1].Spent));
        Assert.Equal(pack, picks[1].Offered);
        Assert.Empty(draft.Spendable(1));
    }

    [Fact]
    public void A_second_card_needs_a_Librarian_drafted_before_the_pick()
    {
        BoosterDraft draft = Deal();
        draft.AdditionalPicks = [draft.Current(1)!.Cards[0]];

        // Seat 1 has nothing in its pool yet: the Librarian in the pack cannot pay for a card taken with it.
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, 0, [0, 1], Start));
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, 0, [1, 1], Start));
        Assert.Empty(draft.PoolOf(1));
        Assert.Equal(15, draft.Current(1)!.Cards.Count);

        // With one Librarian, one additional card and no more.
        draft = LibrarianInSeatOnesPool();
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, 1, [0, 1, 2], Start));
        Assert.Single(draft.PoolOf(1));
    }

    [Fact]
    public void A_Librarian_spent_on_the_last_two_cards_of_a_pack_is_the_last_card_for_the_next_seat()
    {
        BoosterDraft draft = LibrarianInSeatOnesPool(packSize: 3);
        string librarian = draft.AdditionalPicks[0];

        draft.Take(1, 1, [0, 1], Start);

        Assert.Equal(2, draft.PoolOf(1).Count);
        Assert.DoesNotContain(librarian, draft.PoolOf(1));
        Assert.Equal([librarian], draft.QueueOf(2)[^1].Cards);
    }

    private static void DrainPack(BoosterDraft draft)
    {
        int pack = draft.Pack;
        while (draft.Pack == pack && !draft.IsOver)
        {
            for (int seat = 1; seat <= draft.Seats; seat++)
            {
                if (draft.Pack == pack && draft.Current(seat) is not null)
                {
                    draft.Take(seat, draft.Picks[seat - 1], 0, Start);
                }
            }
        }
    }
}
