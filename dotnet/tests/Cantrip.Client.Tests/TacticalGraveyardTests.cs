using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Graveyard hate used as that (<c>policy.graveyard</c>), and Mox Amber counted as mana only with a legend to name a
/// colour.
/// </summary>
public sealed class TacticalGraveyardTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static TacticalPlay Without(string policy) =>
        new(TacticalModel.Prior().With([KeyValuePair.Create("policy." + policy, 0.0)]), seed: 1);

    private const string LanternText = "When this artifact enters, exile target card from a graveyard.\n"
        + "{T}, Sacrifice this artifact: Exile each opponent's graveyard.\n{1}, {T}, Sacrifice this artifact: Draw a card.";

    private static ObjectView Lantern(int handle) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Soul-Guide Lantern",
        ManaValue = 1,
        OracleText = LanternText,
        CardTypes = [CardTypeView.Artifact],
    };

    private static ObjectView Dead(int handle, string name, int manaValue, int power, int toughness, int owner = Them) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Graveyard,
        ZoneOwner = owner,
        Kind = ObjectKindView.Card,
        Owner = owner,
        Controller = owner,
        Name = name,
        ManaValue = manaValue,
        CardTypes = [CardTypeView.Creature],
        Power = power,
        Toughness = toughness,
    };

    private static ObjectView Reanimate(int handle, int target) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Stack,
        Kind = ObjectKindView.Card,
        Owner = Them,
        Controller = Them,
        Name = "Reanimate",
        ManaValue = 1,
        OracleText = "Put target creature card from a graveyard onto the battlefield under your control. You lose life equal to its mana value.",
        CardTypes = [CardTypeView.Sorcery],
        Targets = [new SpellTargetView { Object = Handle(target), StillLegal = true }],
    };

    private static ActionOptionMessage Exile(int lantern) => new()
    {
        Kind = ActionKindView.ActivateAbility,
        Source = Handle(lantern),
        AbilityIndex = 1,
        Affordable = true,
        Description = "{T}, Sacrifice this artifact: Exile each opponent's graveyard.",
    };

    private static ActionOptionMessage Cash(int lantern) => new()
    {
        Kind = ActionKindView.ActivateAbility,
        Source = Handle(lantern),
        AbilityIndex = 2,
        Affordable = true,
        Description = "{1}, {T}, Sacrifice this artifact: Draw a card.",
    };

    [Fact]
    public void Exiles_their_graveyard_in_answer_to_a_reanimation()
    {
        GameClient client = Seated([Land(1), Lantern(10), Dead(20, "Griselbrand", 8, 7, 7), Reanimate(30, 20)], active: Them);
        var prompt = new PriorityPromptMessage { Player = Me, Options = [Exile(10), Cash(10)] };

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, prompt));
        Assert.Equal(Handle(10), answer.Source);
        Assert.Equal(1, answer.AbilityIndex);

        Assert.IsType<PassAnswer>(Without("graveyard").Decide(client, prompt));
    }

    [Fact]
    public void Answers_Exhume_and_aims_a_targeted_exile_at_the_card_coming_back_but_not_a_return_to_hand()
    {
        ObjectView exhume = Reanimate(30, 20);
        exhume.Name = "Exhume";
        exhume.OracleText = "Each player puts a creature card from their graveyard onto the battlefield.";
        exhume.Targets = [];
        GameClient exhumed = Seated([Land(1), Lantern(10), Dead(20, "Griselbrand", 8, 7, 7), exhume], active: Them);
        Assert.Equal(1, Assert.IsType<TakeActionAnswer>(Bot().Decide(exhumed, new PriorityPromptMessage { Player = Me, Options = [Exile(10), Cash(10)] })).AbilityIndex);

        ObjectView vacuum = Lantern(11);
        vacuum.Name = "Ghost Vacuum";
        vacuum.OracleText = "{T}: Exile target card from a graveyard.";
        var aimed = new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility, Source = Handle(11), Affordable = true,
            Description = "{T}: Exile target card from a graveyard.", TargetsRequired = 1, TargetCandidates = [At(21), At(20)],
        };
        GameClient raised = Seated([Land(1), vacuum, Dead(20, "Griselbrand", 8, 7, 7), Dead(21, "Bear", 2, 2, 2), Reanimate(30, 20)], active: Them);
        TakeActionAnswer vacuumed = Assert.IsType<TakeActionAnswer>(Bot().Decide(raised, new PriorityPromptMessage { Player = Me, Options = [aimed] }));
        Assert.Equal(Handle(20), Assert.Single(vacuumed.Targets).Object);

        // Raise Dead brings it to a hand, not the battlefield: nothing worth a Lantern.
        ObjectView raiseDead = Reanimate(30, 20);
        raiseDead.Name = "Raise Dead";
        raiseDead.OracleText = "Return target creature card from your graveyard to your hand.";
        GameClient toHand = Seated([Land(1), Lantern(10), Dead(20, "Griselbrand", 8, 7, 7), raiseDead], active: Them);
        Assert.IsType<PassAnswer>(Bot().Decide(toHand, new PriorityPromptMessage { Player = Me, Options = [Exile(10)] }));
    }

    [Fact]
    public void Keeps_the_Lantern_while_their_graveyard_holds_something_worth_bringing_back()
    {
        var prompt = new PriorityPromptMessage { Player = Me, Options = [Exile(10), Cash(10)] };

        GameClient fat = Seated([Land(1), Land(2), Lantern(10), Dead(20, "Griselbrand", 8, 7, 7)]);
        Assert.IsType<PassAnswer>(Bot().Decide(fat, prompt));
        Assert.Equal(2, Assert.IsType<TakeActionAnswer>(Without("graveyard").Decide(fat, prompt)).AbilityIndex);

        GameClient lean = Seated([Land(1), Land(2), Lantern(10), Dead(20, "Bear", 2, 2, 2)]);
        TakeActionAnswer cashed = Assert.IsType<TakeActionAnswer>(Bot().Decide(lean, prompt));
        Assert.Equal(2, cashed.AbilityIndex);
    }

    [Fact]
    public void Aims_the_Lantern_at_their_creature_card_not_its_own()
    {
        GameClient client = Seated([Land(1), Lantern(10), Dead(20, "Griselbrand", 8, 7, 7), Dead(21, "Bear", 2, 2, 2, owner: Me),
            Dead(22, "Shock", 1, 0, 0)]);
        client.View!.Objects.Find(card => card.Name == "Shock")!.CardTypes = [CardTypeView.Instant];

        var prompt = new ChooseTargetPromptMessage
        {
            Player = Me,
            Source = Handle(10),
            Candidates = [At(21), At(22), At(20)],
            Count = 1,
        };

        TargetsChosenAnswer chosen = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, prompt));
        Assert.Equal(Handle(20), Assert.Single(chosen.Targets).Object);
        Assert.NotEqual(Handle(20), Assert.Single(Assert.IsType<TargetsChosenAnswer>(Without("graveyard").Decide(client, prompt)).Targets).Object);
    }

    [Fact]
    public void Counts_Mox_Amber_as_mana_only_with_a_legend_out()
    {
        const string AmberText = "{T}: Add one mana of any color among legendary creatures and planeswalkers you control.";
        ObjectView amber = new()
        {
            Handle = Handle(10),
            Zone = ZoneKindView.Battlefield,
            Kind = ObjectKindView.Card,
            Owner = Me,
            Controller = Me,
            Name = "Mox Amber",
            OracleText = AmberText,
            CardTypes = [CardTypeView.Artifact],
            Supertypes = [SupertypeView.Legendary],
            ProducesMana = true,
        };

        ObjectView giant = InHand(11, "Hill Giant", CardTypeView.Creature, 2, power: 3, toughness: 3);
        var prompt = new PriorityPromptMessage { Player = Me, Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(11), Affordable = true }] };

        static string Told(GameClient client, PriorityPromptMessage prompt)
        {
            var told = new StringWriter();
            new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told }.Decide(client, prompt);
            return told.ToString();
        }

        Assert.Contains("weighed (mana 1,", Told(Seated([Land(1), amber, giant]), prompt), StringComparison.Ordinal);

        // A legendary artifact is no legend for it (the Mox is one itself).
        ObjectView relic = InHand(13, "Relic", CardTypeView.Artifact, 3);
        relic.Zone = ZoneKindView.Battlefield;
        relic.Supertypes = [SupertypeView.Legendary];
        Assert.Contains("weighed (mana 1,", Told(Seated([Land(1), amber, relic, giant]), prompt), StringComparison.Ordinal);

        ObjectView legend = Creature(12, "Thalia", 2, 1);
        legend.Supertypes = [SupertypeView.Legendary];
        Assert.Contains("weighed (mana 2,", Told(Seated([Land(1), amber, legend, giant]), prompt), StringComparison.Ordinal);
    }

    [Fact]
    public void Sacrifices_three_permanents_for_Scavengers_Talent_only_for_a_creature_card_worth_more_than_they_are()
    {
        // Scavenger's Talent, level 3 ("SacrificeThreeToReturn"): three other nonland permanents given up for a creature card
        // back from the graveyard. It is accepted when that card is worth more on the battlefield than the three cheapest
        // permanents put together, and never for a graveyard with no creature card in it (the generic "Sacrifice" is refused).
        ObjectView talent = Creature(1, "Scavenger's Talent", 0, 0);
        talent.CardTypes = [CardTypeView.Enchantment];
        MayPromptMessage prompt = new() { Player = Me, Source = Handle(1), Verb = "SacrificeThreeToReturn" };

        ObjectView[] fodder = [Creature(2, "Bear", 1, 1), Creature(3, "Bear", 1, 1), Creature(4, "Bear", 1, 1)];
        ObjectView bigDead = Dead(10, "Hill Giant", 3, 7, 7, Me);
        ObjectView smallDead = Dead(11, "Bear", 2, 1, 1, Me);

        Assert.True(Assert.IsType<MayChosenAnswer>(Bot().Decide(Seated([talent, .. fodder, bigDead]), prompt)).Accept);
        Assert.False(Assert.IsType<MayChosenAnswer>(Bot().Decide(Seated([talent, .. fodder, smallDead]), prompt)).Accept);
        Assert.False(Assert.IsType<MayChosenAnswer>(Bot().Decide(Seated([talent, .. fodder]), prompt)).Accept);
        Assert.False(Assert.IsType<MayChosenAnswer>(Bot().Decide(Seated([talent, .. fodder, bigDead]), new MayPromptMessage { Player = Me, Source = Handle(1), Verb = "Sacrifice" })).Accept);
    }
}
