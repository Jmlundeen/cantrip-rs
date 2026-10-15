using Cantrip.Client.Drafting;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.DraftingCards;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the drafter reads off a card it is shown: the colours its cost asks for, the mana it makes, and what it is
/// for - all from the cost, the type line and the text, so that a card nobody has ranked is still read.
/// </summary>
public sealed class DraftCardTests
{
    private const int W = 1, U = 2, B = 4, R = 8, G = 16;

    [Fact]
    public void A_cost_asks_for_each_colour_it_has_a_symbol_of()
    {
        DraftCard card = DraftCard.Read(Spell("Double White", "{1}{W}{W}"));

        Assert.Equal(0, card.OffColor(W | U));
        Assert.Equal(2, card.OffColor(U | B));
        Assert.Equal(2, card.Pips(0, W | U));
        Assert.Equal(0, card.Pips(1, W | U));
    }

    [Fact]
    public void A_hybrid_symbol_is_paid_by_either_colour_and_asks_for_the_one_the_deck_has()
    {
        DraftCard card = DraftCard.Read(Spell("Hybrid", "{R/W}{R/W}"));

        Assert.Equal(0, card.OffColor(R | G));
        Assert.Equal(0, card.OffColor(W | U));
        Assert.Equal(2, card.OffColor(U | B));
        Assert.Equal(2, card.Pips(3, R | G));

        // With both colours in the deck neither is asked for in particular.
        Assert.Equal(0, card.Pips(3, R | W));
    }

    [Fact]
    public void Phyrexian_mana_asks_for_no_colour()
    {
        DraftCard card = DraftCard.Read(Spell("Phyrexian", "{3}{U/P}"));

        Assert.Equal(0, card.OffColor(R | G));
    }

    [Fact]
    public void A_split_card_is_cast_by_whichever_half_the_deck_can()
    {
        // As the server shows one: the halves' costs added together (CR 709.4), and each half's own at the head of its text.
        ObjectView split = Spell("Fire // Ice", "{2}{U}{R}", CardTypeView.Instant,
            "Fire {1}{R} \u2014 Instant\nFire deals 2 damage divided as you choose among one or two targets.\n\nIce {1}{U} \u2014 Instant\nTap target permanent.\nDraw a card.");
        DraftCard card = DraftCard.Read(split);

        Assert.Equal(0, card.OffColor(R | G));
        Assert.Equal(0, card.OffColor(U | B));
        Assert.Equal(1, card.OffColor(W | G));
        Assert.Equal(1, card.Pips(3, R | G));
    }

    [Fact]
    public void A_coloured_card_with_no_cost_to_read_asks_for_its_colour()
    {
        ObjectView suspended = Spell("Suspended", string.Empty, CardTypeView.Sorcery, "Suspend 4\u2014{U}\nTarget player draws three cards.");
        suspended.Colors = [ColorView.Blue];
        DraftCard card = DraftCard.Read(suspended);

        Assert.Equal(0, card.OffColor(U | B));
        Assert.Equal(1, card.OffColor(R | G));
    }

    [Fact]
    public void A_dual_land_makes_the_colours_of_its_types()
    {
        DraftCard card = DraftCard.Read(Land("Typed Dual", "({T}: Add {U} or {B}.)", "Island", "Swamp"));

        Assert.True(card.MakesMana);
        Assert.Equal(U | B, card.Produces);
    }

    [Fact]
    public void A_land_makes_the_colours_its_text_adds()
    {
        Assert.Equal(R | G, DraftCard.Read(Land("Pain Land", "{T}: Add {C}.\n{T}: Add {R} or {G}. Pain Land deals 1 damage to you.")).Produces);
        Assert.Equal(DraftCard.AllColors, DraftCard.Read(Land("Rainbow Land", "{T}, Pay 1 life: Add one mana of any color.")).Produces);
    }

    [Fact]
    public void A_fetch_land_makes_the_colours_of_the_lands_it_finds()
    {
        DraftCard card = DraftCard.Read(Land("Fetch", "{T}, Pay 1 life, Sacrifice Fetch: Search your library for an Island or Swamp card, put it onto the battlefield, then shuffle."));

        Assert.True(card.MakesMana);
        Assert.Equal(U | B, card.Produces);
    }

    [Fact]
    public void A_land_that_makes_only_colourless_mana_makes_no_colour()
    {
        DraftCard card = DraftCard.Read(Land("Factory", "{T}: Add {C}.\n{1}: Factory becomes a 2/2 creature until end of turn."));

        Assert.True(card.MakesMana);
        Assert.Equal(0, card.Produces);
    }

