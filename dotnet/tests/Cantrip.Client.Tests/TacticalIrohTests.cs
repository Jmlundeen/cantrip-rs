using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Iroh, Tea Master's "you may have target opponent gain control of target permanent you control" ("GiveControl"): the prompt's
/// source is Iroh the permanent, and what the ability targets is read from the ability on the stack, which names its source card.
/// </summary>
public sealed class TacticalIrohTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Permanent(int handle, string name, ObjectKindView kind, CardTypeView type) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = kind,
        Owner = Me,
        Controller = Me,
        Name = name,
        CardTypes = [type],
    };

    private static ObjectView CombatAbility(int handle, string source, int target, int controller = Me) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Stack,
        Kind = ObjectKindView.AbilityOnStack,
        Owner = controller,
        Controller = controller,
        Name = source,
        Targets = [new SpellTargetView { Object = Handle(target), StillLegal = true }],
    };

    private static bool Accepts(params ObjectView[] objects) =>
        Assert.IsType<MayChosenAnswer>(Bot().Decide(Seated(objects), new MayPromptMessage { Player = Me, Source = Handle(1), Verb = "GiveControl" })).Accept;

    [Fact]
    public void Gives_a_targeted_noncreature_token_away_and_keeps_anything_else()
    {
        ObjectView iroh = Creature(1, "Iroh, Tea Master", 2, 2);
        ObjectView food = Permanent(2, "Food", ObjectKindView.Token, CardTypeView.Artifact);
        ObjectView ally = Creature(3, "Ally", 1, 1);
        ally.Kind = ObjectKindView.Token;
        ObjectView relic = Permanent(4, "Sol Ring", ObjectKindView.Card, CardTypeView.Artifact);

        Assert.True(Accepts(iroh, food, CombatAbility(10, "Iroh, Tea Master", 2)));
        Assert.False(Accepts(iroh, ally, CombatAbility(10, "Iroh, Tea Master", 3)));
        Assert.False(Accepts(iroh, relic, CombatAbility(10, "Iroh, Tea Master", 4)));
        Assert.False(Accepts(iroh, iroh, CombatAbility(10, "Iroh, Tea Master", 1)));
    }

    [Fact]
    public void Reads_only_the_ability_of_Iroh_this_seat_controls()
    {
        ObjectView iroh = Creature(1, "Iroh, Tea Master", 2, 2);
        ObjectView food = Permanent(2, "Food", ObjectKindView.Token, CardTypeView.Artifact);

        // No ability on the stack, another card's ability, or the opponent's: nothing is given away.
        Assert.False(Accepts(iroh, food));
        Assert.False(Accepts(iroh, food, CombatAbility(10, "Appa, Steadfast Guardian", 2)));
        Assert.False(Accepts(iroh, food, CombatAbility(10, "Iroh, Tea Master", 2, Them)));
    }
}
