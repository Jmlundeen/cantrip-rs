using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Controlling another player (CR 723: Mindslaver, Emrakul, the Promised End), from the canned positions of
/// <see cref="Cantrip.Client.Presentation.PromptFixtures"/>: the page of the seat in control says whose decision it is making,
/// shows the controlled player's hand and plays from it; the page of the seat controlled says who is making its decisions.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class PlayerControlTests(WebClient web)
{
    /// <summary>
    /// This seat controls Bot 2 during Bot 2's turn: the dock says it is deciding for Bot 2, Bot 2's rail says so, Bot 2's hand
    /// is shown face up (CR 723.4), and a click on its Plains in the opened hand plays it - Bot 2's land, played for Bot 2.
    /// </summary>
    [Fact]
    public async Task The_controlling_seat_decides_for_the_player_and_plays_from_their_hand()
    {
        const string Fixture = "priority-controlling";
        await using TablePage table = await TablePage.OpenAsync(web, $"prompt={Fixture}", WebClient.Pictures(nameof(PlayerControlTests)));

        TableState opened = await table.SettledAsync(state => state.Fixture == Fixture && state.Title.Length > 0);
        Assert.StartsWith("Deciding for Bot 2", opened.Title);
        PlayerState rival = Assert.Single(opened.Players ?? [], player => player.Seat == 2);
        Assert.Contains(rival.Facts ?? [], fact => fact.Contains("You control them during their turn", StringComparison.Ordinal));
        // Their hand, which this seat may see while it controls them: both cards, face up in their fan.
        Assert.Contains(opened.Cards, card => card.Name == "Wrath of God" && card.Place.Length == 0);
        await table.PictureAsync("controlling");

        await table.ClickCardAsync("Wrath of God");
        TableState peeked = await table.SettledAsync(state => state.Cards.Any(card => card.Place == "choice" && card.Name == "Plains"));
        CardState plains = peeked.Cards.First(card => card.Place == "choice" && card.Name == "Plains");
        await table.PictureAsync("their-hand");
        await table.Page.Mouse.ClickAsync(plains.X, plains.Y);

        TableState played = await table.SettledAsync(state => state.Answered.Length > 0);
        Assert.Equal($"TakeActionAnswer PlayLand {plains.Handle}", played.Answered);
        Assert.Empty(table.Errors);
    }

    /// <summary>This seat's own turn, controlled by Bot 2: nothing is asked of it, and the dock says who makes its decisions.</summary>
    [Fact]
    public async Task The_controlled_seat_is_told_who_makes_its_decisions()
    {
        const string Fixture = "controlled-waiting";
        await using TablePage table = await TablePage.OpenAsync(web, $"prompt={Fixture}", WebClient.Pictures(nameof(PlayerControlTests)));

        TableState opened = await table.SettledAsync(state => state.Fixture == Fixture && state.Detail.Length > 0);
        Assert.Equal("Bot 2 controls you this turn and makes your decisions", opened.Detail);
        PlayerState me = Assert.Single(opened.Players ?? [], player => player.Seat == 1);
        Assert.Contains(me.Facts ?? [], fact => fact.Contains("Bot 2 controls you during your turn", StringComparison.Ordinal));
        await table.PictureAsync("controlled");
        Assert.Empty(table.Errors);
    }
}
