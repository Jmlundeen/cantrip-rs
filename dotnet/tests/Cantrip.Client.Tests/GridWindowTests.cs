using Cantrip.Client.Presentation;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>Which cells of a grid of equal cells a window onto it covers, as the deck editor's search lays them out.</summary>
public sealed class GridWindowTests
{
    [Fact]
    public void The_grid_stands_as_tall_as_its_rows_and_the_gaps_between_them()
    {
        var grid = new GridWindow(Count: 10, Across: 4, CellHeight: 100, Gap: 10);

        Assert.Equal(3, grid.Rows);
        Assert.Equal(320, grid.Height);
        Assert.Equal([0d, 0d, 0d, 0d, 110d, 110d, 110d, 110d, 220d, 220d], Enumerable.Range(0, 10).Select(grid.Top));
        Assert.Equal(0, new GridWindow(0, 4, 100, 10).Height);
    }

    [Fact]
    public void A_window_covers_every_row_any_part_of_which_is_in_it()
    {
        var grid = new GridWindow(Count: 100, Across: 5, CellHeight: 100, Gap: 10);

        // Rows 0 and 1, the second only in part.
        Assert.Equal((0, 10), grid.Within(0, 150));
        // In the gap under row 0 and into row 2: row 0 has ended, so rows 1 and 2.
        Assert.Equal((5, 15), grid.Within(105, 120));
        // Ending exactly where row 2 starts takes none of it.
        Assert.Equal((5, 10), grid.Within(105, 115));
    }

    [Fact]
    public void Spare_rows_either_side_are_kept_inside_the_grid()
    {
        var grid = new GridWindow(Count: 100, Across: 5, CellHeight: 100, Gap: 10);

        Assert.Equal((5, 35), grid.Within(330, 200, spare: 2));
        Assert.Equal((0, 15), grid.Within(0, 50, spare: 2));
        Assert.Equal((85, 100), grid.Within(grid.Height - 50, 50, spare: 2));
    }

    [Fact]
    public void The_last_row_of_a_long_list_is_reached_at_the_bottom_and_may_be_short()
    {
        // The whole card pool at a typical width: the last card is laid out once the view is scrolled to the bottom.
        var grid = new GridWindow(Count: 1789, Across: 7, CellHeight: 140, Gap: 8);
        double view = 600;

        (int first, int end) = grid.Within(grid.Height - view, view);

        Assert.Equal(1789, end);
        Assert.True(first > 1700);
        Assert.True(end - first <= 7 * 6);
    }

    [Fact]
    public void A_window_past_the_end_or_onto_an_empty_grid_covers_nothing()
    {
        Assert.Equal((0, 0), new GridWindow(0, 5, 100, 10).Within(0, 500));
        Assert.Equal((0, 0), new GridWindow(10, 5, 100, 10).Within(5_000, 500));
        Assert.Equal((0, 0), new GridWindow(10, 5, 100, 10).Within(0, 0));
    }

    [Fact]
    public void A_window_above_the_top_covers_the_rows_it_reaches()
    {
        Assert.Equal((0, 5), new GridWindow(10, 5, 100, 10).Within(-50, 100));
    }
}
