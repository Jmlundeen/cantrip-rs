using Cantrip.Client.Drafting;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.DraftingCards;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the cube drafter takes from a pack, and why: raw power while nothing is settled, the deck its pool is becoming
/// once something is, and what the packs passed to it say about the colours to come.
/// </summary>
public sealed class CubeDrafterTests
{
    /// <summary>Ranks of 360 that read as a bomb, a strong card, a good one, a middling one and a poor one.</summary>
    private const int Bomb = 2, Strong = 30, Good = 90, Middling = 180, Poor = 300;

    private static DraftMessage Pack(int round, int pick, IReadOnlyList<ObjectView> pool, params (ObjectView Card, int Rank)[] pack) => new()
    {
        Format = "booster",
        Decision = ((round - 1) * 15) + pick - 1,
        Pile = [.. pack.Select(card => card.Card)],
        PileRanks = [.. pack.Select(card => card.Rank)],
        Pool = [.. pool],
        Total = 360,
        PackNumber = round,
        Packs = 3,
        PackSize = 15,
        PickNumber = pick,
        Direction = round == 2 ? -1 : 1,
    };

    /// <summary>A drafter that has seen these cards at these ranks, as it would have seen them in the packs it took them from.</summary>
    private static CubeDrafter Knowing(IEnumerable<ObjectView> cards, int rank)
    {
        var drafter = new CubeDrafter();
        List<ObjectView> known = [.. cards];
        drafter.Learn(known, [.. known.Select((_, at) => rank + 1 + at)], 360);
        return drafter;
    }

    private static List<ObjectView> Creatures(char color, int count, int from = 0) =>
        [.. Enumerable.Range(from, count).Select(number => Filler(color, 1 + (number % 5), number))];

    [Fact]
    public void The_first_pick_is_the_best_card_whatever_its_colour()
    {
        var drafter = new CubeDrafter();
        DraftChoice choice = drafter.Choose(Pack(1, 1, [],
            (Filler('G', 2, 1), Good),
            (Filler('U', 3, 2), Bomb),
            (Filler('R', 2, 3), Strong)));

        Assert.Equal(1, choice.Places[0]);
        Assert.True(choice.Scores[2] > choice.Scores[0]);
    }

    [Fact]
    public void Of_two_cards_as_good_the_colourless_one_is_taken_first_and_the_gold_one_last()
    {
        var drafter = new CubeDrafter();
        ObjectView colourless = Spell("Colourless", "{3}", CardTypeView.Artifact);
        ObjectView mono = Spell("Mono", "{2}{R}");
        ObjectView gold = Spell("Gold", "{1}{R}{G}");
        DraftChoice choice = drafter.Choose(Pack(1, 1, [], (gold, Strong), (mono, Strong + 1), (colourless, Strong + 2)));

        Assert.True(choice.Scores[2] > choice.Scores[1] && choice.Scores[1] > choice.Scores[0], string.Join(", ", choice.Scores));
        Assert.Equal(2, choice.Places[0]);
    }