    [Fact]
    public void A_mox_makes_mana_and_a_lotus_spent_once_does_not()
    {
        ObjectView mox = Spell("Mox", "{0}", CardTypeView.Artifact, "{T}: Add {G}.");
        mox.ProducesMana = true;
        ObjectView lotus = Spell("Lotus", "{0}", CardTypeView.Artifact, "{T}, Sacrifice Lotus: Add three mana of any one color.");
        lotus.ProducesMana = true;

        Assert.True(DraftCard.Read(mox).MakesMana);
        Assert.Equal(G, DraftCard.Read(mox).Produces);
        Assert.False(DraftCard.Read(lotus).MakesMana);
    }

    [Fact]
    public void Mana_of_a_choice_of_colours_is_one_mana_and_two_symbols_together_are_two()
    {
        ObjectView talisman = Spell("Talisman", "{2}", CardTypeView.Artifact, "{T}: Add {C}.\n{T}: Add {W} or {U}. Talisman deals 1 damage to you.");
        ObjectView hierarch = Creature("Hierarch", "{G}", 0, 1, "Exalted\n{T}: Add {G}, {W}, or {U}.");
        ObjectView ring = Spell("Ring", "{1}", CardTypeView.Artifact, "{T}: Add {C}{C}.");

        Assert.Equal(1, DraftCard.Read(talisman).ManaMade);
        Assert.Equal(W | U, DraftCard.Read(talisman).Produces);
        Assert.Equal(1, DraftCard.Read(hierarch).ManaMade);
        Assert.Equal(2, DraftCard.Read(ring).ManaMade);
    }

    [Fact]
    public void What_a_card_is_for_is_read_from_its_text()
    {
        Assert.True(DraftCard.Read(Spell("Kill", "{1}{B}", CardTypeView.Instant, "Destroy target creature.")).Removal);
        Assert.True(DraftCard.Read(Spell("Bolt", "{R}", CardTypeView.Instant, "Bolt deals 3 damage to any target.")).Removal);
        Assert.True(DraftCard.Read(Spell("Wrath", "{2}{W}{W}", CardTypeView.Sorcery, "Destroy all creatures.")).Sweeper);
        Assert.True(DraftCard.Read(Spell("Negate", "{U}{U}", CardTypeView.Instant, "Counter target spell.")).Counterspell);
        Assert.True(DraftCard.Read(Spell("Divination", "{2}{U}", CardTypeView.Sorcery, "Draw two cards.")).CardAdvantage);
        Assert.True(DraftCard.Read(Creature("Bird", "{1}{W}", 1, 1, "Flying")).Evasive);
        Assert.False(DraftCard.Read(Creature("Bear", "{1}{G}", 2, 2)).Removal);
    }

    [Fact]
    public void A_cheap_creature_that_hits_is_for_a_fast_deck_and_a_sweeper_for_a_slow_one()
    {
        Assert.True(DraftCard.Read(Creature("Guide", "{R}", 2, 2, "Haste")).Speed > 0.5);
        Assert.True(DraftCard.Read(Spell("Wrath", "{2}{W}{W}", CardTypeView.Sorcery, "Destroy all creatures.")).Speed < -0.5);
    }

    [Fact]
    public void A_card_no_pick_order_names_is_worth_what_it_reads_as()
    {
        DraftCard free = DraftCard.Read(MoxLike());
        DraftCard dull = DraftCard.Read(Creature("Dull", "{4}{G}{G}", 3, 3, "Trample"));

        Assert.False(free.Ranked);
        Assert.True(free.Worth > dull.Worth + 0.5, $"{free.Worth} against {dull.Worth}");

        static ObjectView MoxLike()
        {
            ObjectView mox = Spell("Free Mana", "{0}", CardTypeView.Artifact, "{T}: Add {G}.");
            mox.ProducesMana = true;
            return mox;
        }
    }

    [Fact]
    public void A_rank_is_read_onto_a_scale_that_is_flat_in_the_middle_and_steep_at_the_ends()
    {
        double best = CardWorth.FromRank(0, 360);
        double good = CardWorth.FromRank(36, 360);
        double middle = CardWorth.FromRank(180, 360);
        double nearMiddle = CardWorth.FromRank(200, 360);
        double worst = CardWorth.FromRank(359, 360);

        Assert.InRange(middle, 2.9, 3.1);
        Assert.True(best - good > 2);
        Assert.True(middle - nearMiddle < 0.2);
        Assert.True(worst < 1);
    }

