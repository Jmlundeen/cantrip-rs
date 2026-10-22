using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Creatures cheated onto the battlefield (<c>policy.cheat</c>): from the hand, from a graveyard, and into a graveyard
/// to be brought back. Each test contrasts the seat with the weight at zero, which read these cards as nothing.
/// </summary>
public sealed class TacticalCheatsTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static TacticalPlay Without() =>
        new(TacticalModel.Prior().With([KeyValuePair.Create("policy.cheat", 0.0)]), seed: 1);

    private static PriorityPromptMessage Priority(params ActionOptionMessage[] options) => new() { Player = Me, Options = [.. options] };

    private static ActionOptionMessage Cast(int source, params TargetMessage[] targets) => new()
    {
        Kind = ActionKindView.CastSpell,
        Source = Handle(source),
        Affordable = true,
        TargetsRequired = targets.Length > 0 ? 1 : 0,
        TargetCandidates = [.. targets],
    };

    private const string Sneak =
        "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice the creature at the beginning of the next end step.";

    private const string Breach =
        "You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice that creature at the beginning of the next end step.";

    private static ObjectView Griselbrand(int handle, ZoneKindView zone = ZoneKindView.Hand, int owner = Me)
    {
        ObjectView card = InHand(handle, "Griselbrand", CardTypeView.Creature, 8, "Flying, lifelink", power: 7, toughness: 7, color: ColorView.Black);
        card.Zone = zone;
        card.ZoneOwner = owner;
        card.Owner = owner;
        card.Controller = owner;
        return card;
    }

    private static List<ObjectView> Lands(int count) => [.. Enumerable.Range(1, count).Select(i => Land(i))];

    [Fact]
    public void Sneaks_in_the_creature_it_cannot_cast_before_combat_on_its_own_turn()
    {
        ObjectView sneak = new()
        {
            Handle = Handle(30), Zone = ZoneKindView.Battlefield, Kind = ObjectKindView.Card, Owner = Me, Controller = Me,
            Name = "Sneak Attack", CardTypes = [CardTypeView.Enchantment], OracleText = Sneak,
        };
        var activate = new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility, Source = Handle(30), Affordable = true, Description = Sneak,
        };

        GameClient mine = Seated([.. Lands(4), sneak, Griselbrand(10)]);
        Assert.Equal(Handle(30), Assert.IsType<TakeActionAnswer>(Bot().Decide(mine, Priority(activate))).Source);
        Assert.IsType<PassAnswer>(Without().Decide(mine, Priority(activate)));

        // After combat it would only be sacrificed.
        GameClient late = Seated([.. Lands(4), sneak, Griselbrand(10)], step: TurnStepView.PostcombatMain);
        Assert.IsType<PassAnswer>(Bot().Decide(late, Priority(activate)));
    }

    [Fact]
    public void Casts_Through_the_Breach_for_the_fatty_in_hand_and_not_with_none()
    {
        ObjectView breach = InHand(20, "Through the Breach", CardTypeView.Instant, 5, Breach);

        // Two cards for one hasty attack: worth it when the attack is most of the game.
        GameClient fatty = Seated([.. Lands(5), breach, Griselbrand(10)], theirLife: 9);
        Assert.Equal(Handle(20), Assert.IsType<TakeActionAnswer>(Bot().Decide(fatty, Priority(Cast(20)))).Source);

        GameClient none = Seated([.. Lands(5), breach, InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)], theirLife: 9);
        Assert.IsType<PassAnswer>(Bot().Decide(none, Priority(Cast(20))));
    }

    [Fact]
    public void Puts_the_creature_worth_most_onto_the_battlefield_whatever_it_costs()
    {
        GameClient client = Seated([.. Lands(2), Griselbrand(10), InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)]);

        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Purpose = CardChoicePurposeView.PutOntoBattlefield,
            Cards = [Handle(10), Handle(11)],
            Verdict = new() { Addable = [Handle(10), Handle(11)] },
            Count = 1,
            CountKind = CountKindView.UpTo,
        };

        Assert.Equal([Handle(10)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards);
    }

    [Fact]
    public void Exhumes_its_own_fatty_unless_the_opponent_gets_a_bigger_one()
    {
        ObjectView exhume = InHand(20, "Exhume", CardTypeView.Sorcery, 2, "Each player puts a creature card from their graveyard onto the battlefield.");

        GameClient ahead = Seated([.. Lands(2), exhume, Griselbrand(10, ZoneKindView.Graveyard)]);
        Assert.Equal(Handle(20), Assert.IsType<TakeActionAnswer>(Bot().Decide(ahead, Priority(Cast(20)))).Source);

        ObjectView theirs = Griselbrand(11, ZoneKindView.Graveyard, Them);
        ObjectView bear = InHand(12, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2);
        bear.Zone = ZoneKindView.Graveyard;
        GameClient behind = Seated([.. Lands(2), exhume, bear, theirs]);
        Assert.IsType<PassAnswer>(Bot().Decide(behind, Priority(Cast(20))));
    }

    [Fact]
    public void Casts_Animate_Dead_at_the_fatty_in_a_graveyard()
    {
        ObjectView animate = InHand(20, "Animate Dead", CardTypeView.Enchantment, 2,
            "Enchant creature card in a graveyard\nWhen this Aura enters, if it's on the battlefield, it loses \"enchant creature card in a graveyard\". Return enchanted creature card to the battlefield under your control and attach this Aura to it.\nEnchanted creature gets -1/-0.");
        animate.Subtypes = ["Aura"];

        ObjectView bear = InHand(12, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2);
        bear.Zone = ZoneKindView.Graveyard;
        GameClient client = Seated([.. Lands(2), animate, bear, Griselbrand(10, ZoneKindView.Graveyard, Them)]);

        TakeActionAnswer cast = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(20, At(12), At(10)))));
        Assert.Equal([Handle(10)], cast.Targets.Select(target => target.Object));

        Assert.IsType<PassAnswer>(Without().Decide(client, Priority(Cast(20, At(12), At(10)))));
    }

    [Fact]
    public void Flash_puts_in_the_creature_it_can_pay_for_not_the_biggest()
    {
        const string Flash = "You may put a creature card from your hand onto the battlefield. If you do, sacrifice it unless you pay its mana cost reduced by {2}.";
        ObjectView flash = InHand(20, "Flash", CardTypeView.Instant, 2, Flash, color: ColorView.Blue);
        flash.Zone = ZoneKindView.Stack;
        flash.ZoneOwner = 0;

        // Flash paid for with two of four lands; two are left for a four-drop's cost less two, not Griselbrand's six.
        ObjectView giant = InHand(11, "Giant", CardTypeView.Creature, 4, power: 4, toughness: 4);
        GameClient client = Seated([Land(1, tapped: true), Land(2, tapped: true), Land(3), Land(4), flash, giant, Griselbrand(10)]);

        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Source = Handle(20),
            Purpose = CardChoicePurposeView.PutOntoBattlefield,
            Cards = [Handle(10), Handle(11)],
            Verdict = new() { Addable = [Handle(10), Handle(11)] },
            Count = 1,
            CountKind = CountKindView.UpTo,
        };

        Assert.Equal([Handle(11)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards);
    }

    [Fact]
    public void Does_not_cast_Necromancy_with_flash_at_the_end_of_the_opponents_turn()
    {
        ObjectView necromancy = InHand(20, "Necromancy", CardTypeView.Enchantment, 3,
            "You may cast this spell as though it had flash. If you cast it any time a sorcery couldn't have been cast, the controller of the permanent it becomes sacrifices it at the beginning of the next cleanup step.\nWhen this enchantment enters, if it's on the battlefield, it becomes an Aura with \"enchant creature put onto the battlefield with Necromancy.\" Put target creature card from a graveyard onto the battlefield under your control and attach this enchantment to it.",
            color: ColorView.Black);

        GameClient mine = Seated([.. Lands(3), necromancy, Griselbrand(10, ZoneKindView.Graveyard)]);
        Assert.Equal(Handle(20), Assert.IsType<TakeActionAnswer>(Bot().Decide(mine, Priority(Cast(20)))).Source);

        GameClient theirs = Seated([.. Lands(3), necromancy, Griselbrand(10, ZoneKindView.Graveyard)], active: Them, step: TurnStepView.End);
        Assert.IsType<PassAnswer>(Bot().Decide(theirs, Priority(Cast(20))));
    }

    [Fact]
    public void Does_not_draw_with_a_Top_that_goes_back_on_the_library()
    {
        // Two seats that did this every turn played 1,100 turns with neither library ever running out.
        const string Draw = "{T}: Draw a card, then put Sensei's Divining Top on top of its owner's library.";
        ObjectView top = new()
        {
            Handle = Handle(30), Zone = ZoneKindView.Battlefield, Kind = ObjectKindView.Card, Owner = Me, Controller = Me,
            Name = "Sensei's Divining Top", CardTypes = [CardTypeView.Artifact], OracleText = "{1}: Look at the top three cards of your library, then put them back in any order.\n" + Draw,
        };

        GameClient client = Seated([.. Lands(2), top]);
        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility, Source = Handle(30), AbilityIndex = 1, Affordable = true, Description = Draw,
        })));
    }

    [Fact]
    public void Entombs_only_with_a_way_to_bring_it_back_in_hand()
    {
        const string Entomb = "Search your library for a card, put that card into your graveyard, then shuffle.";
        ObjectView entomb = InHand(20, "Entomb", CardTypeView.Instant, 1, Entomb, color: ColorView.Black);
        ObjectView reanimate = InHand(21, "Reanimate", CardTypeView.Sorcery, 1,
            "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to that card's mana value.", color: ColorView.Black);

        GameClient with = Seated([.. Lands(2), entomb, reanimate]);
        Assert.Equal(Handle(20), Assert.IsType<TakeActionAnswer>(Bot().Decide(with, Priority(Cast(20)))).Source);

        // Without one it is a spell the seat cannot read, cast only for spare mana: never ahead of a card it can.
        GameClient without = Seated([.. Lands(1), entomb, InHand(11, "Bear", CardTypeView.Creature, 1, power: 2, toughness: 2)]);
        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(Bot().Decide(without, Priority(Cast(20), Cast(11)))).Source);
    }
}
