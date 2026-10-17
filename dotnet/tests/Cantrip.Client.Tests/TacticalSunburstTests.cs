using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Sunburst (CR 702.44a): a spell with it gets a counter for each color of mana spent to cast it, so the tactical seat pays
/// its generic mana with a color not yet spent on it - the pool's, or a source's - rather than another of one already spent.
/// </summary>
public sealed class TacticalSunburstTests
{
    private const string ExplosivesText =
        "Sunburst (This artifact enters with a charge counter on it for each color of mana spent to cast it.)\n{2}, Sacrifice this artifact: Destroy each nonland permanent with mana value equal to the number of charge counters on this artifact.";

    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Explosives() => new()
    {
        Handle = Handle(1),
        Zone = ZoneKindView.Stack,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Engineered Explosives",
        CardTypes = [CardTypeView.Artifact],
        OracleText = ExplosivesText,
    };

    private static ObjectView Mox(int handle, string name) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = name,
        CardTypes = [CardTypeView.Artifact],
        ProducesMana = true,
    };

    private static ActionOptionMessage Tap(int handle, ManaTypeView adds) =>
        new() { Kind = ActionKindView.ActivateAbility, Source = Handle(handle), Produces = [adds] };

    private static PayCostsPromptMessage Paying(int generic, List<RestrictedManaView> committed, List<ActionOptionMessage> options) => new()
    {
        Player = Me,
        Spell = Handle(1),
        HasX = true,
        X = 2,
        Total = "{2}",
        Remaining = $"{{{generic}}}",
        RemainingGeneric = generic,
        Committed = committed,
        Options = [.. options, new() { Kind = ActionKindView.CancelCasting }],
    };

    [Fact]
    public void A_source_of_a_color_not_yet_spent_pays_before_another_of_one_already_spent()
    {
        GameClient client = Seated([Explosives(), Mox(10, "Mox Ruby"), Mox(11, "Mox Jet")]);
        PayCostsPromptMessage prompt = Paying(
            1,
            [new() { Type = ManaTypeView.Red, Amount = 1 }],
            [Tap(10, ManaTypeView.Red), Tap(11, ManaTypeView.Black)]);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, prompt));

        Assert.Equal(ActionKindView.ActivateAbility, answer.Kind);
        Assert.Equal(Handle(11), answer.Source);
    }

    [Fact]
    public void Pool_mana_of_a_new_color_is_put_towards_it_before_colorless()
    {
        GameClient client = Seated([Explosives()]);
        PayCostsPromptMessage prompt = Paying(
            1,
            [new() { Type = ManaTypeView.Red, Amount = 1 }],
            [
                new() { Kind = ActionKindView.AssignMana, Produces = [ManaTypeView.Colorless] },
                new() { Kind = ActionKindView.AssignMana, Produces = [ManaTypeView.Red] },
                new() { Kind = ActionKindView.AssignMana, Produces = [ManaTypeView.Green] },
            ]);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, prompt));

        Assert.Equal(ActionKindView.AssignMana, answer.Kind);
        Assert.Equal(ManaTypeView.Green, answer.Mana);
    }
}
