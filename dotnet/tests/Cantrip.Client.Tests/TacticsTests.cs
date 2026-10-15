using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the tactical seat reads off a card, and how it works a combat out: the two things its decisions stand on.
/// </summary>
public sealed class CardReadingTests
{
    [Fact]
    public void Keywords_are_read_off_a_keyword_line_and_not_off_a_sentence_that_mentions_one()
    {
        ObjectView pride = Creature(1, "Ocelot Pride", 1, 1, text: "First strike, lifelink\nAscend (If you control ten or more permanents, you get the city's blessing.)");
        ObjectView anthem = Creature(2, "Captain", 2, 2, text: "Other creatures you control have flying.");

        Assert.Equal(Keywords.FirstStrike | Keywords.Lifelink, CardReading.KeywordsOf(pride));
        Assert.Equal(Keywords.None, CardReading.KeywordsOf(anthem));
    }

    [Fact]
    public void A_keyword_this_does_not_know_does_not_hide_the_ones_beside_it()
    {
        Assert.Equal(Keywords.Flying, CardReading.KeywordsOf(Creature(1, "Drake", 2, 2, text: "Flying, ward {2}")));
        Assert.Equal(
            Keywords.Trample | Keywords.Indestructible,
            CardReading.KeywordsOf(Creature(2, "Colossus", 11, 11, text: "Trample, infect, indestructible")));
    }

    [Fact]
    public void Keywords_come_from_what_an_effect_granted_and_from_keyword_counters_too()
    {
        ObjectView creature = Creature(1, "Human", 1, 1);
        creature.GrantedAbilities.Add(new AbilityTextView { Text = "Trample" });
        creature.Counters.Add(new CounterView { Name = "flying", Count = 1 });

        Assert.Equal(Keywords.Trample | Keywords.Flying, CardReading.KeywordsOf(creature));
    }

    [Theory]
    [InlineData("Lightning Bolt deals 3 damage to any target.", EffectKind.Damage, 3)]
    [InlineData("Flame Slash deals 4 damage to target creature.", EffectKind.Damage, 4)]
    [InlineData("Exile target creature. Its controller gains life equal to its power.", EffectKind.Remove, 0)]
    [InlineData("Destroy target artifact.", EffectKind.Remove, 0)]
    [InlineData("Return target creature to its owner's hand.", EffectKind.Bounce, 0)]
    [InlineData("Target creature gets -3/-3 until end of turn.", EffectKind.Shrink, 3)]
    [InlineData("Counter target spell.", EffectKind.Counter, 0)]
    [InlineData("{T}: Target creature you control gains protection from the color of your choice until end of turn.", EffectKind.Protect, 0)]
    [InlineData("Put a +1/+1 counter on target creature you control.", EffectKind.Help, 0)]
    [InlineData("Target player draws three cards.", EffectKind.Help, 0)]
    [InlineData("Target creature gains flying until end of turn.", EffectKind.Help, 0)]
    [InlineData("Put target creature card from a graveyard onto the battlefield under your control.", EffectKind.Raise, 1)]
    [InlineData("Target player discards two cards.", EffectKind.Harm, 2)]
    [InlineData("Target player reveals their hand. You choose a nonland card from it. That player discards that card.", EffectKind.Harm, 0)]
    [InlineData("Pyrokinesis deals 4 damage divided as you choose among any number of target creatures.", EffectKind.Damage, 4)]
    [InlineData("Put target nonland permanent into its owner's library just beneath the top X cards of that library.", EffectKind.Bounce, 0)]
    [InlineData("Target player takes an extra turn after this one.", EffectKind.Help, 0)]
    [InlineData("Draw two cards.", EffectKind.Unknown, 0)]
    public void An_effect_is_read_off_its_words(string text, EffectKind kind, int amount)
    {
        EffectRead effect = CardReading.Effect(text);

        Assert.Equal(kind, effect.Kind);
        Assert.Equal(amount, effect.Amount);
    }

    [Fact]
    public void A_draw_is_read_off_whichever_line_it_is_on()
    {
        Assert.Equal(3, CardReading.Draws("Delve (Each card you exile from your graveyard while casting this spell pays for {1}.)\nDraw three cards."));
        Assert.Equal(1, CardReading.Draws("{2}, Sacrifice this token: Draw a card."));
        Assert.Equal(0, CardReading.Draws("Target opponent discards a card."));
    }

