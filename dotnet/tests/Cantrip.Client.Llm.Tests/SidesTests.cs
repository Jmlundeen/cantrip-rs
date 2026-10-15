using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Llm.Tests.Board;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// Telling the two sides of the table apart.
/// <para>
/// This is here because it went wrong in a real game. The client names its human player "You" by default, and this
/// seat used to read the other side by name — so a burn spell offered its two player targets as "you" and "You",
/// differing by one capital letter, and the model picked the wrong one twice in a row and shot itself in the face.
/// It was not misjudging anything. It was answering the question it was asked.
/// </para>
/// <para>
/// So no player is named here at all: a name says nothing a decision needs, and the one thing that matters about
/// the other side of the table is that it is the other side of the table.
/// </para>
/// </summary>
public sealed class SidesTests
{
    [Fact]
    public void Never_offers_two_players_that_differ_only_by_their_case()
    {
        // The exact shape of the bug: the human across the table is called "You".
        var chooser = new ScriptedChooser([1]);

        new Decisions(chooser).Answer(
            Facing("You"),
            new ChooseTargetPromptMessage
            {
                Player = Them,
                Ability = Handle(9),
                Text = "Chain Lightning deals 3 damage to any target",
                Candidates = [new() { Player = Me }, new() { Player = Them }],
            });

        IReadOnlyList<string> choices = chooser.Asked[0].Choices;

        Assert.Equal(2, choices.Count);
        Assert.NotEqual(choices[0], choices[1], StringComparer.OrdinalIgnoreCase);
        Assert.DoesNotContain("You", choices, StringComparer.Ordinal);
    }

    [Fact]
    public void Calls_the_two_sides_Yourself_and_Opponent()
    {
        var chooser = new ScriptedChooser([0]);

        AnswerPayload answer = new Decisions(chooser).Answer(
            Facing("Bot 1"),
            new ChooseTargetPromptMessage
            {
                Player = Them,
                Ability = Handle(9),
                Text = "deals 3 damage to any target",
                Candidates = [new() { Player = Me }, new() { Player = Them }],
            });

        Assert.Equal(["Opponent", "Yourself"], chooser.Asked[0].Choices);

        // And the answer is still the seat the choice named, not the word.
        Assert.Equal(Me, Assert.Single(Assert.IsType<TargetsChosenAnswer>(answer).Targets).Player);
    }

    [Fact]
    public void Names_nobody_anywhere_in_the_position()
    {
        // Not only in choices. A log line reading "You cast Serra Angel" is the same trap one sentence later.
        GameClient client = Facing("You");

        string position = GameText.Position(client);
        Assert.DoesNotContain("You:", position, StringComparison.Ordinal);
        Assert.Contains("Yourself:", position, StringComparison.Ordinal);
        Assert.Contains("Opponent:", position, StringComparison.Ordinal);

        Assert.Equal(
            "Opponent cast Serra Angel #6, targeting Yourself",
            GameText.Line(client, new SpellCastEvent
            {
                Spell = Handle(6),
                Controller = Me,
                Targets = [new() { Player = Them }],
            }));
    }

    [Fact]
    public void Tells_several_opponents_apart_by_seat()
    {
        // "Opponent" alone stops being an answer the moment there is more than one of them.
        GameClient client = Crowded();

        Assert.Equal("Opponent (seat 1)", GameText.Player(client, 1));
        Assert.Equal("Opponent (seat 3)", GameText.Player(client, 3));
        Assert.Equal("Yourself", GameText.Player(client, Them));
    }

    /// <summary>A two-player game seen from seat 2, with the seat-1 player called whatever a test wants.</summary>
    private static GameClient Facing(string opponent)
    {
        var transport = new Held();
        var client = new GameClient(transport);

        PlayerView view = View(Creature(6, "Serra Angel", 4, 4, Me));
        view.Viewer = Them;
        view.Players[0].Name = opponent;

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Them, PlayerNames = [opponent, "Bot 2"] });
        transport.Inbound.Enqueue(new StateMessage { View = view });
        client.Pump();

        return client;
    }

    /// <summary>A three-player game seen from seat 2.</summary>
    private static GameClient Crowded()
    {
        var transport = new Held();
        var client = new GameClient(transport);

        PlayerView view = View();
        view.Viewer = Them;
        view.Players.Add(new PlayerSummaryView { Seat = 3, Name = "Bot 3", Team = 3, Life = 20 });

        transport.Inbound.Enqueue(new WelcomeMessage { Seat = Them, PlayerNames = ["Bot 1", "Bot 2", "Bot 3"] });
        transport.Inbound.Enqueue(new StateMessage { View = view });
        client.Pump();

        return client;
    }
}
