using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// Going back to the page a player came to a table from once its game is over: the site's draft queues after a draft's
/// game, its constructed play after a constructed one - and nowhere from a table that names no page.
/// </summary>
public sealed class GoingBackTests
{
    private static readonly DateTimeOffset Start = new(2026, 10, 19, 12, 0, 0, TimeSpan.Zero);
    private static readonly TimeSpan Wait = TimeSpan.FromSeconds(30);

    private static TableMessage Table(string page, string name) => new() { Phase = TablePhase.Playing, Return = page, ReturnName = name };

    [Fact]
    public void Nothing_is_offered_while_the_game_is_being_played()
    {
        var back = new GoingBack(Wait);

        Assert.Null(back.Follow(Table("https://cantrip.example/draft", "Draft Queues"), over: false, Start));
        Assert.Null(back.Address);
        Assert.False(back.Counting);
        Assert.Equal(string.Empty, back.Countdown(Start));
    }

    [Fact]
    public void A_table_that_names_no_page_has_no_way_back_however_long_the_result_is_up()
    {
        // A table of the player's own (tools\play-vs-bot.cmd, ?bots=2), or a booster pod's game, which sends its players on itself.
        var back = new GoingBack(Wait);

        Assert.Null(back.Follow(Table(string.Empty, string.Empty), over: true, Start));
        Assert.Null(back.Follow(Table(string.Empty, string.Empty), over: true, Start + TimeSpan.FromMinutes(5)));
        Assert.Null(back.Follow(null, over: true, Start + TimeSpan.FromMinutes(5)));
        Assert.Null(back.Address);
        Assert.Null(back.Go());
    }

    [Theory]
    [InlineData("https://cantrip.example/draft", "Draft Queues")]
    [InlineData("https://cantrip.example/constructed", "Constructed Play")]
    public void The_result_is_shown_for_a_while_and_then_the_page_goes_back_by_itself_once(string page, string name)
    {
        var back = new GoingBack(Wait);
        TableMessage table = Table(page, name);

        Assert.Null(back.Follow(table, over: true, Start));
        Assert.Equal(new Uri(page), back.Address);
        Assert.Equal($"Back to {name}", back.Button);
        Assert.Equal($"Back to {name} in 30 s", back.Countdown(Start));
        Assert.Equal($"Back to {name} in 5 s", back.Countdown(Start + TimeSpan.FromSeconds(25)));

        Assert.Null(back.Follow(table, over: true, Start + TimeSpan.FromSeconds(29)));
        Assert.Equal(new Uri(page), back.Follow(table, over: true, Start + Wait));

        // Gone once: the page is on its way, and the next tick does not set off again.
        Assert.Null(back.Follow(table, over: true, Start + Wait + TimeSpan.FromSeconds(1)));
        Assert.False(back.Counting);
    }

    [Fact]
    public void The_button_goes_back_at_once()
    {
        var back = new GoingBack(Wait);
        TableMessage table = Table("https://cantrip.example/constructed", "Constructed Play");
        back.Follow(table, over: true, Start);

        Assert.Equal(new Uri("https://cantrip.example/constructed"), back.Go());
        Assert.Null(back.Go());
        Assert.Null(back.Follow(table, over: true, Start + Wait));
    }

    [Fact]
    public void A_player_looking_at_the_board_is_not_taken_away_but_can_still_go()
    {
        var back = new GoingBack(Wait);
        TableMessage table = Table("https://cantrip.example/draft", "Draft Queues");
        back.Follow(table, over: true, Start);

        back.Stay();

        Assert.Null(back.Follow(table, over: true, Start + TimeSpan.FromMinutes(10)));
        Assert.False(back.Counting);
        Assert.Equal(string.Empty, back.Countdown(Start + TimeSpan.FromMinutes(10)));
        Assert.Equal("Back to Draft Queues", back.Button);
        Assert.Equal(new Uri("https://cantrip.example/draft"), back.Go());
    }

    [Theory]
    [InlineData("javascript:alert(1)")]
    [InlineData("/draft")]
    [InlineData("ftp://cantrip.example/draft")]
    [InlineData("not a page")]
    public void Only_an_absolute_web_address_is_gone_to(string page)
    {
        var back = new GoingBack(Wait);

        Assert.Null(back.Follow(Table(page, "Elsewhere"), over: true, Start + Wait));
        Assert.Null(back.Address);
    }
}
