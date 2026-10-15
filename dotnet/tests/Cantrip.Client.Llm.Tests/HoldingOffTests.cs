using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Llm.Tests.Board;

namespace Cantrip.Client.Llm.Tests;

/// <summary>
/// Waiting for a named point in the game, and what brings the seat back early.
/// <para>
/// This closes a leak rather than saving money, though it does both. A seat is asked at priority exactly when it
/// has something it could cast, so a seat that stops to think at every phase of a quiet turn has told the player
/// across the table that it is holding an instant — information the board does not show and the rules do not give
/// away. One pause and then silence says nothing at all.
/// </para>
/// <para>
/// Points rather than a single "I am done", because keeping mana up for their end step and keeping it up for their
/// upkeep are different plans and a seat should be able to say which.
/// </para>
/// </summary>
public sealed class HoldingOffTests
{
    [Fact]
    public void Offers_the_stops_a_player_would_name_out_loud()
    {
        var chooser = new ScriptedChooser([1]);

        new Decisions(chooser).Answer(OnMyTurn(), Instant());

        IReadOnlyList<string> choices = chooser.Asked[0].Choices;

        // The spell, passing for now, and three points to wait for: my end step, their upkeep, their end step.
        Assert.Equal(5, choices.Count);
        Assert.Contains("your own end step", choices[2], StringComparison.Ordinal);
        Assert.Contains("their upkeep", choices[3], StringComparison.Ordinal);
        Assert.Contains("their end step", choices[4], StringComparison.Ordinal);
    }

    [Fact]
    public void Never_offers_to_wait_for_a_point_already_gone_by()
    {
        // In an end step there is no end step left to wait for, and a seat that waited for one would sit out until
        // the turn after next.
        var chooser = new ScriptedChooser([1]);
        GameClient client = OnMyTurn();
        client.View!.Step = TurnStepView.End;

        new Decisions(chooser).Answer(client, Instant());

        Assert.DoesNotContain(chooser.Asked[0].Choices, choice => choice.Contains("own end step", StringComparison.Ordinal));
    }

    [Fact]
    public void Never_offers_to_wait_out_a_turn_of_its_own()
    {
        // From the opponent's turn, "next turn's end step" is this seat's own. Waiting for it would give up a whole
        // turn to keep mana up.
        var chooser = new ScriptedChooser([1]);

        new Decisions(chooser).Answer(OnTheirTurn(), Instant());

        IReadOnlyList<string> choices = chooser.Asked[0].Choices;
        Assert.Equal(4, choices.Count);
        Assert.Contains("their end step", choices[2], StringComparison.Ordinal);
        Assert.Contains("your own upkeep", choices[3], StringComparison.Ordinal);
    }

    [Fact]
    public void Passes_unasked_until_the_point_it_named()
    {
        // Waiting for their end step, from their precombat main.
        var chooser = new ScriptedChooser([2], [1]);
        var decisions = new Decisions(chooser);
        GameClient client = OnTheirTurn();

        Assert.IsType<PassAnswer>(decisions.Answer(client, Instant()));
        Assert.Single(chooser.Asked);

        foreach (TurnStepView step in (TurnStepView[])
            [TurnStepView.BeginCombat, TurnStepView.DeclareBlockers, TurnStepView.PostcombatMain])
        {
            client.View!.Step = step;
            Assert.IsType<PassAnswer>(decisions.Answer(client, Instant()));
        }

        Assert.Single(chooser.Asked);

        // And there it is: the end step, and it is asked again.
        client.View!.Step = TurnStepView.End;
        decisions.Answer(client, Instant());
        Assert.Equal(2, chooser.Asked.Count);
    }

    [Fact]
    public void Waits_through_a_whole_turn_when_that_is_what_it_asked_for()
    {
        // Their upkeep, named from this seat's own precombat main: the rest of its turn goes by unasked.
        var chooser = new ScriptedChooser([3], [1]);
        var decisions = new Decisions(chooser);
        GameClient client = OnMyTurn();

        decisions.Answer(client, Instant());

        client.View!.Step = TurnStepView.End;
        Assert.IsType<PassAnswer>(decisions.Answer(client, Instant()));
        Assert.Single(chooser.Asked);

        client.View!.Turn = 4;
        client.View!.Step = TurnStepView.Upkeep;
        decisions.Answer(client, Instant());
        Assert.Equal(2, chooser.Asked.Count);
    }

    [Fact]
    public void Is_asked_again_the_moment_something_happens()
    {
        // The whole point of waiting is that it is not a commitment. A spell cast at it, or a creature dying, is
        // exactly when a held instant is worth casting.
        var chooser = new ScriptedChooser([4], [0]);
        var decisions = new Decisions(chooser);
        GameClient client = OnMyTurn();

        decisions.Answer(client, Instant());
        Assert.Single(chooser.Asked);

        decisions.Stir();

        Assert.IsType<TakeActionAnswer>(decisions.Answer(client, Instant()));
        Assert.Equal(2, chooser.Asked.Count);
    }

    [Fact]
    public void Never_offers_to_wait_with_a_land_still_to_play()
    {
        // A land is a free once-a-turn resource. A seat that gave one up by waiting would be throwing away a land
        // drop for a reason that had nothing to do with land.
        var chooser = new ScriptedChooser([1]);

        new Decisions(chooser).Answer(
            OnMyTurn(),
            new PriorityPromptMessage
            {
                Player = Me,
                Options = [new() { Kind = ActionKindView.PlayLand, Source = Handle(3) }],
            });

        Assert.Equal(2, chooser.Asked[0].Choices.Count);
        Assert.DoesNotContain(chooser.Asked[0].Choices, choice => choice.StartsWith("Wait", StringComparison.Ordinal));
    }

    [Fact]
    public void Passing_for_now_keeps_being_asked()
    {
        // The plain pass still means what it says: a seat that wants to see the next step before deciding has not
        // given anything up.
        var chooser = new ScriptedChooser([1], [1]);
        var decisions = new Decisions(chooser);
        GameClient client = OnMyTurn();

        Assert.IsType<PassAnswer>(decisions.Answer(client, Instant()));

        client.View!.Step = TurnStepView.PostcombatMain;
        Assert.IsType<PassAnswer>(decisions.Answer(client, Instant()));

        Assert.Equal(2, chooser.Asked.Count);
    }

    private static GameClient OnMyTurn() => Seated(Card(2, "Lightning Bolt", "{R}"), Card(3, "Mountain"));

    private static GameClient OnTheirTurn()
    {
        GameClient client = OnMyTurn();
        client.View!.ActivePlayer = Them;
        return client;
    }

    /// <summary>One castable instant, so the seat is asked and the waiting choices are offered.</summary>
    private static PriorityPromptMessage Instant() => new()
    {
        Player = Me,
        Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(2), Affordable = true }],
    };
}
