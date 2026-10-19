using Cantrip.Client.Presentation;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>Which cards a rectangle dragged out over the deck editor's columns picks: those it touches where they show.</summary>
public sealed class MarqueeTests
{
    /// <summary>A column of cards 100 by 140, each 24 below the last, as the deck's columns stack them: only names show but the last.</summary>
    private static List<Box> Column(double x, int cards) => [.. Enumerable.Range(0, cards).Select(row => new Box(x, row * 24, 100, 140))];

    [Fact]
    public void A_drag_either_way_makes_the_same_box_never_thinner_than_one()
    {
        Assert.Equal(new Box(10, 20, 30, 40), Marquee.Between(40, 60, 10, 20));
        Assert.Equal(new Box(10, 20, 30, 40), Marquee.Between(10, 20, 40, 60));
        Assert.Equal(new Box(5, 5, 1, 1), Marquee.Between(5, 5, 5, 5));
    }

    [Fact]
    public void A_card_touched_only_in_part_is_picked_and_one_not_reached_is_not()
    {
        List<Box> row = [new(0, 0, 100, 140), new(110, 0, 100, 140), new(220, 0, 100, 140)];

        Assert.Equal([0, 1], Marquee.Touched(row, Marquee.Between(90, 130, 115, 200)));
        Assert.Empty(Marquee.Touched(row, Marquee.Between(101, 10, 109, 120)));
    }

    [Fact]
    public void In_a_column_only_the_cards_whose_showing_part_is_crossed_are_picked()
    {
        List<Box> column = Column(0, 5);

        // Across the names of the second and third cards only: the cards under them are covered there.
        Assert.Equal([1, 2], Marquee.Touched(column, Marquee.Between(-10, 30, 50, 60)));

        // Low on the last card, the whole of which shows: it alone, though every card's box reaches down there.
        Assert.Equal([4], Marquee.Touched(column, Marquee.Between(-10, 200, 50, 220)));

        // Down the whole column: every card.
        Assert.Equal([0, 1, 2, 3, 4], Marquee.Touched(column, Marquee.Between(-10, -10, 20, 300)));
    }

    [Fact]
    public void A_rectangle_over_several_columns_picks_from_each()
    {
        List<Box> cards = [.. Column(0, 3), .. Column(110, 2), .. Column(220, 4)];

        // From below the first column's last card, up and right into the second column's last card.
        IReadOnlyList<int> touched = Marquee.Touched(cards, Marquee.Between(50, 250, 150, 150));

        Assert.Equal([2, 4], touched);
    }

    [Fact]
    public void A_box_less_another_leaves_what_is_outside_it()
    {
        var box = new Box(0, 0, 100, 100);

        Assert.Equal([box], box.Without(new Box(200, 200, 10, 10)));
        Assert.Empty(box.Without(new Box(-5, -5, 200, 200)));
        Assert.Equal(100 * 100 - 20 * 20, box.Without(new Box(40, 40, 20, 20)).Sum(part => part.Width * part.Height));
        Assert.Equal([new Box(0, 50, 100, 50)], box.Without(new Box(-10, -10, 200, 60)));
    }
}
