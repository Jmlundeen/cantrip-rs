using Cantrip.Client.Drafting;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.DraftingCards;

namespace Cantrip.Client.Tests;

/// <summary>
/// The deck a pool is built into: always the minimum size, the colours that cast the most of the pool's best cards,
/// and lands that cast them.
/// </summary>
public sealed class DeckBuilderTests
{
    private const int W = 1, U = 2, B = 4, R = 8, G = 16;

    /// <summary>A pool of plain creatures, so many of each colour, costing one to five and worth a middling 3.</summary>
    private static List<DraftCard> Pool(params (char Color, int Count)[] colors)
    {
        List<DraftCard> pool = [];
        foreach ((char color, int count) in colors)
        {
            for (int number = 0; number < count; number++)
            {
                pool.Add(Filler(color, 1 + (number % 5), number).Worth(3));
            }
        }

        return pool;
    }

    private static int Basic(BuiltDeck deck, string name) => deck.Basics.GetValueOrDefault(name);

    [Fact]
    public void A_deck_is_the_minimum_size_whatever_the_pool()
    {
        Assert.Equal(40, DeckBuilder.Build(Pool(('R', 14), ('G', 12), ('U', 10), ('W', 9)), 40).Size);
        Assert.Equal(40, DeckBuilder.Build(Pool(('R', 45)), 40).Size);
        Assert.Equal(40, DeckBuilder.Build(Pool(('R', 3), ('G', 2)), 40).Size);
        Assert.Equal(40, DeckBuilder.Build([], 40).Size);
        Assert.Equal(60, DeckBuilder.Build(Pool(('B', 30), ('W', 30)), 60).Size);
    }

    [Fact]
    public void The_colours_are_the_two_whose_cards_are_worth_most_not_the_two_with_most_cards()
    {
        List<DraftCard> pool = Pool(('R', 12), ('G', 12), ('U', 13));

        // Blue has the most cards, and they are poor ones.
        for (int place = 24; place < pool.Count; place++)
        {
            pool[place] = pool[place].WithWorth(1.5);
        }

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Equal(R | G, deck.Colors);
        Assert.All(deck.Main, place => Assert.True(place < 24));
        Assert.Equal(0, Basic(deck, "Island"));
        Assert.True(Basic(deck, "Mountain") >= 7 && Basic(deck, "Forest") >= 7, $"{Basic(deck, "Mountain")} Mountain, {Basic(deck, "Forest")} Forest");
    }

    [Fact]
    public void A_curve_of_ordinary_cards_gets_sixteen_or_seventeen_lands()
    {
        BuiltDeck deck = DeckBuilder.Build(Pool(('W', 14), ('B', 14)), 40);

        Assert.InRange(deck.Lands, 16, 17);
        Assert.Equal(40 - deck.Lands, deck.Main.Count);
    }

    [Fact]
    public void A_deck_names_each_card_of_its_pool_once_and_only_the_basic_lands_there_are()
    {
        // A pool with everything in it: two colours, a third to splash, more dual lands than a deck has room for.
        List<DraftCard> pool = Pool(('W', 11), ('G', 11), ('B', 6));
        pool.Add(Creature("Red Bomb", "{4}{R}", 6, 6, "Flying").Worth(7));
        for (int number = 0; number < 20; number++)
        {
            pool.Add(Land($"White-Green Dual {number}", "({T}: Add {G} or {W}.)", "Forest", "Plains").Worth(4));
        }

        pool.Add(Land("Green-Red Dual", "({T}: Add {R} or {G}.)", "Mountain", "Forest").Worth(4));
        pool.Add(Land("White-Red Dual", "({T}: Add {R} or {W}.)", "Mountain", "Plains").Worth(4));

        foreach (int minimum in (int[])[40, 60, 30])
        {
            BuiltDeck deck = DeckBuilder.Build(pool, minimum);

            Assert.Equal(minimum, deck.Size);
            Assert.Equal(deck.Main.Count, deck.Main.Distinct().Count());
            Assert.All(deck.Main, place => Assert.InRange(place, 0, pool.Count - 1));
            Assert.All(deck.Basics, basic => Assert.Contains(basic.Key, DeckBuilder.BasicLands));
            Assert.All(deck.Basics, basic => Assert.True(basic.Value > 0));
            Assert.Equal(deck.Lands, deck.Main.Count(place => pool[place].IsLand) + deck.Basics.Values.Sum());
        }
    }

