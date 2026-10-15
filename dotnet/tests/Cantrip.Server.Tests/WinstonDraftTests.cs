using Cantrip.Core.Random;
using Cantrip.Server.Events;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// The Winston draft's own rules, on cards named by number so every move can be followed: a pile taken is started
/// again from the stack, a pile passed gets a card from it, the last pile passed takes the top of the stack, and every
/// card is taken once and only once.
/// </summary>
public sealed class WinstonDraftTests
{
    private static List<string> Cards(int count) => [.. Enumerable.Range(1, count).Select(i => $"c{i}")];

    private static WinstonDraft Start(int cards = 12, int first = 1) =>
        WinstonDraft.Start(Cards(cards), piles: 3, seats: 2, first, new Pcg32Random(5));

    [Fact]
    public void A_draft_lays_a_card_on_each_pile_and_the_first_seat_looks_at_the_first()
    {
        WinstonDraft draft = Start(first: 2);

        Assert.Equal([1, 1, 1], draft.Piles.Select(pile => pile.Count));
        Assert.Equal(9, draft.Stack.Count);
        Assert.Equal(2, draft.Active);
        Assert.Equal(0, draft.Looking);
        Assert.Equal(Cards(12).Order(), draft.Stack.Concat(draft.Piles.SelectMany(pile => pile)).Order());
    }

    [Fact]
    public void Taking_a_pile_starts_it_again_from_the_stack_and_passes_the_turn()
    {
        WinstonDraft draft = Start();
        string top = draft.Stack[0];
        List<string> pile = [.. draft.Piles[0]];

        IReadOnlyList<string> taken = draft.Take(1, draft.Decision);

        Assert.Equal(pile, taken);
        Assert.Equal([top], draft.Piles[0]);
        Assert.Equal(2, draft.Active);
        Assert.Equal(0, draft.Looking);
    }

    [Fact]
    public void Passing_a_pile_adds_the_top_of_the_stack_to_it_and_moves_on()
    {
        WinstonDraft draft = Start();
        string top = draft.Stack[0];

        Assert.Null(draft.Pass(1, draft.Decision));

        Assert.Equal(2, draft.Piles[0].Count);
        Assert.Equal(top, draft.Piles[0][1]);
        Assert.Equal(1, draft.Active);
        Assert.Equal(1, draft.Looking);
    }

    [Fact]
    public void Passing_the_last_pile_takes_the_top_of_the_stack_blind()
    {
        WinstonDraft draft = Start();
        draft.Pass(1, draft.Decision);
        draft.Pass(1, draft.Decision);
        string blind = draft.Stack[1];

        Assert.Equal(blind, draft.Pass(1, draft.Decision));
        Assert.Equal(2, draft.Active);
        Assert.True(draft.History[^1].Blind);
    }

    [Fact]
    public void A_choice_for_a_decision_already_made_or_from_the_wrong_seat_is_refused()
    {
        WinstonDraft draft = Start();
        int stale = draft.Decision;
        draft.Pass(1, stale);

        Assert.Throws<InvalidOperationException>(() => draft.Take(1, stale));
        Assert.Throws<InvalidOperationException>(() => draft.Take(2, draft.Decision));
    }

    [Fact]
    public void Every_card_is_taken_once_and_the_last_cards_must_be_taken()
    {
        WinstonDraft draft = Start(cards: 30);
        Dictionary<int, List<string>> pools = new() { [1] = [], [2] = [] };
        var choices = new Pcg32Random(9);

        while (!draft.IsOver)
        {
            int seat = draft.Active;
            if (draft.CanPass && choices.NextInt(3) > 0)
            {
                if (draft.Pass(seat, draft.Decision) is { } blind) pools[seat].Add(blind);
            }
            else
            {
                Assert.True(draft.CanPass || draft.Stack.Count == 0);
                pools[seat].AddRange(draft.Take(seat, draft.Decision));
            }
        }

        Assert.Equal(Cards(30).Order(), pools.Values.SelectMany(pool => pool).Order());
        Assert.Equal(pools[1], draft.PoolOf(1));
        Assert.Equal(pools[2], draft.PoolOf(2));
        Assert.Equal(0, draft.Active);
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, draft.Decision));
    }

    [Fact]
    public void Passing_the_last_pile_with_one_card_left_in_the_stack_takes_that_card()
    {
        // Six cards in three piles: three on the piles and three in the stack, so the third pass finds one left.
        WinstonDraft draft = WinstonDraft.Start(Cards(6), piles: 3, seats: 2, first: 1, new Pcg32Random(5));
        draft.Pass(1, draft.Decision);
        draft.Pass(1, draft.Decision);
        Assert.Single(draft.Stack);
        string last = draft.Stack[0];

        Assert.True(draft.CanPass);
        Assert.Equal(last, draft.Pass(1, draft.Decision));

        Assert.Empty(draft.Stack);
        Assert.Equal([2, 2, 1], draft.Piles.Select(pile => pile.Count));
        Assert.Equal([last], draft.PoolOf(1));
        Assert.Equal(2, draft.Active);
    }

    [Fact]
    public void With_the_stack_empty_the_last_pile_left_cannot_be_passed()
    {
        WinstonDraft draft = Start(cards: 3);

        Assert.Empty(draft.Stack);
        draft.Pass(1, draft.Decision);
        draft.Pass(1, draft.Decision);

        Assert.False(draft.CanPass);
        Assert.Throws<InvalidOperationException>(() => draft.Pass(1, draft.Decision));
        Assert.Single(draft.Take(1, draft.Decision));
    }

    [Fact]
    public void The_clock_plays_out_the_turn_passing_piles_of_one_and_taking_a_pile_of_two()
    {
        WinstonDraft draft = Start();
        string blind = draft.Stack[3];

        // Every pile has one card: each is passed, a card from the stack onto it, and the last pass takes the top of the stack.
        draft.TakeForClock();
        Assert.Equal([blind], draft.PoolOf(1));
        Assert.Equal([2, 2, 2], draft.Piles.Select(pile => pile.Count));
        Assert.Equal(2, draft.Active);
        Assert.All(draft.History, action => Assert.True(action.Auto));
        Assert.Equal(1, draft.Turns);

        List<string> first = [.. draft.Piles[0]];
        draft.TakeForClock();
        Assert.Equal(first, draft.PoolOf(2));
        Assert.Equal(1, draft.Active);
        Assert.Equal(2, draft.Turns);
    }
}
