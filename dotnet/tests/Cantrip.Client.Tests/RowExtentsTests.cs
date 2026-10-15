using Cantrip.Client.Presentation;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>Where the rows of a list that only makes the rows in view stand, as the game log's panel reads them.</summary>
public sealed class RowExtentsTests
{
    [Fact]
    public void Rows_stand_one_under_another_with_the_gap_between()
    {
        RowExtents rows = Rows(gap: 2, 10, 20, 30);

        Assert.Equal([0d, 12d, 34d], Enumerable.Range(0, 3).Select(rows.Top));
        Assert.Equal(64, rows.Total);
    }

    [Fact]
    public void A_row_measured_taller_than_its_guess_moves_every_row_under_it_and_says_by_how_much()
    {
        RowExtents rows = Rows(gap: 0, 10, 10, 10);
        Assert.Equal(20, rows.Top(2));

        double change = rows.Measure(1, 25);

        Assert.Equal(15, change);
        Assert.Equal(0, rows.Top(0));
        Assert.Equal(10, rows.Top(1));
        Assert.Equal(35, rows.Top(2));
        Assert.Equal(45, rows.Total);
        Assert.True(rows.IsMeasured(1));
        Assert.False(rows.IsMeasured(2));
    }

    [Fact]
    public void A_window_covers_every_row_any_part_of_which_is_in_it()
    {
        RowExtents rows = Rows(gap: 0, 10, 10, 10, 10, 10);

        Assert.Equal((1, 2), rows.Within(15, 25));
        Assert.Equal((1, 1), rows.Within(10, 20));
        Assert.Equal((0, 4), rows.Within(-100, 1000));
        Assert.Equal((4, 4), rows.Within(45, 1000));
    }

    [Fact]
    public void A_window_past_the_end_or_onto_an_empty_list_covers_nothing()
    {
        Assert.Equal((0, -1), Rows(gap: 0, 10, 10).Within(20, 40));
        Assert.Equal((0, -1), Rows(gap: 0).Within(0, 100));
        Assert.Equal(0, Rows(gap: 4).Total);
    }

    [Fact]
    public void Rows_added_after_a_window_was_read_stand_under_the_rest()
    {
        RowExtents rows = Rows(gap: 1, 10);
        Assert.Equal(10, rows.Total);

        rows.Add(5);
        rows.Add(5);

        Assert.Equal(11, rows.Top(1));
        Assert.Equal(17, rows.Top(2));
        Assert.Equal((2, 2), rows.Within(17, 18));
    }

    [Fact]
    public void Remeasuring_keeps_each_height_as_the_guess_until_the_row_is_measured_again()
    {
        RowExtents rows = Rows(gap: 0, 10, 10);
        rows.Measure(0, 30);

        rows.Remeasure();

        Assert.False(rows.IsMeasured(0));
        Assert.Equal(30, rows.Height(0));
        Assert.Equal(30, rows.Top(1));
    }

    [Fact]
    public void Thousands_of_rows_are_found_by_where_they_stand_not_counted_through()
    {
        var rows = new RowExtents(gap: 2);
        for (int i = 0; i < 5000; i++)
        {
            rows.Add(18);
        }

        Assert.Equal((2500, 2502), rows.Within(2500 * 20, (2500 * 20) + 50));
    }

    private static RowExtents Rows(double gap, params double[] heights)
    {
        var rows = new RowExtents(gap);
        foreach (double height in heights)
        {
            rows.Add(height);
        }

        return rows;
    }
}