    [Fact]
    public void A_ritual_is_known_for_one_and_a_cantrip_that_makes_mana_is_not()
    {
        Assert.True(CardReading.Ritual("Add {B}{B}{B}."));
        Assert.False(CardReading.Ritual("Add two mana in any combination of colors.\nDraw a card."));
        Assert.False(CardReading.Ritual("Lightning Bolt deals 3 damage to any target."));
    }

    [Fact]
    public void A_kicker_is_read_with_what_it_costs_and_what_it_changes()
    {
        const string Burst = "Kicker {4} (You may pay an additional {4} as you cast this spell.)\nBurst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.";

        Assert.Equal(4, CardReading.KickerCost(Burst));
        Assert.Equal(new EffectRead(EffectKind.Damage, 2, Kicked: 4), CardReading.Effect(Burst));
    }

    [Fact]
    public void Swords_is_read_with_the_life_it_gives_back()
    {
        Assert.True(CardReading.Effect("Exile target creature. Its controller gains life equal to its power.").OwnerGainsPower);
        Assert.False(CardReading.Effect("Exile target creature. Its controller may search their library for a basic land card.").OwnerGainsPower);
    }

    [Fact]
    public void A_modal_card_is_read_one_mode_at_a_time()
    {
        IReadOnlyList<string> modes = CardReading.Modes("Choose one —\n• Abrade deals 3 damage to target creature.\n• Destroy target artifact.");

        Assert.Equal(2, modes.Count);
        Assert.Equal(EffectKind.Damage, CardReading.Effect(modes[0]).Kind);
        Assert.Equal(EffectKind.Remove, CardReading.Effect(modes[1]).Kind);
    }

    [Theory]
    [InlineData("{1}{R}", 2)]
    [InlineData("{3}{R}{R}, Exile this card from your graveyard", 5)]
    [InlineData("{T}, Pay 1 life", 0)]
    [InlineData("{R/W}{R/W}{R/W}", 3)]
    public void A_cost_is_counted_in_mana(string cost, int value) => Assert.Equal(value, CardReading.ManaValue(cost));

    [Fact]
    public void An_ability_that_asks_its_creature_to_be_something_says_what()
    {
        const string Second = "{R/W}{R/W}{R/W}: If this creature is a Spirit, it becomes a Kithkin Spirit Warrior with base power and toughness 4/4.";

        Assert.Equal("spirit", CardReading.RequiredType(Second));
        Assert.Equal((4, 4), CardReading.BaseSize(Second));
        Assert.Null(CardReading.RequiredType("{R/W}: This creature becomes a Kithkin Spirit with base power and toughness 2/2."));
    }
}

public sealed class CombatTests
{
    private static readonly TacticalModel Model = TacticalModel.Prior();

    private static Unit A(int handle, int power, int toughness, Keywords keys = Keywords.None, double worth = 0.5) => new()
    {
        Handle = Handle(handle),
        Power = power,
        Toughness = toughness,
        Keys = keys,
        Worth = worth,
    };

    [Fact]
    public void An_unblocked_attacker_deals_its_power_to_the_player()
    {
        var combat = new Combat(Model, [A(1, 3, 3)], [], attackerLife: 20, defenderLife: 20);

        Combat.Result result = combat.BestBlocks(attacking: 1);

        Assert.Equal(3, result.Damage);
        Assert.Equal(0, result.AttackersDead);
    }

    [Fact]
    public void A_first_striker_kills_its_blocker_before_the_blocker_hits_back()
    {
        var combat = new Combat(Model, [A(1, 2, 1, Keywords.FirstStrike)], [A(2, 2, 2)], 20, 20);

        Combat.Result result = combat.Evaluate(attacking: 1, blocks: [0]);

        Assert.Equal(0, result.AttackersDead);
        Assert.Equal(1, result.BlockersDead);
    }

    [Fact]
    public void Deathtouch_kills_whatever_it_touches_and_trample_carries_the_rest_over()
    {
        var combat = new Combat(Model, [A(1, 5, 5, Keywords.Deathtouch | Keywords.Trample)], [A(2, 4, 4)], 20, 20);

        Combat.Result result = combat.Evaluate(attacking: 1, blocks: [0]);

        Assert.Equal(1, result.BlockersDead);
        Assert.Equal(4, result.Damage);
    }

