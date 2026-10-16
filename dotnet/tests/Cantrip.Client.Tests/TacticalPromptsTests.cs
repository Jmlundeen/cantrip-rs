using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the tactical seat answers to questions that were once read as something else: collecting evidence, the modes of a
/// "choose one or both", the faces a card may be cast as, and the targets of an ability that exiles a card to use it.
/// </summary>
public sealed class TacticalPromptsTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Dead(int handle, string name, int manaValue, int owner, bool creature = true, string text = "") => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Graveyard,
        ZoneOwner = owner,
        Kind = ObjectKindView.Card,
        Owner = owner,
        Controller = owner,
        Name = name,
        ManaValue = manaValue,
        OracleText = text,
        CardTypes = [creature ? CardTypeView.Creature : CardTypeView.Sorcery],
        Power = creature ? 2 : null,
        Toughness = creature ? 2 : null,
    };

    private static ChooseModesPromptMessage Evidence() => new()
    {
        Player = Me,
        Modes = ["Cast without collecting evidence", "Collect evidence 6"],
        Eligible = [0, 1],
        Count = 1,
        AdditionalCost = true,
    };

    // --- Collect evidence -------------------------------------------------------------------------------------------

    [Fact]
    public void Collects_evidence_for_a_bonus_that_takes_a_card_from_an_opponents_graveyard()
    {
        GameClient client = Seated([Dead(3, "Ornithopter", 0, Them), Dead(4, "Shock", 1, Me, creature: false)]);

        Assert.Equal([1], Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, Evidence())).Modes);
    }

    [Fact]
    public void Does_not_collect_evidence_with_nothing_in_an_opponents_graveyard_for_the_bonus_to_take()
    {
        GameClient client = Seated([Dead(4, "Shock", 1, Me, creature: false)]);

        Assert.Equal([0], Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, Evidence())).Modes);
    }

    [Fact]
    public void Keeps_a_creature_in_its_own_graveyard_rather_than_collect_evidence_with_it()
    {
        GameClient client = Seated([Dead(3, "Ornithopter", 0, Them), Dead(4, "Griselbrand", 8, Me)]);

        Assert.Equal([0], Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, Evidence())).Modes);
    }

    // --- Choose one or both -----------------------------------------------------------------------------------------

    private static ChooseModesPromptMessage OneOrBoth(bool extraModesCost) => new()
    {
        Player = Me,
        Modes = ["Lightning deals 3 damage to any target.", "Destroy target creature."],
        Eligible = [0, 1],
        Count = 1,
        AtMost = 2,
        ExtraModesCost = extraModesCost,
    };

    [Fact]
    public void Takes_both_modes_of_a_one_or_both_when_neither_costs_extra_and_both_are_worth_doing()
    {
        GameClient client = Seated([Creature(3, "Giant", 4, 3, Them)]);

        Assert.Equal(2, Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, OneOrBoth(extraModesCost: false))).Modes.Count);
    }

    [Fact]
    public void Takes_one_mode_when_the_ones_past_the_first_cost_something()
    {
        GameClient client = Seated([Creature(3, "Giant", 4, 3, Them)]);

        Assert.Single(Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, OneOrBoth(extraModesCost: true))).Modes);
    }

    // --- The faces a card may be cast as ----------------------------------------------------------------------------

    private static ChooseModesPromptMessage Faces() => new()
    {
        Player = Me,
        Heading = "Choose which face to cast",
        Modes = ["Stomp {1}{R}", "Bonecrusher Giant {2}{R}"],
        Texts = ["Stomp deals 2 damage to any target.", string.Empty],
        Eligible = [0, 1],
        Count = 1,
    };

    [Fact]
    public void Casts_the_face_that_removes_what_is_worth_removing_and_otherwise_the_creature()
    {
        GameClient prey = Seated([Creature(3, "Dragon", 6, 2, Them, "Flying")]);
        Assert.Equal([0], Assert.IsType<ModesChosenAnswer>(Bot().Decide(prey, Faces())).Modes);

        // Nothing for the damage to do but hit a player at twenty life: the body is worth more.
        GameClient nothing = Seated([]);
        Assert.Equal([1], Assert.IsType<ModesChosenAnswer>(Bot().Decide(nothing, Faces())).Modes);
    }

    // --- A card exiled to be used -----------------------------------------------------------------------------------

    [Fact]
    public void Aims_an_ability_that_exiles_a_card_of_its_own_graveyard_to_copy_it_at_a_card_worth_a_copy()
    {
        GameClient client = Seated([Dead(3, "Griselbrand", 8, Me), Land(9)]);

        TargetsChosenAnswer to = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, new ChooseTargetPromptMessage
        {
            Player = Me,
            Text = "Exile up to one target nonland card from your graveyard of a card type none of your other targets has, and copy it",
            Candidates = [At(3)],
            Optional = true,
            Count = 1,
        }));
        Assert.Equal(At(3).Object, Assert.Single(to.Targets).Object);

        // The same exile with no use made of the card is one given up for nothing.
        TargetsChosenAnswer away = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, new ChooseTargetPromptMessage
        {
            Player = Me,
            Text = "Exile up to one target card from your graveyard.",
            Candidates = [At(3)],
            Optional = true,
            Count = 1,
        }));
        Assert.Empty(away.Targets);
    }
}