    [Fact]
    public void A_pool_of_nothing_but_lands_is_still_a_deck_of_the_minimum()
    {
        List<DraftCard> pool = [.. Enumerable.Range(0, 10).Select(number => Land($"Dual {number}", "({T}: Add {U} or {B}.)", "Island", "Swamp").Worth(4))];

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Equal(40, deck.Size);
        Assert.Equal(deck.Main.Count, deck.Main.Distinct().Count());
    }

    [Fact]
    public void Free_mana_takes_lands_out()
    {
        // A curve that asks for seventeen lands, and the same with two of its cards free mana.
        List<DraftCard> curve = [.. Enumerable.Range(0, 26).Select(number => Filler('R', 2 + (number % 3), number).Worth(3))];
        List<DraftCard> withMoxen = [.. curve];
        for (int number = 0; number < 2; number++)
        {
            ObjectView mox = Spell($"Mox {number}", "{0}", CardTypeView.Artifact, "{T}: Add {R}.");
            mox.ProducesMana = true;
            withMoxen.Add(mox.Worth(6));
        }

        BuiltDeck plain = DeckBuilder.Build(curve, 40);
        BuiltDeck fast = DeckBuilder.Build(withMoxen, 40);

        Assert.Equal(17, plain.Lands);
        Assert.Equal(15, fast.Lands);
        Assert.Contains(26, fast.Main);
        Assert.Contains(27, fast.Main);
    }

    [Fact]
    public void A_low_curve_takes_a_land_out_and_a_high_one_puts_one_in()
    {
        BuiltDeck low = DeckBuilder.Build([.. Enumerable.Range(0, 26).Select(number => Filler('R', 1 + (number % 3), number).Worth(3))], 40);
        BuiltDeck high = DeckBuilder.Build([.. Enumerable.Range(0, 26).Select(number => Filler('R', 3 + (number % 3), number).Worth(3))], 40);

        Assert.Equal(16, low.Lands);
        Assert.Equal(18, high.Lands);
    }

    [Fact]
    public void The_lands_follow_what_the_spells_ask_for()
    {
        // Sixteen white cards, half of them asking for two white, and seven blue ones.
        List<DraftCard> pool = [];
        for (int number = 0; number < 16; number++)
        {
            pool.Add(Creature($"White {number}", number % 2 == 0 ? "{W}{W}" : "{2}{W}", 2, 2).Worth(3));
        }

        for (int number = 0; number < 7; number++)
        {
            pool.Add(Creature($"Blue {number}", "{3}{U}", 3, 3).Worth(3));
        }

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Equal(W | U, deck.Colors);
        Assert.True(Basic(deck, "Plains") > Basic(deck, "Island") + 2, $"{Basic(deck, "Plains")} Plains, {Basic(deck, "Island")} Island");
        Assert.True(Basic(deck, "Island") >= 5, $"{Basic(deck, "Plains")} Plains, {Basic(deck, "Island")} Island");
    }

    [Fact]
    public void A_land_that_makes_both_colours_is_played_and_one_that_makes_neither_is_not()
    {
        List<DraftCard> pool = Pool(('U', 12), ('B', 12));
        pool.Add(Land("Our Dual", "({T}: Add {U} or {B}.)", "Island", "Swamp").Worth(4));
        pool.Add(Land("Their Dual", "({T}: Add {R} or {G}.)", "Mountain", "Forest").Worth(4));

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Contains(24, deck.Main);
        Assert.DoesNotContain(25, deck.Main);
        Assert.Equal(40, deck.Size);
    }

    [Fact]
    public void A_bomb_of_a_third_colour_is_splashed_when_the_lands_are_there_for_it()
    {
        List<DraftCard> pool = Pool(('W', 12), ('G', 12));
        pool.Add(Creature("Red Bomb", "{4}{R}", 6, 6, "Flying").Worth(7));
        pool.Add(Land("White-Red Dual", "({T}: Add {R} or {W}.)", "Mountain", "Plains").Worth(4));
        pool.Add(Land("Green-Red Dual", "({T}: Add {R} or {G}.)", "Mountain", "Forest").Worth(4));

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Equal(W | G, deck.Colors);
        Assert.Equal(R, deck.Splash);
        Assert.Contains(24, deck.Main);
        Assert.True(deck.Sources[3] >= 3, $"{deck.Sources[3]} red sources");
    }

    [Fact]
    public void An_ordinary_card_of_a_third_colour_is_not_splashed()
    {
        List<DraftCard> pool = Pool(('W', 14), ('G', 14));
        pool.Add(Creature("Red Card", "{2}{R}", 3, 3).Worth(3.5));

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Equal(0, deck.Splash);
        Assert.DoesNotContain(28, deck.Main);
        Assert.Equal(0, Basic(deck, "Mountain"));
    }

