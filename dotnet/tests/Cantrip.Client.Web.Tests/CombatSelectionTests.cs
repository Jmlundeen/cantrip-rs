using Microsoft.Playwright;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Declaring attackers and blockers by picking creatures up and then clicking what they go at, from the canned positions in
/// <c>PromptFixtures</c>: a click on a creature, or a rectangle dragged over several from an empty part of the battlefield,
/// picks them up; a click on a planeswalker or battle, or the opposing player, sends them all; a click anywhere else on the
/// battlefield puts them down.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class CombatSelectionTests(WebClient web)
{
    [Fact]
    public async Task Creatures_picked_up_attack_the_planeswalker_clicked()
    {
        await using TablePage table = await TablePage.OpenAsync(web, "prompt=attackers-planeswalker", WebClient.Pictures(nameof(CombatSelectionTests)));
        await table.SettledAsync(state => state.Fixture == "attackers-planeswalker");

        // One creature picked up, then another; Cancel puts them both down.
        await table.ClickCardAsync("Vampire Nighthawk");
        TableState one = await table.SettledAsync(state => state.Selected == 1);
        await table.ClickCardAsync("Gurmag Angler");
        TableState two = await table.SettledAsync(state => state.Selected == 2);
        Assert.Contains(two.Controls, control => control.Kind == "button" && control.Label == "Cancel");
        await table.PictureAsync("two-picked-up");

        await table.ClickAsync("Cancel", "button");
        TableState cancelled = await table.SettledAsync(state => state.Selected == 0);
        Assert.DoesNotContain(cancelled.Controls, control => control.Label == "Cancel");

        // A press on an empty part of the battlefield puts them down as well.
        await table.ClickCardAsync("Vampire Nighthawk");
        await table.SettledAsync(state => state.Selected == 1);
        CardState lurker = await table.CardAsync("Vampire Nighthawk");
        await table.Page.Mouse.ClickAsync(lurker.X, lurker.Y - (lurker.H * 0.5f) - 25);
        await table.SettledAsync(state => state.Selected == 0);

        // A rectangle dragged over both picks both up.
        CardState brute = await table.CardAsync("Gurmag Angler");
        float left = Math.Min(lurker.X, brute.X) - (lurker.W * 0.8f);
        float right = Math.Max(lurker.X, brute.X) + (lurker.W * 0.8f);
        float top = lurker.Y - (lurker.H * 0.5f) - 25;
        float bottom = lurker.Y + (lurker.H * 0.3f);
        await table.Page.Mouse.MoveAsync(left, top);
        await table.Page.Mouse.DownAsync();
        await table.Page.Mouse.MoveAsync((left + right) / 2, (top + bottom) / 2, new MouseMoveOptions { Steps = 5 });
        await table.PictureAsync("dragging");
        await table.Page.Mouse.MoveAsync(right, bottom, new MouseMoveOptions { Steps = 5 });
        await table.Page.Mouse.UpAsync();
        TableState swept = await table.SettledAsync(state => state.Selected == 2);
        await table.PictureAsync("swept");

        // Clicking the planeswalker sends them both at it.
        CardState walker = swept.Cards.First(card => card.Visible && card.Name.Contains("Saheeli", StringComparison.Ordinal));
        await table.Page.Mouse.ClickAsync(walker.X, walker.Y);
        TableState sent = await table.SettledAsync(state => state.Selected == 0 && state.Controls.Any(control => control.Label == "Attack with 2"));
        await table.PictureAsync("sent");
        Assert.DoesNotContain(sent.Controls, control => control.Label == "Cancel");

        Assert.Empty(table.Errors);
    }

    [Fact]
    public async Task Creatures_picked_up_block_the_attacker_clicked()
    {
        await using TablePage table = await TablePage.OpenAsync(web, "prompt=blockers", WebClient.Pictures(nameof(CombatSelectionTests)));

        // The fixture starts with the first blocker already picked up, as a click on it would have.
        await table.SettledAsync(state => state.Fixture == "blockers" && state.Selected == 1);
        await table.ClickCardAsync("Gurmag Angler");
        TableState two = await table.SettledAsync(state => state.Selected == 2);
        Assert.Contains(two.Controls, control => control.Kind == "button" && control.Label == "Cancel");
        await table.PictureAsync("blockers-picked-up");

        await table.ClickCardAsync("Savannah Lions");
        TableState sent = await table.SettledAsync(state => state.Selected == 0 && state.Controls.Any(control => control.Label == "Block with 2"));
        await table.PictureAsync("blockers-sent");
        Assert.DoesNotContain(sent.Controls, control => control.Label == "Cancel");

        Assert.Empty(table.Errors);
    }

    [Fact]
    public async Task An_attacker_that_cant_be_blocked_by_more_than_one_creature_takes_one_of_the_blockers_picked_up()
    {
        await using TablePage table = await TablePage.OpenAsync(web, "prompt=blockers-one-at-most", WebClient.Pictures(nameof(CombatSelectionTests)));
        await table.SettledAsync(state => state.Fixture == "blockers-one-at-most");

        await table.ClickCardAsync("Vampire Nighthawk");
        await table.SettledAsync(state => state.Selected == 1);
        await table.ClickCardAsync("Gurmag Angler");
        await table.SettledAsync(state => state.Selected == 2);

        // Savannah Lions takes one blocker (CR 509.1b); the other stays picked up, and the dock says why.
        await table.ClickCardAsync("Savannah Lions");
        TableState one = await table.SettledAsync(state => state.Selected == 1 && state.Controls.Any(control => control.Label == "Block with 1"));
        Assert.Equal("Savannah Lions can't be blocked by more than one creature", one.Detail);
        await table.PictureAsync("one-blocker-at-most");

        // A second click on it sends nothing more; the one still held may block the Serra Angel.
        await table.ClickCardAsync("Savannah Lions");
        await table.SettledAsync(state => state.Selected == 1 && state.Controls.Any(control => control.Label == "Block with 1"));
        await table.ClickCardAsync("Serra Angel");
        TableState both = await table.SettledAsync(state => state.Selected == 0 && state.Controls.Any(control => control.Label == "Block with 2"));
        Assert.Equal("2 attacking  ·  2 blocking", both.Detail);

        Assert.Empty(table.Errors);
    }

    /// <summary>
    /// The page holds whatever blocks the rules' verdict offers and knows no rule of its own (CR 509.1a): where the verdict lets
    /// a creature block two attackers (the gallery's stand-in for "can block an additional creature each combat"), the creature
    /// blocking one is picked up again and blocks the other too, and the declaration sent has both blocks.
    /// </summary>
    [Fact]
    public async Task A_creature_the_verdict_lets_block_two_attackers_blocks_both()
    {
        await using TablePage table = await TablePage.OpenAsync(web, "prompt=blockers-two-each", WebClient.Pictures(nameof(CombatSelectionTests)));
        await table.SettledAsync(state => state.Fixture == "blockers-two-each");

        await table.ClickCardAsync("Vampire Nighthawk");
        await table.SettledAsync(state => state.Selected == 1);
        await table.ClickCardAsync("Savannah Lions");
        await table.SettledAsync(state => state.Selected == 0 && state.Controls.Any(control => control.Label == "Block with 1"));

        // Blocking the Lions, the Nighthawk may still block the Serra Angel: a click picks it up again, not off the Lions.
        await table.ClickCardAsync("Vampire Nighthawk");
        await table.SettledAsync(state => state.Selected == 1);
        await table.ClickCardAsync("Serra Angel");
        TableState both = await table.SettledAsync(state => state.Selected == 0 && state.Controls.Any(control => control.Label == "Block with 1"));
        Assert.Equal("2 attacking  ·  1 blocking", both.Detail);
        await table.PictureAsync("one-creature-blocking-two");

        await table.ClickAsync("Block with 1", "button");
        TableState sent = await table.SettledAsync(state => state.Answered.Length > 0);
        Assert.StartsWith("BlockersDeclaredAnswer ", sent.Answered);
        Assert.Equal(2, sent.Answered.Split(',').Length);

        Assert.Empty(table.Errors);
    }
}
