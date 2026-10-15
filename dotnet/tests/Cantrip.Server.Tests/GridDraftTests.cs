using Cantrip.Core.Random;
using Cantrip.Server.Events;
using Xunit;

namespace Cantrip.Server.Tests;

/// <summary>
/// The grid draft's own rules, on cards named by number: nine cards a grid, a row or a column for the first player and
/// another of what is left for the second, the rest discarded, and the next grid picked from first by the other player.
/// </summary>
public sealed class GridDraftTests
{
    private static List<string> Cards(int count) => [.. Enumerable.Range(1, count).Select(i => $"c{i}")];

    private static GridDraft Start(int packs = 3, int first = 1, int cards = 0) =>
        GridDraft.Start(Cards(cards == 0 ? packs * GridDraft.PackSize : cards), packs, first, new Pcg32Random(5));

    [Fact]
    public void A_draft_lays_out_a_grid_of_nine_for_the_first_seat()
    {
        GridDraft draft = Start(first: 2);

        Assert.Equal(9, draft.Grid.Count);
        Assert.All(draft.Grid, Assert.NotNull);
        Assert.Equal(18, draft.Stack.Count);
        Assert.Equal(1, draft.Pack);
        Assert.Equal(2, draft.Active);
        Assert.Equal(1, draft.PickOfPack);
        Assert.Equal(Cards(27).Order(), draft.Stack.Concat(draft.Grid.OfType<string>()).Order());
    }

    [Fact]
    public void Only_as_many_cards_as_the_grids_take_are_kept_from_more()
    {
        GridDraft draft = Start(packs: 2, cards: 40);

        Assert.Equal(18, draft.Stack.Count + draft.Grid.Count);
    }

    [Fact]
    public void Rows_and_columns_are_the_grids_lines()
    {
        Assert.Equal([3, 4, 5], GridDraft.Places(1));
        Assert.Equal([2, 5, 8], GridDraft.Places(5));
        Assert.Equal("column 1", GridDraft.Named(3));
    }

    [Fact]
    public void The_first_pick_takes_three_cards_and_the_second_may_take_two_where_the_lines_cross()
    {
        GridDraft draft = Start();
        List<string> grid = [.. draft.Grid.OfType<string>()];

        IReadOnlyList<string> row = draft.Take(1, draft.Decision, 0);
        Assert.Equal([grid[0], grid[1], grid[2]], row);
        Assert.Equal(2, draft.Active);
        Assert.Equal(2, draft.PickOfPack);
        Assert.False(draft.CanTake(0));

        // Column 1 has lost its top card to row 1: two are left of it.
        IReadOnlyList<string> column = draft.Take(2, draft.Decision, 3);
        Assert.Equal([grid[3], grid[6]], column);
        Assert.Equal([.. row], draft.PoolOf(1));
        Assert.Equal([.. column], draft.PoolOf(2));

        // The other four are discarded and the next grid is laid out, for the second seat to pick from first.
        Assert.Equal([grid[4], grid[5], grid[7], grid[8]], draft.Discarded);
        Assert.Equal(2, draft.Pack);
        Assert.Equal(2, draft.Active);
        Assert.Equal(1, draft.PickOfPack);
        Assert.All(draft.Grid, Assert.NotNull);
    }

    [Fact]
    public void The_second_pick_may_take_a_whole_line_beside_the_first()
    {
        GridDraft draft = Start();
        draft.Take(1, draft.Decision, 3);

        Assert.Equal(3, draft.Take(2, draft.Decision, 4).Count);
        Assert.Equal(3, draft.Discarded.Count);
    }

    [Fact]
    public void Who_picks_first_alternates_and_the_draft_ends_after_the_last_grid()
    {
        GridDraft draft = Start(packs: 3);
        List<int> firsts = [];
        while (!draft.IsOver)
        {
            firsts.Add(draft.Active);
            draft.Take(draft.Active, draft.Decision, 0);
            draft.Take(draft.Active, draft.Decision, 1);
        }

        Assert.Equal([1, 2, 1], firsts);
        Assert.Equal(0, draft.Pack);
        Assert.Empty(draft.Grid);
        Assert.Equal(9, draft.PoolOf(1).Count);
        Assert.Equal(9, draft.PoolOf(2).Count);
        Assert.Equal(9, draft.Discarded.Count);
        Assert.Equal(Cards(27).Order(), draft.PoolOf(1).Concat(draft.PoolOf(2)).Concat(draft.Discarded).Order());
    }

    [Fact]
    public void A_pick_out_of_turn_for_an_old_decision_or_of_an_empty_line_is_refused()
    {
        GridDraft draft = Start();
        int decision = draft.Decision;

        Assert.Throws<InvalidOperationException>(() => draft.Take(2, decision, 0));
        Assert.Throws<InvalidOperationException>(() => draft.Take(1, decision, 6));
        draft.Take(1, decision, 0);
        Assert.Throws<InvalidOperationException>(() => draft.Take(2, decision, 1));
        Assert.Throws<InvalidOperationException>(() => draft.Take(2, draft.Decision, 0));
    }

    [Fact]
    public void A_draft_needs_nine_cards_a_grid()
    {
        Assert.Throws<ArgumentException>(() => GridDraft.Start(Cards(17), 2, 1, new Pcg32Random(1)));
    }

    [Fact]
    public void The_pick_order_is_kept_as_far_as_the_drafts_cards_go()
    {
        GridDraft draft = GridDraft.Start(Cards(9), 1, 1, new Pcg32Random(1), ["c3", "nope", "c1"]);

        Assert.Equal(["c3", "c1"], draft.PickOrder);
        Assert.Equal(0, draft.Rank("c3"));
        Assert.Equal(2, draft.Rank("c2"));
    }

    /// <summary>A draft on its first grid, laid out as given, row by row (null a place already taken), with c1 the best card of the cube.</summary>
    private static GridDraft Laid(params string?[] grid) => new()
    {
        Packs = 1,
        Pack = 1,
        Grid = [.. grid],
        First = 1,
        Active = grid.Contains(null) ? 2 : 1,
        PickOrder = Cards(9),
        Pools = [[], []],
    };

    [Fact]
    public void The_clock_takes_the_line_whose_cards_squared_ranks_are_lowest_not_whose_ranks_add_up_lowest()
    {
        // Row 1 and column 1 add up to 14 each, row 2 to 15; but squared, row 1's 9th-best card costs it 81 of its 94,
        // and row 2's 16, 25 and 36 come to 77.
        GridDraft draft = Laid(
            "c2", "c3", "c9",
            "c4", "c5", "c6",
            "c8", "c7", "c1");

        Assert.Equal(1, draft.ClockLine());
        Assert.Equal(["c4", "c5", "c6"], draft.TakeForClock());
        Assert.True(draft.History.Single().Auto);
        Assert.Equal(2, draft.Active);
    }

    [Fact]
    public void A_place_already_taken_counts_as_the_worst_card_left_on_the_grid()
    {
        // Column 1's c1 and c3 alone would be the best, but its empty place counts as c9: 81 + 1 + 9 = 91, against row 2's
        // 1 + 4 + 81 = 86.
        GridDraft draft = Laid(
            null, null, null,
            "c1", "c2", "c9",
            "c3", "c8", "c7");

        Assert.Equal(1, draft.ClockLine());
    }

    [Fact]
    public void Without_a_pick_order_the_clock_takes_the_first_line_it_can()
    {
        GridDraft draft = Start();
        draft.Take(1, draft.Decision, 0);

        Assert.Equal(1, draft.ClockLine());
    }
}
