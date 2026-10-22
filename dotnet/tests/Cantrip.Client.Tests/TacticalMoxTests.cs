using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Mana paid for with a card (<c>policy.imprint</c>), a Vehicle that holds cards (Reckoner Bankbuster, <c>policy.stored</c>),
/// and what a cheated Griselbrand does at once (<c>policy.cheat</c>).
/// </summary>
public sealed class TacticalMoxTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static TacticalPlay Without(string policy) =>
        new(TacticalModel.Prior().With([KeyValuePair.Create("policy." + policy, 0.0)]), seed: 1);

    private static PriorityPromptMessage Priority(params ActionOptionMessage[] options) => new() { Player = Me, Options = [.. options] };

    private static ActionOptionMessage Cast(int source) => new() { Kind = ActionKindView.CastSpell, Source = Handle(source), Affordable = true };

    private const string Chrome = "Imprint — When this artifact enters, you may exile a nonartifact, nonland card from your hand.\n{T}: Add one mana of any of the exiled card's colors.";
    private const string Diamond = "If this artifact would enter, you may discard a land card instead. If you do, put this artifact onto the battlefield. If you don't, put it into its owner's graveyard.\n{T}: Add one mana of any color.";

    private static ObjectView Spell(int handle, string name, int manaValue) => InHand(handle, name, CardTypeView.Sorcery, manaValue, "Draw two cards.");

    [Fact]
    public void Casts_Chrome_Mox_early_with_a_spell_to_spare_and_imprints_the_one_wanted_least()
    {
        ObjectView mox = InHand(10, "Chrome Mox", CardTypeView.Artifact, 0, Chrome);

        GameClient spare = Seated([Land(1), mox, Spell(11, "Divination", 3), InHand(12, "Giant", CardTypeView.Creature, 3, power: 4, toughness: 4)]);
        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(Bot().Decide(spare, Priority(Cast(10)))).Source);
        Assert.IsType<PassAnswer>(Without("imprint").Decide(spare, Priority(Cast(10))));

        // Its last spell is not given up for a mana.
        GameClient last = Seated([Land(1), mox, Spell(11, "Divination", 3)]);
        Assert.IsType<PassAnswer>(Bot().Decide(last, Priority(Cast(10))));

        // Late, two cards for a mana is a card thrown away.
        GameClient late = Seated([Land(1), Land(2), Land(3), Land(4), Land(5), mox, Spell(11, "Divination", 3), Spell(12, "Concentrate", 4)]);
        Assert.IsType<PassAnswer>(Bot().Decide(late, Priority(Cast(10))));

        ObjectView onTable = InHand(10, "Chrome Mox", CardTypeView.Artifact, 0, Chrome);
        onTable.Zone = ZoneKindView.Battlefield;
        GameClient imprinting = Seated([Land(1), onTable, Spell(11, "Divination", 3), InHand(12, "Giant", CardTypeView.Creature, 3, power: 4, toughness: 4)]);
        var imprint = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Source = Handle(10),
            Purpose = CardChoicePurposeView.Exile,
            Cards = [Handle(11), Handle(12)],
            Verdict = new() { Addable = [Handle(11), Handle(12)] },
            Count = 1,
            CountKind = CountKindView.UpTo,
        };

        Assert.Single(Assert.IsType<CardsChosenAnswer>(Bot().Decide(imprinting, imprint)).Cards);
        Assert.Empty(Assert.IsType<CardsChosenAnswer>(Without("imprint").Decide(imprinting, imprint)).Cards);
    }

    [Fact]
    public void Casts_Jace_the_Mind_Sculptor_ahead_of_a_four_mana_creature()
    {
        ObjectView jace = InHand(10, "Jace, the Mind Sculptor", CardTypeView.Planeswalker, 4,
            "+2: Look at the top card of target player's library. You may put that card on the bottom of that player's library.\n0: Draw three cards, then put two cards from your hand on top of your library in any order.\n−1: Return target creature to its owner's hand.\n−12: Exile all cards from target player's library, then that player shuffles their hand into their library.",
            color: ColorView.Blue);
        GameClient client = Seated([Land(1), Land(2), Land(3), Land(4), jace, InHand(11, "Ogre", CardTypeView.Creature, 4, power: 3, toughness: 3)]);

        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10), Cast(11)))).Source);
        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(Without("walkercast").Decide(client, Priority(Cast(10), Cast(11)))).Source);
    }

    [Fact]
    public void Casts_Mox_Diamond_with_a_second_land_in_hand()
    {
        ObjectView mox = InHand(10, "Mox Diamond", CardTypeView.Artifact, 0, Diamond);

        GameClient two = Seated([Land(1), mox, InHand(11, "Mountain", CardTypeView.Land, 0), InHand(12, "Forest", CardTypeView.Land, 0)]);
        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(Bot().Decide(two, Priority(Cast(10)))).Source);

        GameClient one = Seated([Land(1), mox, InHand(11, "Mountain", CardTypeView.Land, 0)]);
        Assert.IsType<PassAnswer>(Bot().Decide(one, Priority(Cast(10))));
    }

    [Fact]
    public void Casts_Reckoner_Bankbuster_for_the_cards_it_holds()
    {
        const string Bankbuster = "This Vehicle enters with three charge counters on it.\n{2}, {T}, Remove a charge counter from this Vehicle: Draw a card. Then if there are no charge counters on this Vehicle, create a Treasure token and a 1/1 colorless Pilot creature token with \"This token crews Vehicles as though its power were 2 greater.\"\nCrew 3";
        ObjectView card = InHand(10, "Reckoner Bankbuster", CardTypeView.Artifact, 2, Bankbuster, power: 4, toughness: 4);
        card.Subtypes = ["Vehicle"];

        // A card's worth at two mana, the same as a 2/2 for two.
        GameClient client = Seated([Land(1), Land(2), card, InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)]);
        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10), Cast(11)))).Source);
        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(Without("stored").Decide(client, Priority(Cast(10), Cast(11)))).Source);
    }

    [Fact]
    public void A_cheated_Griselbrand_is_worth_the_seven_cards_it_draws_while_there_is_life_for_them()
    {
        const string Breach = "You may put a creature card from your hand onto the battlefield. That creature gains haste. Sacrifice that creature at the beginning of the next end step.";
        ObjectView breach = InHand(20, "Through the Breach", CardTypeView.Instant, 5, Breach);
        ObjectView griselbrand = InHand(10, "Griselbrand", CardTypeView.Creature, 8, "Flying, lifelink\nPay 7 life: Draw seven cards.", power: 7, toughness: 7, color: ColorView.Black);
        List<ObjectView> board = [.. Enumerable.Range(1, 5).Select(i => Land(i)), breach, griselbrand, Creature(30, "Wurm", 9, 9, Them, "Reach")];

        // Into a 9/9 with reach at twenty life: the attack is nothing, the seven cards are the reason.
        Assert.IsType<TakeActionAnswer>(Bot().Decide(Seated(board), Priority(Cast(20))));
        Assert.IsType<PassAnswer>(Bot().Decide(Seated(board, myLife: 9), Priority(Cast(20))));
    }
}
