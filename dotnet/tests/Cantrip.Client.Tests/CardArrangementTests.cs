using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// How a library search sorts and groups what it shows. Presentation only, but a sort that loses a card or a grouping
/// that files a card under the wrong heading is a card the player does not find.
/// </summary>
public sealed class CardArrangementTests
{
    private static int _next = 1;

    private static ObjectView Card(string name, int manaValue, CardTypeView[] types, params ColorView[] colors) => new()
    {
        Handle = new ViewHandle(_next++),
        Name = name,
        ManaValue = manaValue,
        CardTypes = [.. types],
        Colors = [.. colors],
    };

    private static readonly ObjectView Bolt = Card("Lightning Bolt", 1, [CardTypeView.Instant], ColorView.Red);
    private static readonly ObjectView Island = Card("Island", 0, [CardTypeView.Land]);
    private static readonly ObjectView Bear = Card("Grizzly Bears", 2, [CardTypeView.Creature], ColorView.Green);
    private static readonly ObjectView Counterspell = Card("Counterspell", 2, [CardTypeView.Instant], ColorView.Blue);
    private static readonly ObjectView Decay = Card("Abrupt Decay", 2, [CardTypeView.Instant], ColorView.Black, ColorView.Green);
    private static readonly ObjectView Mox = Card("Mox Sapphire", 0, [CardTypeView.Artifact]);
    private static readonly ObjectView Dryad = Card("Dryad Arbor", 0, [CardTypeView.Land, CardTypeView.Creature], ColorView.Green);
    private static readonly ObjectView Seat = Card("Seat of the Synod", 0, [CardTypeView.Artifact, CardTypeView.Land]);

    private static readonly ObjectView[] Library = [Bolt, Island, Bear, Counterspell, Decay, Mox];

    private static string[] Names(CardGroup group) => [.. group.Cards.Select(card => card.Name)];

    [Fact]
    public void None_and_none_is_the_library_as_it_came()
    {
        CardGroup only = Assert.Single(CardArrangement.Arrange(Library, CardSort.None, CardGrouping.None));

        Assert.Equal(string.Empty, only.Heading);
        Assert.Equal(Library, only.Cards);
    }

    [Fact]
    public void Sorting_by_name_ignores_case()
    {
        ObjectView lower = Card("air Elemental", 5, [CardTypeView.Creature], ColorView.Blue);

        CardGroup only = Assert.Single(CardArrangement.Arrange([Bolt, lower, Island], CardSort.Name, CardGrouping.None));

        Assert.Equal(["air Elemental", "Island", "Lightning Bolt"], Names(only));
    }

    [Fact]
    public void Sorting_by_mana_value_keeps_ties_in_library_order()
    {
        CardGroup only = Assert.Single(CardArrangement.Arrange(Library, CardSort.ManaValue, CardGrouping.None));

        // Island and Mox at 0, then Bolt, then the three twos as the library had them.
        Assert.Equal(["Island", "Mox Sapphire", "Lightning Bolt", "Grizzly Bears", "Counterspell", "Abrupt Decay"], Names(only));
    }

    [Fact]
    public void Grouping_by_color_runs_the_pie_then_multicolor_then_colorless()
    {
        IReadOnlyList<CardGroup> groups = CardArrangement.Arrange(Library, CardSort.None, CardGrouping.Color);

        Assert.Equal(["Blue", "Red", "Green", "Multicolor", "Colorless"], groups.Select(group => group.Heading));
        Assert.Equal(["Island", "Mox Sapphire"], Names(groups[^1]));
        Assert.Equal(["Abrupt Decay"], Names(groups[3]));
    }

    [Fact]
    public void Grouping_by_type_files_a_card_with_several_types_once()
    {
        IReadOnlyList<CardGroup> groups = CardArrangement.Arrange(
            [.. Library, Dryad, Seat],
            CardSort.None,
            CardGrouping.Type);

        Assert.Equal(["Creatures", "Instants", "Artifacts", "Lands"], groups.Select(group => group.Heading));

        // A land creature is a creature; an artifact land is a land.
        Assert.Equal(["Grizzly Bears", "Dryad Arbor"], Names(groups[0]));
        Assert.Equal(["Island", "Seat of the Synod"], Names(groups[^1]));
        Assert.Equal(Library.Length + 2, groups.Sum(group => group.Cards.Count));
    }

    [Fact]
    public void Each_group_is_sorted_within_itself()
    {
        IReadOnlyList<CardGroup> groups = CardArrangement.Arrange(Library, CardSort.Name, CardGrouping.Type);

        Assert.Equal(["Abrupt Decay", "Counterspell", "Lightning Bolt"], Names(groups.Single(group => group.Heading == "Instants")));
    }

    [Fact]
    public void Nothing_to_arrange_is_no_groups()
    {
        Assert.Empty(CardArrangement.Arrange([], CardSort.Name, CardGrouping.None));
        Assert.Empty(CardArrangement.Arrange([], CardSort.None, CardGrouping.Type));
    }
}