    [Theory]
    [InlineData(3, 0, 5, Keywords.None, 0)]            // a 3/3 trampler into a 0/5: nothing gets through
    [InlineData(5, 2, 2, Keywords.Indestructible, 3)]  // lethal is still owed to a blocker that will not die
    [InlineData(5, 2, 2, Keywords.None, 3)]
    public void Trample_sends_through_only_what_is_left_after_lethal_to_the_blocker(
        int power, int blockerPower, int blockerToughness, Keywords blockerKeys, int through)
    {
        var combat = new Combat(Model, [A(1, power, power, Keywords.Trample)], [A(2, blockerPower, blockerToughness, blockerKeys)], 20, 20);

        Assert.Equal(through, combat.Evaluate(attacking: 1, blocks: [0]).Damage);
    }

    [Fact]
    public void Trample_past_two_blockers_needs_lethal_to_both()
    {
        // Five power into a 2/2 and a 4/4 is not lethal to both, so none of it reaches the player.
        var combat = new Combat(Model, [A(1, 5, 5, Keywords.Trample)], [A(2, 2, 2), A(3, 4, 4)], 20, 20);

        Assert.Equal(0, combat.Evaluate(attacking: 1, blocks: [0, 0]).Damage);
    }

    [Fact]
    public void A_double_striker_finishes_in_the_second_step_what_it_wounded_in_the_first()
    {
        // Three and three more is six into a 5/5, which hits back for five only once.
        var combat = new Combat(Model, [A(1, 3, 6, Keywords.DoubleStrike)], [A(2, 5, 5)], 20, 20);

        Combat.Result result = combat.Evaluate(attacking: 1, blocks: [0]);

        Assert.Equal(1, result.BlockersDead);
        Assert.Equal(0, result.AttackersDead);
    }

    [Fact]
    public void A_blocker_does_not_hit_an_attacker_that_first_strike_already_killed()
    {
        // A first-striking deathtouch blocker kills the attacker before the lifelink blocker beside it ever swings.
        var combat = new Combat(
            Model,
            [A(1, 4, 4)],
            [A(2, 1, 1, Keywords.FirstStrike | Keywords.Deathtouch), A(3, 2, 2, Keywords.Lifelink)],
            20,
            20);

        Combat.Result result = combat.Evaluate(attacking: 1, blocks: [0, 0]);

        Assert.Equal(1, result.AttackersDead);
        Assert.Equal(0, result.DefenderGains);
        Assert.Equal(0, result.BlockersDead);
    }

    [Fact]
    public void Lifelink_gains_its_controller_the_damage_dealt()
    {
        var combat = new Combat(Model, [A(1, 3, 3, Keywords.Lifelink)], [A(2, 1, 1, Keywords.Lifelink)], 20, 20);

        Combat.Result result = combat.Evaluate(attacking: 1, blocks: [0]);

        Assert.Equal(3, result.AttackerGains);
        Assert.Equal(1, result.DefenderGains);
    }

    [Fact]
    public void The_defender_takes_the_block_that_kills_for_nothing()
    {
        // A 2/2 attacks into a 3/3: blocking kills it and loses nothing.
        var combat = new Combat(Model, [A(1, 2, 2)], [A(2, 3, 3)], 20, 20);

        Combat.Result result = combat.BestBlocks(attacking: 1);

        Assert.Equal([0], result.Blocks);
        Assert.Equal(1, result.AttackersDead);
        Assert.Equal(0, result.BlockersDead);
    }

    [Fact]
    public void A_chump_block_is_made_at_three_life_and_not_at_twenty()
    {
        Unit giant = A(1, 5, 5, worth: 1.2);
        Unit token = A(2, 1, 1, worth: 0.3);

        Combat.Result healthy = new Combat(Model, [giant], [token], 20, defenderLife: 20).BestBlocks(1);
        Combat.Result dying = new Combat(Model, [giant], [token], 20, defenderLife: 3).BestBlocks(1);

        Assert.Equal([-1], healthy.Blocks);
        Assert.Equal([0], dying.Blocks);
    }

    [Fact]
    public void Two_blockers_are_put_on_an_attacker_neither_could_kill_alone()
    {
        // A 4/4 into two 2/2s, with the defender low enough that the damage matters: together they kill it.
        var combat = new Combat(Model, [A(1, 4, 4, worth: 1.0)], [A(2, 2, 2, worth: 0.4), A(3, 2, 2, worth: 0.4)], 20, defenderLife: 8);

        Combat.Result result = combat.BestBlocks(attacking: 1);

        Assert.Equal([0, 0], result.Blocks);
        Assert.Equal(1, result.AttackersDead);
    }