    [Fact]
    public void A_pool_set_on_two_colours_takes_its_own_good_card_over_a_better_one_it_cannot_cast()
    {
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 9)];
        CubeDrafter drafter = Knowing(pool, Good);

        DraftChoice choice = drafter.Choose(Pack(2, 4, pool,
            (Filler('U', 3, 50), Strong),
            (Filler('R', 3, 51), Good),
            (Filler('G', 2, 52), Strong + 5)));

        Assert.Equal(1, choice.Places[0]);
    }

    [Fact]
    public void Early_on_a_much_better_card_of_another_colour_is_still_taken()
    {
        List<ObjectView> pool = Creatures('R', 2);
        CubeDrafter drafter = Knowing(pool, Good);

        DraftChoice choice = drafter.Choose(Pack(1, 3, pool,
            (Filler('U', 3, 50), Bomb),
            (Filler('R', 3, 51), Middling)));

        Assert.Equal(0, choice.Places[0]);
    }

    [Fact]
    public void What_the_pool_says_its_deck_is_grows_surer_as_it_fills()
    {
        List<ObjectView> few = [.. Creatures('R', 2), .. Creatures('W', 1)];
        List<ObjectView> many = [.. Creatures('R', 9), .. Creatures('W', 9)];
        int redWhite = DeckBuilder.Combinations.ToList().IndexOf(1 | 8);

        double early = Knowing(few, Good).Choose(Pack(1, 4, few, (Filler('B', 2, 60), Middling))).Beliefs[redWhite];
        double late = Knowing(many, Good).Choose(Pack(2, 4, many, (Filler('B', 2, 61), Middling))).Beliefs[redWhite];

        Assert.True(early < 0.6, $"{early:P0} after three picks");
        Assert.True(late > 0.9, $"{late:P0} after eighteen");
    }

    [Fact]
    public void A_deck_with_its_spells_already_takes_the_land_that_fixes_its_mana_over_one_more_spell()
    {
        // Twenty-six good playables: a twenty-seventh, no better, would not make the deck.
        List<ObjectView> pool = [.. Creatures('U', 13), .. Creatures('B', 13)];
        CubeDrafter drafter = Knowing(pool, Good);

        DraftChoice choice = drafter.Choose(Pack(3, 6, pool,
            (Filler('U', 3, 70), Middling),
            (Land("Blue-Black Dual", "({T}: Add {U} or {B}.)", "Island", "Swamp"), Middling),
            (Land("Red-Green Dual", "({T}: Add {R} or {G}.)", "Mountain", "Forest"), Good)));

        Assert.Equal(1, choice.Places[0]);
    }

    [Fact]
    public void A_deck_heavy_at_the_top_takes_the_cheap_card()
    {
        // Plenty of fives, nothing to do early.
        List<ObjectView> pool = [.. Enumerable.Range(0, 9).Select(number => Filler('G', 5, number)), .. Enumerable.Range(0, 9).Select(number => Filler('W', 4 + (number % 2), 20 + number))];
        CubeDrafter drafter = Knowing(pool, Good);

        DraftChoice choice = drafter.Choose(Pack(2, 5, pool,
            (Filler('G', 6, 80), Good + 30),
            (Filler('G', 2, 81), Good + 40)));

        Assert.True(choice.Scores[1] > choice.Scores[0], string.Join(", ", choice.Scores));
    }

    [Fact]
    public void A_pack_is_read_as_what_each_colour_has_worth_playing_against_the_average_colour()
    {
        // Blue has two cards better than middling, red one worse, and a gold card splits what it has between its colours.
        double[] reading = DraftMemory.Reading(
        [
            Filler('U', 3, 1).Worth(5), Filler('U', 2, 2).Worth(4), Filler('R', 2, 3).Worth(2),
            Spell("White-Black", "{W}{B}").Worth(3.6), Spell("Colourless", "{3}", CardTypeView.Artifact).Worth(7),
        ]);

        Assert.Equal(0, reading.Sum(), 6);
        Assert.Equal(3 - 0.8, reading[1], 6);
        Assert.Equal(0.5 - 0.8, reading[0], 6);
        Assert.Equal(reading[0], reading[2], 6);
        Assert.Equal(-0.8, reading[3], 6);
        Assert.Equal(reading[3], reading[4], 6);
    }

    [Fact]
    public void A_colour_that_keeps_coming_late_is_read_as_open()
    {
        var drafter = new CubeDrafter();
        List<ObjectView> pool = [];

        // Picks five to eight of pack one: a strong blue card still there each time, and nothing much else.
        for (int pick = 5; pick <= 8; pick++)
        {
            List<(ObjectView, int)> pack = [(Filler('U', 3, 100 + pick), Strong + pick)];
            pack.AddRange(Enumerable.Range(0, 15 - pick).Select(other => (Filler("WBRG"[other % 4], 4, (pick * 20) + other), Poor + other)));
            drafter.Choose(Pack(1, pick, pool, [.. pack]));
        }

        DraftMemory memory = drafter.Memory;
        Assert.True(memory.Flow(1, 1) > 0.5, $"blue {memory.Flow(1, 1):N2}");
        Assert.True(memory.Flow(3, 1) < 0, $"red {memory.Flow(3, 1):N2}");

        // And expected to go on coming, this pack and the next that comes the same way round.
        Assert.True(memory.Open(1, 1, signals: 1, carry: 0, cut: 0, crowd: 0) > memory.Open(3, 1, signals: 1, carry: 0, cut: 0, crowd: 0));
        Assert.True(memory.Open(1, 3, signals: 0, carry: 1, cut: 0, crowd: 0) > memory.Open(3, 3, signals: 0, carry: 1, cut: 0, crowd: 0));
        Assert.Equal(0, memory.Open(1, 2, signals: 1, carry: 1, cut: 0, crowd: 0));
    }

    [Fact]
    public void A_colour_that_was_open_in_pack_one_is_drafted_in_pack_three_over_one_that_was_not()
    {
        var drafter = new CubeDrafter();
        List<ObjectView> pool = [];

        // Pack one: each pack still rich in blue however late, and a colourless card better than any of it to take,
        // so the pool itself leans nowhere.
        for (int pick = 2; pick <= 8; pick++)
        {
            List<(ObjectView, int)> pack = [(Spell($"Colourless {pick}", "{3}", CardTypeView.Artifact), Bomb + pick), (Filler('U', 3, 200 + pick), Strong + pick), (Filler('U', 2, 220 + pick), Good + pick)];
            pack.AddRange(Enumerable.Range(0, 13 - pick).Select(other => (Filler("WBRG"[other % 4], 4, 300 + (pick * 20) + other), Poor + (pick * 4) + (other % 4))));
            DraftMessage shown = Pack(1, pick, pool, [.. pack]);
            Assert.Equal(0, drafter.Choose(shown).Places[0]);
            pool.Add(shown.Pile[0]);
        }

        // Pack three comes the same way: of a red card and a blue one as good, the blue one.
        DraftChoice choice = drafter.Choose(Pack(3, 1, pool, (Filler('R', 3, 400), Good + 20), (Filler('U', 3, 401), Good + 21)));
        Assert.True(choice.Scores[1] > choice.Scores[0], string.Join(", ", choice.Scores));
    }

    [Fact]
    public void What_was_sent_downstream_counts_against_a_colour_when_the_direction_turns()
    {
        var drafter = new CubeDrafter();

        // Pack one: takes the bomb each time and passes strong green cards on.
        for (int pick = 1; pick <= 4; pick++)
        {
            List<(ObjectView, int)> pack = [(Filler('W', 2, 500 + pick), Bomb + pick), (Filler('G', 3, 520 + pick), Strong + pick), (Filler('G', 4, 540 + pick), Strong + 10 + pick)];
            pack.AddRange(Enumerable.Range(0, 13 - pick).Select(other => (Filler("UBR"[other % 3], 4, 600 + (pick * 20) + other), Poor + other)));
            Assert.Equal(0, drafter.Choose(Pack(1, pick, [], [.. pack])).Places[0]);
        }

        DraftMemory memory = drafter.Memory;
        Assert.True(memory.Passed(4, 1) > 0.5, $"green passed {memory.Passed(4, 1):N2}");
        Assert.True(memory.Open(4, 2, signals: 0, carry: 0, cut: 1, crowd: 0) < memory.Open(1, 2, signals: 0, carry: 0, cut: 1, crowd: 0));
    }

    [Fact]
    public void A_pack_that_comes_round_again_says_what_everyone_else_took()
    {
        var drafter = new CubeDrafter();
        ObjectView taken = Filler('W', 2, 700);
        ObjectView redOne = Filler('R', 2, 701);
        ObjectView redTwo = Filler('R', 3, 702);
        List<(ObjectView, int)> first = [(taken, Bomb), (redOne, Poor), (redTwo, Poor + 1)];
        first.AddRange(Enumerable.Range(0, 12).Select(other => (Filler("UBG"[other % 3], 3, 710 + other), Good + other)));
        Assert.Equal(0, drafter.Choose(Pack(1, 1, [], [.. first])).Places[0]);

        // Seven cards have gone by the time it is back: the two poor red ones among them, taken over better cards.
        List<(ObjectView, int)> back = [.. first.Skip(3).Skip(5)];
        drafter.Choose(Pack(1, 9, [taken], [.. back]));

        DraftMemory memory = drafter.Memory;
        Assert.Equal(7, memory.TakenByOthers.Count);
        Assert.Contains(memory.TakenByOthers, card => card.Name == redOne.Name);
        Assert.True(memory.Contested(3) > 0, $"red {memory.Contested(3):N2}");
        Assert.True(memory.Contested(1) < memory.Contested(3));
        Assert.True(memory.Open(3, 2, signals: 0, carry: 0, cut: 0, crowd: 1) < memory.Open(1, 2, signals: 0, carry: 0, cut: 0, crowd: 1));
        Assert.Same(memory.Sights[0], memory.Sights[1].Before);
    }

    [Fact]
    public void The_last_card_of_a_pack_is_matched_to_the_pack_it_was_and_no_other()
    {
        var drafter = new CubeDrafter();
        ObjectView last = Filler('G', 4, 760);

        // Pick seven is the pack that comes back as the last card; pick fourteen, seen just before it, is another.
        List<(ObjectView, int)> seventh = [(Filler('W', 2, 761), Bomb), (last, Poor), .. Enumerable.Range(0, 7).Select(other => (Filler('U', 3, 770 + other), Good + other))];
        drafter.Choose(Pack(1, 7, [], [.. seventh]));
        drafter.Choose(Pack(1, 14, [], (Filler('R', 2, 790), Middling), (Filler('B', 2, 791), Middling + 1)));
        drafter.Choose(Pack(1, 15, [], (last, Poor)));

        DraftMemory memory = drafter.Memory;
        Assert.Same(memory.Sights[0], memory.Sights[2].Before);
        Assert.Equal(7, memory.TakenByOthers.Count);
        Assert.DoesNotContain(memory.TakenByOthers, card => card.Name == last.Name);
    }

    [Fact]
    public void Of_two_copies_of_a_card_in_a_pack_the_one_not_taken_is_passed_on()
    {
        var drafter = new CubeDrafter();
        ObjectView copy = Filler('W', 2, 765);
        drafter.Choose(Pack(1, 1, [], (copy, Bomb), (copy, Bomb), (Filler('U', 3, 766), Good)));

        DraftMemory.Sight sight = Assert.Single(drafter.Memory.Sights);
        Assert.Single(sight.Taken);
        Assert.Equal(2, sight.Passed.Count());
        Assert.Contains(sight.Passed, card => card.Name == copy.Name);
    }

    [Fact]
    public void A_pack_is_looked_at_once_however_often_it_is_shown()
    {
        var drafter = new CubeDrafter();
        DraftMessage shown = Pack(1, 2, [], (Filler('G', 2, 800), Good), (Filler('U', 3, 801), Middling));

        drafter.Choose(shown);
        drafter.Choose(shown);

        Assert.Single(drafter.Memory.Sights);
    }

    [Fact]
    public void Cards_the_pick_order_does_not_name_are_worth_what_they_read_as()
    {
        // The server gives every card its order does not name the same rank, past the order's end.
        var drafter = new CubeDrafter();
        ObjectView removal = Spell("Unranked Removal", "{1}{B}", CardTypeView.Instant, "Destroy target creature.");
        ObjectView vanilla = Creature("Unranked Vanilla", "{5}{G}", 3, 3);
        ObjectView ranked = Filler('W', 2, 900);
        DraftMessage shown = Pack(1, 1, [], (removal, 340), (vanilla, 340), (ranked, Poor));

        DraftChoice choice = drafter.Choose(shown);

        Assert.False(drafter.Card(removal).Ranked);
        Assert.True(drafter.Card(ranked).Ranked);
        Assert.True(drafter.Card(removal).Worth > drafter.Card(vanilla).Worth);
        Assert.Equal(0, choice.Places[0]);
    }

    [Fact]
    public void With_no_pick_order_at_all_it_drafts_by_reading_the_cards()
    {
        var drafter = new CubeDrafter();
        ObjectView removal = Spell("Cheap Removal", "{1}{B}", CardTypeView.Instant, "Destroy target creature.");
        ObjectView vanilla = Creature("Vanilla", "{5}{G}", 3, 3);
        DraftMessage shown = Pack(1, 1, [], (vanilla, 0), (removal, 0));
        shown.PileRanks = [];

        Assert.Equal(1, drafter.Choose(shown).Places[0]);
    }

    [Fact]
    public void A_seat_spending_a_librarian_is_given_two_cards()
    {
        var drafter = new CubeDrafter();
        DraftChoice choice = drafter.Choose(Pack(1, 1, [], (Filler('G', 2, 950), Good), (Filler('U', 3, 951), Bomb), (Filler('R', 2, 952), Strong)), count: 2);

        Assert.Equal([1, 2], choice.Places);
    }

    [Fact]
    public void Late_in_the_draft_a_cheat_is_taken_only_by_a_pool_with_something_to_cheat()
    {
        ObjectView Cheat() => Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield. That creature gains haste.");
        List<ObjectView> plain = [.. Creatures('R', 9), .. Creatures('W', 9)];
        List<ObjectView> giants = [.. plain, Creature("Red Giant", "{6}{R}", 7, 7)];

        DraftChoice without = Knowing(plain, Good).Choose(Pack(3, 6, plain, (Cheat(), Strong), (Filler('R', 3, 90), Good + 20)));
        DraftChoice with = Knowing(giants, Good).Choose(Pack(3, 6, giants, (Cheat(), Strong), (Filler('R', 3, 91), Good + 20)));

        Assert.Equal(1, without.Places[0]);
        Assert.Equal(0, with.Places[0]);
    }

    /// <summary>A grid as the server shows it: nine places row by row, each card's rank beside it.</summary>
    private static DraftMessage Grid(int pack, IReadOnlyList<ObjectView> pool, params (ObjectView Card, int Rank)[] places) => new()
    {
        Format = "grid",
        Active = 1,
        Pile = [.. places.Select(place => place.Card)],
        PileRanks = [.. places.Select(place => place.Rank)],
        Pool = [.. pool],
        Total = 162,
        PackNumber = pack,
        Packs = 18,
        PackSize = 9,
        PickNumber = 1,
    };

    [Fact]
    public void A_grid_line_of_the_pools_colours_is_taken_over_a_line_ranked_better_that_it_cannot_cast()
    {
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 9)];
        CubeDrafter drafter = Knowing(pool, Good);
        DraftMessage grid = Grid(10, pool,
            (Filler('U', 3, 100), Strong), (Filler('U', 2, 101), Strong + 1), (Filler('U', 4, 102), Strong + 2),
            (Filler('R', 3, 103), Good + 30), (Filler('W', 2, 104), Good + 31), (Filler('R', 2, 105), Good + 32),
            (Filler('G', 3, 106), Poor), (Filler('G', 2, 107), Poor + 1), (Filler('B', 4, 108), Poor + 2));

        Assert.Equal(0, DraftPlay.ChooseLine(grid.Pile, grid.Pool, grid.PileRanks));
        Assert.Equal(1, drafter.ChooseLine(grid));
    }

    /// <summary>A Winston pile as the server shows it, late in the draft: no pick order, and few cards still to come.</summary>
    private static DraftMessage Winston(IReadOnlyList<ObjectView> pool, params ObjectView[] pile) => new()
    {
        Format = "winston",
        Active = 1,
        Looking = 0,
        Piles = [pile.Length, 1, 1],
        Stack = 6,
        Pile = [.. pile],
        CanPass = true,
        Pool = [.. pool],
        Seats = [new DraftSeatView { Seat = 1 }, new DraftSeatView { Seat = 2 }],
        Total = 90,
    };

    [Fact]
    public void A_winston_pile_the_pool_cannot_play_is_passed_for_the_stack_while_its_deck_is_short_and_one_it_can_is_taken()
    {
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 9)];
        ObjectView removal = Spell("Red Removal", "{R}", CardTypeView.Instant, "Red Removal deals 3 damage to any target.");

        // Two symbols of a colour the deck does not play: not even a splash.
        Assert.False(new CubeDrafter().TakePile(Winston(pool, Creature("Blue Pair", "{1}{U}{U}", 3, 3), Creature("Green Pair", "{G}{G}", 2, 2))));
        Assert.True(new CubeDrafter().TakePile(Winston(pool, removal)));
        DraftMessage last = Winston(pool, Filler('U', 3, 112));
        last.CanPass = false;
        Assert.True(new CubeDrafter().TakePile(last));
    }

    [Fact]
    public void A_second_pick_from_a_grid_takes_a_line_with_cards_left_and_reads_no_rank_off_the_places_taken()
    {
        // The first pick took the top row: its places are shown as no card, ranked past the end of the order (a rank no card shown has).
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 9)];
        CubeDrafter drafter = Knowing(pool, Good);
        ObjectView probe = pool[0];
        double before = drafter.Card(probe).Worth;
        ObjectView Gone() => new() { Known = false };
        DraftMessage grid = Grid(10, pool,
            (Gone(), 400), (Gone(), 400), (Gone(), 400),
            (Filler('U', 3, 120), Strong), (Filler('U', 2, 121), Strong + 1), (Filler('B', 4, 122), Poor),
            (Filler('R', 3, 123), Good + 30), (Filler('W', 2, 124), Good + 31), (Filler('R', 2, 125), Good + 32));
        grid.PickNumber = 2;
        grid.Total = 162;

        int line = drafter.ChooseLine(grid);

        Assert.Equal(2, line);
        Assert.Equal(before, drafter.Card(probe).Worth);
    }

    [Fact]
    public void Asked_to_a_first_pick_from_a_grid_leaves_the_other_player_less()
    {
        // Row 1 is a little better for this pool; taking column 3 instead takes the bomb the other player would have.
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 9)];
        DraftMessage Make() => Grid(10, pool,
            (Creature("Blue Bomb", "{3}{U}{U}", 5, 5), Bomb), (Filler('G', 3, 131), Poor), (Filler('G', 4, 132), Poor + 1),
            (Filler('R', 3, 133), Good + 30), (Filler('W', 2, 134), Good + 31), (Filler('R', 2, 135), Good + 32),
            (Filler('R', 4, 136), Good + 33), (Filler('B', 2, 137), Poor + 2), (Filler('B', 3, 138), Poor + 3));

        Assert.Equal(1, Knowing(pool, Good).ChooseLine(Make()));
        var denying = new CubeDrafter { Tuning = new DraftTuning { Deny = 1 } };
        denying.Learn(pool, [.. pool.Select((_, at) => Good + 1 + at)], 360);
        Assert.Equal(3, denying.ChooseLine(Make()));
    }

    [Fact]
    public void Explaining_a_choice_changes_no_score_belief_line_pile_or_deck()
    {
        // Two drafters shown the same packs, one asked to say why each time: every number the same, to the last bit.
        List<ObjectView> pool = [.. Creatures('R', 9), .. Creatures('W', 6)];
        var silent = Knowing(pool, Good);
        var explaining = Knowing(pool, Good);
        ObjectView cheat = Spell("Sneaking", "{3}{R}", CardTypeView.Enchantment, "{R}: You may put a creature card from your hand onto the battlefield.");
        DraftMessage[] packs =
        [
            Pack(2, 3, pool, (Filler('U', 3, 200), Strong), (Filler('R', 3, 201), Good), (Spell("Gold", "{R}{W}"), Strong + 4), (cheat, Good + 2), (Creature("Fatty", "{5}{R}{R}", 7, 7), Middling)),
            Pack(2, 4, pool, (Land("Red White Land", "{T}: Add {R} or {W}."), Good), (Filler('W', 2, 202), Middling), (Spell("Rock", "{2}", CardTypeView.Artifact, "{T}: Add {C}."), Good + 5)),
            Pack(3, 1, pool, (Filler('G', 6, 203), Bomb), (Filler('R', 1, 204), Middling), (Filler('W', 5, 205), Good)),
        ];

        foreach (DraftMessage pack in packs)
        {
            // The pool grows with each pick, as a seat's does.
            pack.Pool = [.. pool];
            DraftChoice quiet = silent.Choose(pack);
            DraftChoice told = explaining.Choose(pack, explain: true);
            Assert.Equal(quiet.Places, told.Places);
            Assert.Equal(quiet.Scores, told.Scores);
            Assert.Equal(quiet.Beliefs, told.Beliefs);
            Assert.Empty(quiet.Why);
            Assert.NotEmpty(told.Why);
            pool.Add(pack.Pile[quiet.Places[0]]);
        }

        DraftMessage grid = Grid(10, pool,
            (Filler('U', 3, 210), Strong), (Filler('U', 2, 211), Strong + 1), (Filler('U', 4, 212), Strong + 2),
            (Filler('R', 3, 213), Good + 30), (Filler('W', 2, 214), Good + 31), (Filler('R', 2, 215), Good + 32),
            (Filler('G', 3, 216), Poor), (Filler('G', 2, 217), Poor + 1), (Filler('B', 4, 218), Poor + 2));
        List<string> why = [];
        Assert.Equal(silent.ChooseLine(grid), explaining.ChooseLine(grid, why));
        Assert.NotEmpty(why);

        foreach (DraftMessage pile in new[] { Winston(pool, Filler('R', 2, 220)), Winston(pool, Filler('U', 2, 221), Filler('G', 3, 222)) })
        {
            why.Clear();
            Assert.Equal(silent.TakePile(pile), explaining.TakePile(pile, why));
            Assert.NotEmpty(why);
        }

        var building = new DeckBuildingMessage { Pool = [.. pool, .. packs.SelectMany(pack => pack.Pile).Except(pool)], Minimum = 40 };
        why.Clear();
        BuiltDeck quietDeck = silent.Build(building);
        BuiltDeck toldDeck = explaining.Build(building, why);
        Assert.Equal(quietDeck.Main, toldDeck.Main);
        Assert.Equal(quietDeck.Basics, toldDeck.Basics);
        Assert.Equal(quietDeck.Score, toldDeck.Score);
        Assert.Contains(why, line => line.StartsWith("Deck: ", StringComparison.Ordinal));
    }

    [Fact]
    public void A_deck_with_no_slots_to_fill_is_told_too_and_picks_as_a_silent_one()
    {
        // A deck of no slots has nothing for a card to take the place of but the card itself, which telling once
        // tripped over - and an explanation that fails is said to have failed rather than costing the pick.
        List<ObjectView> pool = [.. Creatures('R', 4)];
        var tuning = new DraftTuning { Slots = 0 };
        DraftMessage Make() => Pack(1, 5, pool, (Filler('R', 2, 230), Good), (Filler('U', 3, 231), Strong));
        var explaining = new CubeDrafter { Tuning = tuning };
        explaining.Learn(pool, [.. pool.Select((_, at) => Good + 1 + at)], 360);
        var quiet = new CubeDrafter { Tuning = tuning };
        quiet.Learn(pool, [.. pool.Select((_, at) => Good + 1 + at)], 360);

        DraftChoice told = explaining.Choose(Make(), explain: true);
        Assert.Equal(quiet.Choose(Make()).Places, told.Places);
        Assert.NotEmpty(told.Why);
    }
}
