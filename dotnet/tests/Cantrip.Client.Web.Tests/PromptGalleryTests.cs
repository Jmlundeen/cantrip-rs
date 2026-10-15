using Cantrip.Client.Presentation;
using Xunit;

namespace Cantrip.Client.Web.Tests;

/// <summary>
/// Every prompt dock the client can show, from the canned positions in <see cref="PromptFixtures"/>: no game is dealt,
/// so there is nothing to reach and nothing to break when a card is added to a deck. One page for them all - the
/// gallery changes fixture by the address's fragment, without reloading - and a picture of each, named after it, under
/// TestResults\web\PromptGalleryTests.
/// </summary>
[Collection(WebClientCollection.Name)]
public sealed class PromptGalleryTests(WebClient web)
{
    [Fact]
    public async Task Every_prompt_dock_is_drawn()
    {
        IReadOnlyList<string> names = PromptFixtures.Names;
        await using TablePage table = await TablePage.OpenAsync(
            web, $"prompt={Uri.EscapeDataString(names[0])}", WebClient.Pictures(nameof(PromptGalleryTests)));

        foreach (string name in names)
        {
            await table.Page.EvaluateAsync("name => location.hash = name", name);
            // Not every fixture is a question: the lobby is a table before the game, with nothing asked.
            await table.SettledAsync(state => state.Fixture == name);
            await table.PictureAsync(name);
        }

        Assert.Empty(table.Errors);
    }

    /// <summary>
    /// A list of options too long for a button each - every creature type - is found by typing: the box narrows the
    /// buttons to what matches, a click on one answers with that option, and Enter takes the one the typing settles on.
    /// </summary>
    [Fact]
    public async Task A_long_option_list_is_found_by_typing()
    {
        const string Fixture = "choose-option-long";
        await using TablePage table = await TablePage.OpenAsync(
            web, $"prompt={Fixture}", WebClient.Pictures(nameof(PromptGalleryTests)));

        TableState opened = await table.SettledAsync(state => state.Fixture == Fixture);
        Assert.Contains(opened.Controls, control => control.Kind == "textbox" && control.Label == "Type to find one");
        Assert.True(Options(opened).Count <= 8, $"Too many option buttons: {string.Join(", ", Options(opened))}");
        Assert.DoesNotContain("Zombie", Options(opened));

        await table.TypeAsync("Type to find one", "dra");
        TableState narrowed = await table.SettledAsync(state => Options(state).Contains("Hydra"));
        Assert.Equal(["Dragon", "Drake", "Eldrazi", "Hydra"], Options(narrowed));
        await table.PictureAsync("typed-dra");

        await table.ClickAsync("Drake", "button");
        TableState clicked = await table.SettledAsync(state => state.Answered.Length > 0);
        Assert.Equal($"ModesChosenAnswer {Index("Drake")}", clicked.Answered);

        await table.Page.EvaluateAsync("name => location.hash = name", "modes-one");
        await table.SettledAsync(state => state.Fixture == "modes-one");
        await table.Page.EvaluateAsync("name => location.hash = name", Fixture);
        await table.SettledAsync(state => state.Fixture == Fixture && state.Answered.Length == 0);
        await table.TypeAsync("Type to find one", "time lord");
        await table.SettledAsync(state => Options(state).SequenceEqual(["Time Lord"]));
        await table.Page.Keyboard.PressAsync("Enter");
        TableState entered = await table.SettledAsync(state => state.Answered.Length > 0);
        Assert.Equal($"ModesChosenAnswer {Index("Time Lord")}", entered.Answered);

        Assert.Empty(table.Errors);
    }

    /// <summary>The option buttons on the dock: every button but the ones the page always has.</summary>
    private static List<string> Options(TableState state) =>
        [.. state.Controls.Where(control => control.Kind == "button" && Index(control.Label) >= 0).Select(control => control.Label)];

    private static int Index(string option) =>
        PromptFixtures.CreatureTypes.ToList().IndexOf(option);
}
