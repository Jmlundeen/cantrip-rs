using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>
/// Which priority prompts the table answers for the player and which it leaves them.
/// <para>
/// Worth a test rather than an eye because the interesting cases are sequences, not positions: the same board —
/// the player's own spell alone on the stack — is passed the first time it is seen and passed again after an
/// opponent's answer has resolved off the top of it, and in between the player must be asked. Only the run of
/// prompts shows that, and a run of prompts is what a player complains about.
/// </para>
/// </summary>
public sealed class AutoPassTests
{
    private const int Me = 1;
    private const int Them = 2;

    [Fact]
    public void My_own_spell_on_top_of_the_stack_is_passed_rather_than_asked_about()
    {
        // Casting is how almost everyone means "and now over to you".
        Assert.True(AutoPass.Wanted(Board(Mine)));
    }

    [Fact]
    public void An_opponents_spell_on_top_of_mine_is_always_asked_about()
    {
        Assert.False(AutoPass.Wanted(Board(Mine, Theirs)));
    }

    [Fact]
    public void My_spell_is_not_asked_about_again_once_the_answer_to_it_has_resolved()
    {
        // The note this rule comes from, in order: I cast, they respond, their spell resolves, mine is on top again.
        // The middle prompt is mine to use; the first and the last are the same question about the same spell of my
        // own, and being asked it twice is the complaint.
        PriorityBoard cast = Board(Mine);
        PriorityBoard responded = Board(Mine, Theirs);
        PriorityBoard resolved = Board(Mine);

        Assert.True(AutoPass.Wanted(cast));
        Assert.False(AutoPass.Wanted(responded));
        Assert.True(AutoPass.Wanted(resolved));
    }

    [Fact]
    public void Holding_priority_keeps_me_asked_about_my_own_spell_every_time()
    {
        // The whole point of the box, and it holds for as long as the spell is on the stack rather than for one
        // prompt, so it can be ticked mid-flight and is still obeyed on the way back down.
        Assert.False(AutoPass.Wanted(Board(Mine) with { HoldingPriority = true }));
        Assert.False(AutoPass.Wanted(Board(Mine, Theirs) with { HoldingPriority = true }));
        Assert.False(AutoPass.Wanted(Board(Mine) with { HoldingPriority = true }));
    }

    [Fact]
    public void Holding_priority_keeps_me_asked_in_a_step_nobody_stops_in()
    {
        // Nothing stops in an opponent's turn by default, so a trick cast there had nothing but this rule to hold it.
        PriorityBoard trick = Board(Mine) with
        {
            MyTurn = false,
            Step = TurnStepView.DeclareBlockers,
            HoldingPriority = true,
        };

        Assert.False(AutoPass.Wanted(trick));
        Assert.True(AutoPass.Wanted(trick with { HoldingPriority = false }));
    }

    [Fact]
    public void Pass_turn_outranks_holding_priority_over_my_own_spell()
    {
        // Pass Turn is ticked by name and says "nothing more this turn", which is the later word of the two.
        Assert.True(AutoPass.Wanted(Board(Mine) with { HoldingPriority = true, PassTurn = true }));
    }

    [Fact]
    public void Holding_priority_says_nothing_about_a_stack_with_nothing_of_mine_on_top()
    {
        // It is about keeping priority after putting something on the stack, so with nothing of mine on top there is
        // nothing for it to keep: an empty stack in a step I have not stopped in is still passed.
        Assert.True(AutoPass.Wanted(Board() with { HoldingPriority = true }));
    }

    [Fact]
    public void My_own_spell_on_top_beats_a_step_I_asked_to_be_stopped_in()
    {
        // Where the rule sits: above the stops, as the cast-and-pass it replaces did. The stop is about arriving in a
        // step, and it still stops me once the stack is empty again.
        Assert.True(AutoPass.Wanted(Board(Mine) with { StopsAtStep = true }));
        Assert.False(AutoPass.Wanted(Board() with { StopsAtStep = true }));
    }

    [Fact]
    public void Pass_turn_and_resolve_all_still_pass_through_an_opponents_spell()
    {
        // Ticked by name, knowing what was on the stack.
        Assert.True(AutoPass.Wanted(Board(Mine, Theirs) with { PassTurn = true }));
        Assert.True(AutoPass.Wanted(Board(Mine, Theirs) with { ResolveAll = true }));
    }

    [Fact]
    public void Auto_pass_still_passes_an_opponents_spell_I_could_not_answer()
    {
        // Auto Pass is "nothing I could do", which is as true of a response window as of any other.
        Assert.True(AutoPass.Wanted(Board(Mine, Theirs) with { AutoPass = true }));
        Assert.False(AutoPass.Wanted(Board(Mine, Theirs) with { AutoPass = true, Prompt = Castable() }));
    }

    [Fact]
    public void Being_attacked_is_still_worth_a_look_whatever_the_stops_say()
    {
        PriorityBoard attacked = Board() with
        {
            MyTurn = false,
            Step = TurnStepView.DeclareBlockers,
            Attacked = true,
        };

        Assert.False(AutoPass.Wanted(attacked));
        Assert.True(AutoPass.Wanted(attacked with { Attacked = false }));
    }

    [Fact]
    public void A_trigger_of_mine_waiting_under_an_opponents_answer_is_no_different_from_a_spell()
    {
        // The rule reads the controller of what is on top, not what this client last sent, so a triggered ability
        // this player controls counts as theirs however it got there.
        Assert.True(AutoPass.Wanted(Board(
            new ObjectView { Controller = Me, Kind = ObjectKindView.AbilityOnStack, Zone = ZoneKindView.Stack })));
    }

    private static ObjectView Mine => new() { Controller = Me, Kind = ObjectKindView.Card, Zone = ZoneKindView.Stack };

    private static ObjectView Theirs => new() { Controller = Them, Kind = ObjectKindView.Card, Zone = ZoneKindView.Stack };

    /// <summary>A quiet main phase with the given stack, bottom first, and every dock box clear.</summary>
    private static PriorityBoard Board(params ObjectView[] stack) => new()
    {
        Prompt = new PriorityPromptMessage { Player = Me },
        Stack = stack,
        Seat = Me,
        MyTurn = true,
        Step = TurnStepView.PrecombatMain,
        Attacked = false,
        HoldingPriority = false,
        PassTurn = false,
        ResolveAll = false,
        StopsAtStep = false,
        AutoPass = false,
    };

    /// <summary>A priority prompt offering something the player could actually pay for.</summary>
    private static PriorityPromptMessage Castable() => new()
    {
        Player = Me,
        Options = [new ActionOptionMessage { Kind = ActionKindView.CastSpell, Affordable = true }],
    };
}