    [Fact]
    public void A_card_asking_for_two_of_a_third_colour_is_never_splashed()
    {
        List<DraftCard> pool = Pool(('W', 12), ('G', 12));
        pool.Add(Creature("Double Red Bomb", "{3}{R}{R}", 6, 6, "Flying").Worth(7.5));
        pool.Add(Land("White-Red Dual", "({T}: Add {R} or {W}.)", "Mountain", "Plains").Worth(4));

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.DoesNotContain(24, deck.Main);
    }

    [Fact]
    public void A_pool_of_one_colour_is_a_deck_of_one_colour()
    {
        BuiltDeck deck = DeckBuilder.Build(Pool(('B', 30), ('R', 3)), 40);

        Assert.Equal(B, deck.Colors);
        Assert.Equal(deck.Lands, Basic(deck, "Swamp"));
    }

    [Fact]
    public void A_deck_short_of_creatures_plays_a_lesser_creature_over_one_more_spell()
    {
        // Twenty spells that are a little better than the eleven creatures.
        List<DraftCard> pool = [];
        for (int number = 0; number < 20; number++)
        {
            pool.Add(Spell($"Spell {number}", "{1}{U}", CardTypeView.Instant).Worth(3.2));
        }

        for (int number = 0; number < 11; number++)
        {
            pool.Add(Creature($"Creature {number}", "{2}{U}", 2, 2).Worth(3.0));
        }

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.True(deck.Main.Count(place => place >= 20) >= 9, $"{deck.Main.Count(place => place >= 20)} creatures");
    }

    [Fact]
    public void A_shape_costs_what_the_shipped_numbers_say_and_another_tuning_what_it_says()
    {
        // Twenty-three spells: ten creatures, four plays at two or less, seven at five or more.
        Assert.Equal((0.45 * 3) + (0.3 * 2) + (0.5 * 2), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0), 9);