    [Fact]
    public void A_card_that_puts_a_creature_or_an_artifact_into_play_for_less_is_read_as_a_cheat()
    {
        DraftCard fromHand = DraftCard.Read(Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste."));
        DraftCard fromGraveyard = DraftCard.Read(Spell("Raising", "{B}", text: "Put target creature card from a graveyard onto the battlefield under your control."));
        DraftCard fromLibrary = DraftCard.Read(Spell("Ordering", "{2}{G}{G}", text: "As an additional cost to cast this spell, sacrifice a green creature.\nSearch your library for a green creature card, put it onto the battlefield, then shuffle."));
        DraftCard artifact = DraftCard.Read(Spell("Tinkering", "{2}{U}", text: "As an additional cost to cast this spell, sacrifice an artifact.\nSearch your library for an artifact card, put that card onto the battlefield, then shuffle."));

        Assert.Equal(DraftCard.CheatsCreature, fromHand.Cheats);
        Assert.Equal(DraftCard.CheatsCreature, fromGraveyard.Cheats);
        Assert.Equal(DraftCard.CheatsCreature, fromLibrary.Cheats);
        Assert.Equal(DraftCard.CheatsArtifact, artifact.Cheats);

        // Two sentences: what is revealed, and then what is put onto the battlefield; what is found, and then if.
        DraftCard oath = DraftCard.Read(Spell("Oathing", "{1}{G}", CardTypeView.Enchantment, "At the beginning of each player's upkeep, that player chooses target opponent. The first player may reveal cards from the top of their library until they reveal a creature card. If they do, that player puts that card onto the battlefield and all other cards revealed this way into their graveyard."));
        DraftCard transmuting = DraftCard.Read(Spell("Transmuting", "{U}{U}", text: "As an additional cost to cast this spell, sacrifice an artifact.\nSearch your library for an artifact card. If that card's mana value is less than or equal to the sacrificed artifact's mana value, put it onto the battlefield. If it's greater, you may pay {X}, where X is the difference. If you do, put it onto the battlefield. If you don't, put it into its owner's graveyard. Then shuffle."));
        Assert.Equal(DraftCard.CheatsCreature, oath.Cheats);
        Assert.Equal(DraftCard.CheatsArtifact, transmuting.Cheats);
    }

    [Fact]
    public void A_card_capped_at_a_small_mana_value_on_a_body_of_its_own_or_to_the_hand_is_not_a_cheat()
    {
        DraftCard capped = DraftCard.Read(Spell("Small Raising", "{B}", text: "Return target creature card with mana value 3 or less from your graveyard to the battlefield."));
        DraftCard body = DraftCard.Read(Creature("Raising Body", "{4}{B}{B}", 4, 4, "When this creature enters, return up to one target creature card from your graveyard to the battlefield."));
        DraftCard toHand = DraftCard.Read(Spell("Regrowing", "{1}{G}", text: "Return target creature card from your graveyard to your hand."));

        Assert.Equal(0, capped.Cheats);
        Assert.Equal(0, body.Cheats);
        Assert.Equal(0, toHand.Cheats);
    }

    [Fact]
    public void What_is_worth_cheating_is_a_creature_or_an_artifact_costing_six_or_more()
    {
        Assert.Equal(DraftCard.CheatsCreature, DraftCard.Read(Creature("Giant", "{6}{R}", 7, 7)).Payload);
        ObjectView colossus = Creature("Colossus", "{8}", 11, 11);
        colossus.CardTypes = [CardTypeView.Artifact, CardTypeView.Creature];
        Assert.Equal(DraftCard.CheatsCreature | DraftCard.CheatsArtifact, DraftCard.Read(colossus).Payload);
        Assert.Equal(0, DraftCard.Read(Creature("Five", "{4}{R}", 5, 5)).Payload);
        Assert.Equal(0, DraftCard.Read(Spell("Big Sorcery", "{6}{R}")).Payload);
    }

    [Fact]
    public void A_card_that_puts_its_own_cards_into_the_graveyard_is_an_outlet_and_a_reanimation_is_fed_by_one()
    {
        DraftCard looting = DraftCard.Read(Spell("Looting", "{R}", text: "Draw two cards, then discard two cards."));
        DraftCard entombing = DraftCard.Read(Spell("Entombing", "{B}", CardTypeView.Instant, "Search your library for a card, put that card into your graveyard, then shuffle."));
        DraftCard costing = DraftCard.Read(Creature("Discarding Elf", "{1}{B}", 2, 1, "Discard a card: This creature gains indestructible until end of turn."));
        DraftCard drawing = DraftCard.Read(Spell("Drawing", "{1}{U}", text: "Draw two cards."));
        DraftCard raising = DraftCard.Read(Spell("Raising", "{B}", text: "Put target creature card from a graveyard onto the battlefield under your control."));
        DraftCard sneaking = DraftCard.Read(Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield."));

        Assert.True(looting.Outlet && entombing.Outlet && costing.Outlet);
        Assert.False(drawing.Outlet);
        Assert.True(raising.FromGraveyard);
        Assert.False(sneaking.FromGraveyard);
    }
}