    [Fact]
    public void An_attacker_with_menace_is_never_blocked_by_one_creature()
    {
        var combat = new Combat(Model, [A(1, 2, 2, Keywords.Menace)], [A(2, 5, 5)], 20, 20);

        Combat.Result result = combat.BestBlocks(attacking: 1);

        Assert.Equal([-1], result.Blocks);
    }

    [Fact]
    public void A_flier_is_not_blocked_from_the_ground()
    {
        var combat = new Combat(Model, [A(1, 2, 2, Keywords.Flying)], [A(2, 5, 5), A(3, 1, 3, Keywords.Reach)], 20, 20);

        Combat.Result result = combat.BestBlocks(attacking: 1);

        Assert.Equal(-1, result.Blocks[0]);
        Assert.Equal(0, result.Blocks[1]);
    }

    [Fact]
    public void Damage_that_would_end_the_game_is_scored_as_winning_it()
    {
        var combat = new Combat(Model, [A(1, 4, 4)], [], 20, defenderLife: 4);

        Assert.True(combat.BestBlocks(attacking: 1).Lethal);
    }

    [Fact]
    public void A_crowded_board_is_climbed_to_the_same_blocks_a_full_search_finds()
    {
        List<Unit> attackers = [A(1, 3, 3), A(2, 2, 2), A(3, 4, 4, worth: 1.0), A(4, 1, 1, worth: 0.2)];
        List<Unit> blockers = [A(5, 3, 4), A(6, 2, 3), A(7, 1, 1, worth: 0.2), A(8, 4, 5, worth: 1.0)];

        Combat.Result searched = new Combat(Model, attackers, blockers, 20, 9).BestBlocks(0b1111, budget: 100_000);
        Combat.Result climbed = new Combat(Model, attackers, blockers, 20, 9).BestBlocks(0b1111, budget: 1);

        // Climbing is allowed to be a little worse than exhaustive search; it is not allowed to be much worse.
        Assert.True(climbed.Value <= searched.Value + 0.25, $"climbed to {climbed.Value:N2}, a full search finds {searched.Value:N2}");
    }
}

public sealed class TacticalModelTests
{
    [Fact]
    public void A_model_written_out_is_read_back_the_same()
    {
        TacticalModel model = TacticalModel.Prior().With([KeyValuePair.Create("card:Mother of Runes", 0.4321)]);

        TacticalModel read = TacticalModel.Parse(model.Write("A heading\nof two lines"));

        Assert.Equal(model.Weights.OrderBy(pair => pair.Key), read.Weights.OrderBy(pair => pair.Key));
    }

    [Fact]
    public void Life_is_worth_more_the_less_of_it_there_is()
    {
        TacticalModel model = TacticalModel.Prior();

        Assert.Equal(0, model.Life(0));
        Assert.True(model.Life(5) - model.Life(2) > model.Life(20) - model.Life(17));
        Assert.True(model.Life(20) > model.Life(10));
    }

    [Fact]
    public void A_position_is_worth_to_one_seat_what_it_costs_the_other()
    {
        // The model is symmetric: the same table seen from the other side has every feature negated.
        List<ObjectView> table = [Creature(1, "Hero of Bladehold", 3, 4), Creature(2, "Monk", 1, 1, Them), Land(3), Land(4, controller: Them), Land(5, controller: Them)];
        PlayerView mine = View(table, 14, 9, Me, TurnStepView.PrecombatMain, 6);
        PlayerView theirs = View(table, 14, 9, Me, TurnStepView.PrecombatMain, 6);
        theirs.Viewer = Them;

        Dictionary<string, double> fromHere = TacticalModel.Features(mine);
        Dictionary<string, double> fromThere = TacticalModel.Features(theirs);

        Assert.Equal(fromHere.Keys.Order(), fromThere.Keys.Order());
        Assert.All(fromHere, pair => Assert.Equal(-pair.Value, fromThere[pair.Key]));
    }

    [Fact]
    public void The_shipped_model_has_a_weight_for_everything_the_seat_reads()
    {
        TacticalModel shipped = TacticalModel.Default;

        // Whatever a fit or a tuning did to the numbers, every weight the prior names is still there to be read: a
        // model file that lost one would have the seat play with a zero where it expects a value.
        Assert.All(TacticalModel.Prior().Weights.Keys, name => Assert.True(shipped.Weights.ContainsKey(name), name));
        Assert.All(shipped.Weights.Values, value => Assert.True(double.IsFinite(value)));
    }
}