        // Leaning early it wants nine early plays and holds three expensive cards; leaning late, nine creatures and six.
        Assert.Equal((0.45 * 3) + (0.3 * 5) + (0.5 * 4), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0.5), 9);
        Assert.Equal((0.3 * 2) + (0.5 * 1), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: -0.5), 9);

        var asked = new DeckTuning { Cheap = 8, CheapCost = 1, Expensive = 3, ExpensiveCost = 2 };
        Assert.Equal((0.45 * 3) + (1.0 * 4) + (2.0 * 4), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0, asked), 9);
    }

    [Fact]
    public void A_deck_asked_for_more_early_plays_takes_a_lesser_cheap_card_over_a_better_one()
    {
        // Eighteen three-drops a little better than the ten two-drops, all creatures, none of them quick.
        List<DraftCard> pool = [];
        for (int number = 0; number < 18; number++)
        {
            pool.Add(Creature($"Three {number}", "{2}{U}", 2, 2).Worth(3.3));
        }

        for (int number = 0; number < 10; number++)
        {
            pool.Add(Creature($"Two {number}", "{1}{U}", 1, 1).Worth(3.0));
        }

        BuiltDeck shipped = DeckBuilder.Build(pool, 40);
        BuiltDeck asked = DeckBuilder.Build(pool, 40, new DeckTuning { Cheap = 10, CheapCost = 1 });

        Assert.True(shipped.Main.Count(place => place >= 18) < 10, $"{shipped.Main.Count(place => place >= 18)} two-drops as shipped");
        Assert.Equal(10, asked.Main.Count(place => place >= 18));
    }

    [Fact]
    public void A_mox_of_another_colour_is_worth_less_to_the_deck_than_one_of_its_own()
    {
        List<DraftCard> pool = Pool(('R', 23));
        ObjectView ours = Spell("Red Mox", "{0}", CardTypeView.Artifact, "{T}: Add {R}.");
        ours.ProducesMana = true;
        ObjectView theirs = Spell("Blue Mox", "{0}", CardTypeView.Artifact, "{T}: Add {U}.");
        theirs.ProducesMana = true;
        DraftCard oursRead = ours.Worth(5);
        DraftCard theirsRead = theirs.Worth(5);

        Assert.True(DeckBuilder.PowerIn(oursRead, R) > DeckBuilder.PowerIn(theirsRead, R));
    }

    /// <summary>A strong red card that puts a creature from the hand onto the battlefield.</summary>
    private static ObjectView Sneaking() => Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste.");

    [Fact]
    public void A_card_that_cheats_creatures_into_play_is_left_out_of_a_deck_with_none_worth_cheating()
    {
        DraftCard cheat = Sneaking().Worth(4.5);
        List<DraftCard> pool = [.. Pool(('R', 16), ('G', 16)), cheat];

        BuiltDeck deck = DeckBuilder.Build(pool, 40);
        BuiltDeck reading = DeckBuilder.Build(pool, 40, new DeckTuning { Unsupported = 1 });

        Assert.DoesNotContain(pool.Count - 1, deck.Main);
        Assert.Contains(pool.Count - 1, reading.Main);
    }

    [Fact]
    public void A_card_that_cheats_creatures_into_play_is_played_beside_one_worth_cheating()
    {
        DraftCard cheat = Sneaking().Worth(4.5);
        List<DraftCard> pool = [.. Pool(('R', 16), ('G', 16)), Creature("Red Giant", "{6}{R}", 7, 7).Worth(3), cheat];

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.Contains(pool.Count - 1, deck.Main);
        Assert.Contains(pool.Count - 2, deck.Main);
    }

    [Fact]
    public void A_cheat_is_worth_nothing_extra_for_a_creature_left_out_of_the_deck()
    {
        // A Giant at worth 0.5 does not make the deck whatever it would let the cheat do, so the cheat is left out too.
        DraftCard cheat = Sneaking().Worth(4.5);
        List<DraftCard> pool = [.. Pool(('R', 16), ('G', 16)), Creature("Red Giant", "{6}{R}", 7, 7).Worth(0.5), cheat];

        BuiltDeck deck = DeckBuilder.Build(pool, 40);

        Assert.DoesNotContain(pool.Count - 2, deck.Main);
        Assert.DoesNotContain(pool.Count - 1, deck.Main);
    }

    [Fact]
    public void A_cheat_keeps_all_its_power_with_enough_to_cheat_and_a_share_of_it_with_nothing()
    {
        DraftCard cheat = Sneaking().Worth(4.5);
        var tuning = new DeckTuning { Unsupported = 0.4, Payloads = 2 };

        Assert.Equal(0.4, new Payloads(0, 3).Support(cheat, tuning), 9);
        Assert.Equal(0.7, new Payloads(1, 0).Support(cheat, tuning), 9);
        Assert.Equal(1, new Payloads(2, 0).Support(cheat, tuning), 9);
        Assert.Equal(1, new Payloads(0, 0).Support(Filler('R', 3, 1).Worth(3), tuning), 9);
        Assert.Equal(1, new Payloads(0, 0).Support(cheat, new DeckTuning { Unsupported = 1 }), 9);
    }

    [Fact]
    public void Asked_to_a_reanimation_wants_a_way_into_the_graveyard_as_well_as_something_to_put_there()
    {
        DraftCard raising = Spell("Raising", "{B}", text: "Put target creature card from a graveyard onto the battlefield under your control.").Worth(4);
        DraftCard sneaking = Sneaking().Worth(4);
        var asked = new DeckTuning { Outlets = 1 };

        Assert.Equal(0.4, new Payloads(1, 0, 0).Support(raising, asked), 9);
        Assert.Equal(1, new Payloads(1, 0, 1).Support(raising, asked), 9);
        Assert.Equal(1, new Payloads(1, 0, 0).Support(sneaking, asked), 9);
        Assert.Equal(1, new Payloads(1, 0, 0).Support(raising, new DeckTuning()), 9);
    }

    [Fact]
    public void Asked_to_a_deck_of_mana_makers_holds_more_expensive_cards()
    {
        // Seven at five or more against five held: two too many, as shipped whatever makes mana.
        Assert.Equal((0.45 * 3) + (0.3 * 2) + (0.5 * 2), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0, makers: 5), 9);
        Assert.Equal((0.45 * 3) + (0.3 * 2), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0, new DeckTuning { Ramp = 1 }, makers: 4), 9);
        Assert.Equal((0.45 * 3) + (0.3 * 2) + (0.5 * 1), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: 0, new DeckTuning { Ramp = 1 }, makers: 3), 9);
    }

    [Fact]
    public void A_deck_that_leans_late_wants_fewer_creatures_by_as_many_as_asked()
    {
        // Ten creatures, leaning late: nine wanted as shipped, eleven if asked for two fewer rather than four.
        Assert.Equal((0.3 * 2) + (0.5 * 1), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: -0.5), 9);
        Assert.Equal((0.45 * 1) + (0.3 * 2) + (0.5 * 1), DeckBuilder.ShapeCost(23, 10, 4, 7, lean: -0.5, new DeckTuning { LateFewer = 2 }), 9);
    }
}
