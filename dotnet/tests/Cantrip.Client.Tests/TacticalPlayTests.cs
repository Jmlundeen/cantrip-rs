using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the tactical seat does with a board.
/// <para>
/// Each test writes the position it is about — the view and the prompt, which are all the seat ever reads — and asks
/// for the one answer. The seat is deterministic unless told to explore, so an answer here is the answer it gives in
/// a game; and the weights are the hand-written prior, so a refit of the shipped model cannot change what these say.
/// </para>
/// </summary>
public sealed class TacticalPlayTests
{
    private const string Bolt = "Lightning Bolt deals 3 damage to any target.";

    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ActionOptionMessage Cast(int source, params TargetMessage[] candidates) => new()
    {
        Kind = ActionKindView.CastSpell,
        Source = Handle(source),
        Affordable = true,
        TargetsRequired = candidates.Length > 0 ? 1 : 0,
        TargetCandidates = [.. candidates],
    };

    private static PriorityPromptMessage Priority(params ActionOptionMessage[] options) => new() { Player = Me, Options = [.. options] };

    // --- Priority ------------------------------------------------------------------------------------------------------

    [Fact]
    public void Plays_its_land_before_it_casts_anything()
    {
        GameClient client = Seated(
        [
            Land(1),
            InHand(10, "Mountain", CardTypeView.Land, 0),
            InHand(11, "Goblin", CardTypeView.Creature, 1, power: 1, toughness: 1),
        ]);

        AnswerPayload answer = Bot().Decide(client, Priority(
            Cast(11),
            new ActionOptionMessage { Kind = ActionKindView.PlayLand, Source = Handle(10) }));

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(answer);
        Assert.Equal(ActionKindView.PlayLand, action.Kind);
    }

    [Fact]
    public void Casts_the_creature_it_can_pay_for()
    {
        GameClient client = Seated([Land(1), Land(2), InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(11))));

        Assert.Equal(ActionKindView.CastSpell, action.Kind);
        Assert.Equal(Handle(11), action.Source);
    }

    [Fact]
    public void Casts_the_two_spells_that_use_its_mana_rather_than_the_one_that_wastes_it()
    {
        // Three lands: a three-drop alone, or a two-drop and a one-drop that are worth more between them.
        GameClient client = Seated(
        [
            Land(1), Land(2), Land(3),
            InHand(11, "Ogre", CardTypeView.Creature, 3, power: 3, toughness: 2),
            InHand(12, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2),
            InHand(13, "Goblin", CardTypeView.Creature, 1, power: 2, toughness: 1),
        ]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(11), Cast(12), Cast(13))));

        Assert.NotEqual(Handle(11), action.Source);
    }

    [Fact]
    public void Aims_removal_at_the_creature_worth_most_that_it_kills()
    {
        GameClient client = Seated(
        [
            Land(1),
            Creature(2, "Mine", 3, 3),
            Creature(3, "Token", 1, 1, Them),
            Creature(4, "Knight", 3, 3, Them),
            Creature(5, "Giant", 6, 6, Them),
            InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt),
        ]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(
            Cast(10, At(2), At(3), At(4), At(5), AtPlayer(Me), AtPlayer(Them)))));

        // Not the giant it cannot kill, not the token worth less than the card, and never its own.
        Assert.Equal([Handle(4)], action.Targets.Select(target => target.Object));
    }

    [Fact]
    public void Sends_burn_at_a_player_it_would_kill()
    {
        GameClient client = Seated(
            [Land(1), Creature(4, "Knight", 3, 3, Them), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)],
            theirLife: 3);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10, At(4), AtPlayer(Me), AtPlayer(Them)))));

        Assert.Equal([Them], action.Targets.Select(target => target.Player));
    }

    [Fact]
    public void Two_burn_spells_that_add_up_to_lethal_are_both_sent_at_the_player()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2),
            Creature(4, "Knight", 3, 3, Them),
            InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt),
            InHand(11, "Chain Lightning", CardTypeView.Sorcery, 1, "Chain Lightning deals 3 damage to any target."),
        ], theirLife: 6);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(
            Cast(10, At(4), AtPlayer(Them)), Cast(11, At(4), AtPlayer(Them)))));

        Assert.Equal([Them], action.Targets.Select(target => target.Player));
    }

    [Fact]
    public void Keeps_burn_in_hand_when_there_is_nothing_worth_a_card_to_hit()
    {
        GameClient client = Seated([Land(1), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10, AtPlayer(Me), AtPlayer(Them)))));
    }

    [Fact]
    public void Never_spends_removal_on_its_own_creature()
    {
        GameClient client = Seated([Land(1), Creature(2, "Mine", 2, 2), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10, At(2)))));
    }

    [Fact]
    public void Lets_its_own_spell_resolve_rather_than_answering_it()
    {
        ObjectView onStack = InHand(20, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2);
        onStack.Zone = ZoneKindView.Stack;
        onStack.ZoneOwner = 0;

        GameClient client = Seated([Land(1), onStack, InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)], theirLife: 3);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10, AtPlayer(Them)))));
    }

    [Fact]
    public void Does_nothing_at_priority_in_its_own_upkeep()
    {
        GameClient client = Seated(
            [Land(1), Creature(4, "Knight", 3, 3, Them), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)],
            step: TurnStepView.Upkeep);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10, At(4)))));
    }

    [Fact]
    public void Kills_an_attacker_once_the_opponent_has_declared_it()
    {
        GameClient client = Seated(
            [Land(1), Creature(4, "Knight", 3, 3, Them, tapped: true), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, Bolt)],
            active: Them,
            step: TurnStepView.DeclareAttackers);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10, At(4), AtPlayer(Them)))));

        Assert.Equal([Handle(4)], action.Targets.Select(target => target.Object));
    }

    [Fact]
    public void An_ability_that_changed_nothing_is_not_tried_again_that_turn()
    {
        // The seat has no rules engine to tell it that a second activation of Figure of Destiny does nothing. It
        // finds out by looking: the board after is the board before.
        ObjectView figure = Creature(2, "Figure", 2, 2);
        var table = new Table([Land(1), figure]);
        TacticalPlay bot = Bot();

        PriorityPromptMessage Offered() => Priority(new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility,
            Source = Handle(2),
            AbilityIndex = 0,
            Affordable = true,
            Description = "{R}: Put a +1/+1 counter on this creature.",
        });

        Assert.IsType<TakeActionAnswer>(bot.Decide(table.Client, Offered()));

        // The land is tapped and nothing else is different.
        table.Show([Land(1, tapped: true), figure]);
        Assert.IsType<PassAnswer>(bot.Decide(table.Client, Offered()));

        // The next turn it is worth trying again.
        table.Show([Land(1), figure], turn: 7);
        Assert.IsType<TakeActionAnswer>(bot.Decide(table.Client, Offered()));
    }

    [Fact]
    public void An_ability_that_asks_its_creature_to_be_something_it_is_not_is_left_alone()
    {
        GameClient client = Seated([Land(1), Land(2), Land(3), Creature(2, "Figure of Destiny", 1, 1)]);

        AnswerPayload answer = Bot().Decide(client, Priority(new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility,
            Source = Handle(2),
            AbilityIndex = 1,
            Affordable = true,
            Description = "{R/W}{R/W}{R/W}: If this creature is a Spirit, it becomes a Kithkin Spirit Warrior with base power and toughness 4/4.",
        }));

        Assert.IsType<PassAnswer>(answer);
    }

    [Fact]
    public void Cracks_a_Clue_with_mana_it_has_no_other_use_for()
    {
        ObjectView clue = new()
        {
            Handle = Handle(3),
            Zone = ZoneKindView.Battlefield,
            Kind = ObjectKindView.Token,
            Owner = Me,
            Controller = Me,
            Name = "Clue Token",
            CardTypes = [CardTypeView.Artifact],
        };

        GameClient client = Seated([Land(1), Land(2), clue]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility,
            Source = Handle(3),
            Affordable = true,
            Description = "{2}, Sacrifice this token: Draw a card.",
        })));

        Assert.Equal(Handle(3), action.Source);
    }

    [Theory]
    [InlineData(3, false)]
    [InlineData(7, true)]
    public void Gives_up_a_land_for_a_card_only_once_it_has_lands_to_spare(int lands, bool cashedIn)
    {
        ObjectView canyon = Land(1, "Sunbaked Canyon");
        canyon.Supertypes = [];
        canyon.Subtypes = [];

        GameClient client = Seated([canyon, .. Enumerable.Range(2, lands - 1).Select(handle => Land(handle))]);

        AnswerPayload answer = Bot().Decide(client, Priority(new ActionOptionMessage
        {
            Kind = ActionKindView.ActivateAbility,
            Source = Handle(1),
            AbilityIndex = 1,
            Affordable = true,
            Description = "{1}, {T}, Sacrifice this land: Draw a card.",
        }));

        Assert.Equal(cashedIn, answer is TakeActionAnswer);
    }

    [Fact]
    public void Gives_up_an_optional_cost_that_tapping_gets_it_no_nearer_to_paying()
    {
        // Flash's "sacrifice it unless you pay", with the only source on offer one that costs mana to use: the same
        // question comes back with the same amount owed. Twice is enough to know it cannot be paid.
        GameClient client = Seated([Land(1, tapped: true), Creature(2, "Sphinx", 4, 4)]);
        TacticalPlay bot = Bot();

        UnlessPaysPromptMessage Asked() => new()
        {
            Player = Me,
            Source = Handle(30),
            Subject = Handle(2),
            Total = "{1}{U}{U}",
            Remaining = "{1}{U}{U}",
            RemainingGeneric = 1,
            RemainingColored = [ColorView.Blue, ColorView.Blue],
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(9), ProducesOfAnyOneColor = 1, ColorChoices = [ColorView.Blue] },
                new() { Kind = ActionKindView.DeclineCost },
            ],
        };

        List<ActionKindView> answers = [.. Enumerable.Range(0, 4).Select(_ => Assert.IsType<TakeActionAnswer>(bot.Decide(client, Asked())).Kind)];

        Assert.Equal(ActionKindView.ActivateAbility, answers[0]);
        Assert.Equal(ActionKindView.DeclineCost, answers[^1]);
    }

    [Fact]
    public void A_card_it_chose_to_cast_and_is_offered_again_is_not_chosen_again()
    {
        // Etali offers a card to cast; the cast cannot be finished (a spree mode it cannot pay for) and the same offer
        // comes back. Choosing it again would only bring the offer back again.
        ObjectView exiled = InHand(20, "Three Steps Ahead", CardTypeView.Instant, 1, "Spree");
        exiled.Zone = ZoneKindView.Exile;

        GameClient client = Seated([exiled]);
        TacticalPlay bot = Bot();

        ChooseCardsPromptMessage Offered() => new()
        {
            Player = Me, Owner = Me, Purpose = CardChoicePurposeView.Cast, Cards = [Handle(20)], Eligible = [Handle(20)], Count = 1, UpTo = true,
        };

        Assert.Equal([Handle(20)], Assert.IsType<CardsChosenAnswer>(bot.Decide(client, Offered())).Cards);
        Assert.Empty(Assert.IsType<CardsChosenAnswer>(bot.Decide(client, Offered())).Cards);
    }

    [Fact]
    public void Answers_with_the_prompts_own_once_a_step_has_gone_on_far_too_long()
    {
        // Whatever two questions might lead back to each other, a step ends: past a count no honest step reaches,
        // every answer is the prompt's own, which for priority is a pass.
        GameClient client = Seated([Land(1), Land(2), InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)]);
        TacticalPlay bot = Bot();
        var may = new MayPromptMessage { Player = Me, Source = Handle(11), Verb = "Draw" };

        for (int asked = 0; asked < 400; asked++)
        {
            Assert.True(Assert.IsType<MayChosenAnswer>(bot.Decide(client, may)).Accept);
        }

        Assert.False(Assert.IsType<MayChosenAnswer>(bot.Decide(client, may)).Accept);
        Assert.IsType<PassAnswer>(bot.Decide(client, Priority(Cast(11))));
    }

    [Fact]
    public void Does_not_cast_a_ritual_for_nothing_or_a_sweeper_that_sweeps_only_its_own()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2), Land(3), Land(4), Creature(5, "Mine", 3, 3),
            InHand(10, "Dark Ritual", CardTypeView.Instant, 1, "Add {B}{B}{B}."),
            InHand(11, "Wrath of God", CardTypeView.Sorcery, 4, "Destroy all creatures. They can't be regenerated."),
            InHand(12, "Upheaval", CardTypeView.Sorcery, 4, "Return all permanents to their owners' hands."),
        ]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10), Cast(11), Cast(12))));
    }

    [Fact]
    public void Casts_a_sweeper_when_it_is_the_opponent_who_has_the_board()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2), Land(3), Land(4), Creature(5, "Theirs", 3, 3, Them), Creature(6, "Theirs", 4, 4, Them),
            InHand(11, "Wrath of God", CardTypeView.Sorcery, 4, "Destroy all creatures. They can't be regenerated."),
        ]);

        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(11)))).Source);
    }

    [Fact]
    public void Keeps_mana_back_for_a_counterspell_rather_than_tapping_out_for_a_small_play()
    {
        // Four lands, a Counterspell and a one-drop: the one-drop is cast and two lands stay up. With the same hand
        // and a play worth tapping out for, it taps out.
        ObjectView counter = InHand(10, "Counterspell", CardTypeView.Instant, 2, "Counter target spell.", color: ColorView.Blue);
        List<ObjectView> lands = [Land(1), Land(2), Land(3), Land(4)];

        GameClient small = Seated([.. lands, counter, InHand(11, "Ogre", CardTypeView.Creature, 3, power: 2, toughness: 2)]);
        GameClient big = Seated([.. lands, counter, InHand(12, "Dragon", CardTypeView.Creature, 4, power: 6, toughness: 6)]);

        Assert.IsType<PassAnswer>(Bot().Decide(small, Priority(Cast(11))));
        Assert.Equal(Handle(12), Assert.IsType<TakeActionAnswer>(Bot().Decide(big, Priority(Cast(12)))).Source);
    }

    [Fact]
    public void Kicks_a_spell_when_only_the_kicked_one_kills()
    {
        const string Burst = "Kicker {4} (You may pay an additional {4} as you cast this spell.)\nBurst Lightning deals 2 damage to any target. If this spell was kicked, it deals 4 damage instead.";

        GameClient client = Seated(
        [
            Land(1), Land(2), Land(3), Land(4), Land(5),
            Creature(6, "Knight", 3, 4, Them),
            InHand(10, "Burst Lightning", CardTypeView.Instant, 1, Burst),
        ]);

        TacticalPlay bot = Bot();
        TakeActionAnswer cast = Assert.IsType<TakeActionAnswer>(bot.Decide(client, Priority(Cast(10, At(6), AtPlayer(Them)))));
        Assert.Equal([Handle(6)], cast.Targets.Select(target => target.Object));

        // Then the announcement (CR 601.2b): one kick, where the server says one can be paid for.
        TakeActionAnswer announced = Assert.IsType<TakeActionAnswer>(bot.Decide(client, new PayCostsPromptMessage
        {
            Player = Me,
            Spell = Handle(10),
            AnnouncesXOrKicks = true,
            Kicker = "{4}",
            HasX = true,
            X = 0,
            MostAffordable = 1,
            Options =
            [
                new() { Kind = ActionKindView.IncreaseX },
                new() { Kind = ActionKindView.ConfirmCasting },
                new() { Kind = ActionKindView.CancelCasting },
            ],
        }));

        Assert.Equal(ActionKindView.IncreaseX, announced.Kind);
    }

    // --- Combat --------------------------------------------------------------------------------------------------------

    private static DeclareAttackersPromptMessage Attackers(params int[] eligible) => new()
    {
        Player = Me,
        Eligible = [.. eligible.Select(Handle)],
        AttackablePlayers = [Them],
    };

    private static List<int> Attacking(AnswerPayload answer) =>
        [.. Assert.IsType<AttackersDeclaredAnswer>(answer).Attacks.Select(attack => attack.Attacker.Value).Order()];

    [Fact]
    public void Attacks_a_player_with_nothing_to_block_with()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Creature(2, "Goblin", 1, 1)], step: TurnStepView.DeclareAttackers);

        Assert.Equal([1, 2], Attacking(Bot().Decide(client, Attackers(1, 2))));
    }

    [Fact]
    public void Does_not_attack_into_a_blocker_that_would_eat_the_attacker()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Creature(3, "Wall", 3, 4, Them)], step: TurnStepView.DeclareAttackers);

        Assert.Empty(Attacking(Bot().Decide(client, Attackers(1))));
    }

    [Fact]
    public void Attacks_past_a_blocker_with_a_creature_it_cannot_block()
    {
        GameClient client = Seated(
            [Creature(1, "Bear", 2, 2), Creature(2, "Hawk", 2, 1, text: "Flying"), Creature(3, "Wall", 3, 4, Them)],
            step: TurnStepView.DeclareAttackers);

        Assert.Equal([2], Attacking(Bot().Decide(client, Attackers(1, 2))));
    }

    [Fact]
    public void Attacks_with_everything_when_that_is_lethal_whatever_is_blocked()
    {
        // Three attackers into one blocker at four life: one is blocked, and the other two are enough.
        GameClient client = Seated(
            [Creature(1, "Bear", 2, 2), Creature(2, "Bear", 2, 2), Creature(3, "Bear", 2, 2), Creature(4, "Wall", 3, 4, Them)],
            theirLife: 4,
            step: TurnStepView.DeclareAttackers);

        Assert.Equal([1, 2, 3], Attacking(Bot().Decide(client, Attackers(1, 2, 3))));
    }

    [Fact]
    public void Keeps_its_blocker_home_when_attacking_would_leave_it_dead_on_the_way_back()
    {
        // At three life, facing a tapped 3/3 that untaps to attack: the 3/3 that could attack for free has to stay
        // back and block it.
        GameClient client = Seated(
            [Creature(1, "Knight", 3, 3), Creature(4, "Brute", 3, 3, Them, tapped: true)],
            myLife: 3,
            step: TurnStepView.DeclareAttackers);

        Assert.Empty(Attacking(Bot().Decide(client, Attackers(1))));
    }

    [Fact]
    public void A_board_of_dozens_of_tokens_is_still_answered()
    {
        // Twenty attackers into twenty blockers is more combats than could ever be tried, so the search plans around
        // the biggest of each and stops at a fixed amount of work. What this asserts is the part that can be: that an
        // answer comes back, and names only creatures that could attack.
        List<ObjectView> table =
        [
            .. Enumerable.Range(1, 20).Select(i => Creature(i, "Monk", 1 + (i % 3), 1 + (i % 2))),
            .. Enumerable.Range(101, 20).Select(i => Creature(i, "Soldier", 1 + (i % 2), 1 + (i % 3), Them)),
        ];

        GameClient client = Seated(table, step: TurnStepView.DeclareAttackers);
        int[] eligible = [.. Enumerable.Range(1, 20)];

        List<int> attacking = Attacking(Bot().Decide(client, Attackers(eligible)));

        Assert.All(attacking, attacker => Assert.Contains(attacker, eligible));
    }

    [Fact]
    public void A_creature_that_must_attack_does()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Creature(3, "Wall", 3, 4, Them)], step: TurnStepView.DeclareAttackers);

        DeclareAttackersPromptMessage prompt = Attackers(1);
        prompt.Requirements.Add(new AttackRequirementMessage { Attacker = Handle(1) });

        Assert.Equal([1], Attacking(Bot().Decide(client, prompt)));
    }

    private static DeclareBlockersPromptMessage Blockers(int[] attackers, int[] eligible) => new()
    {
        Player = Me,
        Attackers = [.. attackers.Select(Handle)],
        Eligible = [.. eligible.Select(Handle)],
        Allowed = [.. eligible.SelectMany(blocker => attackers.Select(attacker => new BlockMessage { Blocker = Handle(blocker), Attacker = Handle(attacker) }))],
        MinimumBlockers = [.. attackers.Select(_ => 1)],
    };

    [Fact]
    public void Blocks_where_it_kills_the_attacker_and_keeps_the_blocker()
    {
        GameClient client = Seated(
            [Creature(1, "Knight", 3, 3), Creature(4, "Bear", 2, 2, Them, tapped: true)],
            active: Them,
            step: TurnStepView.DeclareBlockers);

        BlockersDeclaredAnswer blocks = Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(client, Blockers([4], [1])));

        BlockMessage block = Assert.Single(blocks.Blocks);
        Assert.Equal((Handle(1), Handle(4)), (block.Blocker, block.Attacker));
    }

    [Fact]
    public void Takes_the_damage_rather_than_throwing_a_creature_away_while_it_has_the_life()
    {
        GameClient client = Seated(
            [Creature(1, "Goblin", 1, 1), Creature(4, "Giant", 5, 5, Them, tapped: true)],
            active: Them,
            step: TurnStepView.DeclareBlockers);

        Assert.Empty(Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(client, Blockers([4], [1]))).Blocks);
    }

    [Fact]
    public void Throws_a_creature_in_the_way_of_damage_that_would_kill_it()
    {
        GameClient client = Seated(
            [Creature(1, "Goblin", 1, 1), Creature(4, "Giant", 5, 5, Them, tapped: true)],
            myLife: 4,
            active: Them,
            step: TurnStepView.DeclareBlockers);

        Assert.Single(Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(client, Blockers([4], [1]))).Blocks);
    }

    [Fact]
    public void Only_blocks_as_the_prompt_allows()
    {
        // The prompt allows no pair at all — a flier, as far as this seat is told — so nothing is blocked however
        // good the block would have been.
        GameClient client = Seated(
            [Creature(1, "Knight", 3, 3), Creature(4, "Bear", 2, 2, Them, tapped: true)],
            active: Them,
            step: TurnStepView.DeclareBlockers);

        DeclareBlockersPromptMessage prompt = Blockers([4], [1]);
        prompt.Allowed.Clear();

        Assert.Empty(Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(client, prompt)).Blocks);
    }

    // --- Everything else -----------------------------------------------------------------------------------------------

    [Fact]
    public void A_trigger_that_helps_is_aimed_at_its_own_best_creature()
    {
        GameClient client = Seated([Creature(1, "Goblin", 1, 1), Creature(2, "Knight", 3, 3), Creature(3, "Bear", 2, 2, Them)]);

        TargetsChosenAnswer answer = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, new ChooseTargetPromptMessage
        {
            Player = Me,
            Text = "Put a +1/+1 counter on target creature you control.",
            Candidates = [At(1), At(2)],
        }));

        Assert.Equal([Handle(2)], answer.Targets.Select(target => target.Object));
    }

    [Fact]
    public void A_trigger_that_hurts_is_aimed_at_an_opponent_and_at_nothing_when_it_may_be()
    {
        GameClient client = Seated([Creature(1, "Goblin", 1, 1), Creature(3, "Bear", 2, 2, Them)]);
        const string Text = "Phlage deals 3 damage to any target and you gain 3 life.";

        TargetsChosenAnswer aimed = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, new ChooseTargetPromptMessage
        {
            Player = Me, Text = Text, Candidates = [At(1), At(3), AtPlayer(Me), AtPlayer(Them)],
        }));

        TargetsChosenAnswer declined = Assert.IsType<TargetsChosenAnswer>(Bot().Decide(client, new ChooseTargetPromptMessage
        {
            Player = Me, Text = Text, Candidates = [At(1), AtPlayer(Me)], Optional = true,
        }));

        Assert.Equal([Handle(3)], aimed.Targets.Select(target => target.Object));
        Assert.Empty(declined.Targets);
    }

    [Theory]
    [InlineData(0, true)]
    [InlineData(1, true)]
    [InlineData(3, false)]
    [InlineData(6, true)]
    public void Mulligans_a_hand_of_seven_it_could_not_play(int lands, bool mulligans)
    {
        GameClient client = Seated(
        [
            .. Enumerable.Range(0, lands).Select(i => InHand(10 + i, "Mountain", CardTypeView.Land, 0)),
            .. Enumerable.Range(0, 7 - lands).Select(i => InHand(20 + i, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2)),
        ]);

        MayChosenAnswer answer = Assert.IsType<MayChosenAnswer>(Bot().Decide(client, new MulliganPromptMessage { Player = Me, Keeps = 7 }));

        Assert.Equal(mulligans, answer.Accept);
    }

    [Fact]
    public void Discards_the_land_it_has_no_use_for_before_the_spell_it_could_cast()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2), Land(3), Land(4), Land(5), Land(6),
            InHand(10, "Mountain", CardTypeView.Land, 0),
            InHand(11, "Bear", CardTypeView.Creature, 2, power: 2, toughness: 2),
        ]);

        CardsChosenAnswer answer = Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, new DiscardPromptMessage
        {
            Player = Me, Count = 1, Hand = [Handle(10), Handle(11)],
        }));

        Assert.Equal([Handle(10)], answer.Cards);
    }

    [Fact]
    public void A_refused_answer_is_given_up_on_for_the_prompts_own()
    {
        var table = new Table([Creature(1, "Bear", 2, 2)], step: TurnStepView.DeclareAttackers);
        TacticalPlay bot = Bot();

        // The same question, id and all, is what a refusal looks like. The seat has no other attack to offer on this
        // board, so twice is enough: the third answer is the prompt's own, which attacks with nobody.
        for (int asked = 0; asked < 3; asked++)
        {
            table.Ask(7, Attackers(1));
            Assert.True(bot.Act(table.Client));
        }

        List<AnswerPayload> answers = [.. table.Sent.OfType<AnswerMessage>().Select(message => message.Answer)];
        Assert.Equal([1], Attacking(answers[0]));
        Assert.Equal([1], Attacking(answers[1]));
        Assert.Empty(Attacking(answers[2]));
        Assert.False(bot.Retrying);
    }

    [Fact]
    public void Does_not_keep_copying_a_spell_that_two_seats_would_pass_back_and_forth_for_ever()
    {
        // Chain of Smog: "Target player discards two cards. That player may copy this spell…". Two seats that always
        // said yes would never stop.
        ObjectView smog = InHand(20, "Chain of Smog", CardTypeView.Sorcery, 2,
            "Target player discards two cards. That player may copy this spell and may choose a new target for that copy.");
        smog.Zone = ZoneKindView.Stack;
        smog.ZoneOwner = 0;
        smog.Controller = Them;

        GameClient client = Seated([smog]);

        MayChosenAnswer answer = Assert.IsType<MayChosenAnswer>(Bot().Decide(client, new MayPromptMessage
        {
            Player = Me, Source = Handle(20), Verb = "CopySpell",
        }));

        Assert.False(answer.Accept);
    }

    [Fact]
    public void Copies_a_burn_spell_that_has_something_to_hit_but_only_so_many_times_a_turn()
    {
        ObjectView chain = InHand(20, "Chain Lightning", CardTypeView.Sorcery, 1,
            "Chain Lightning deals 3 damage to any target. Then that player or that permanent's controller may pay {R}{R}.");
        chain.Zone = ZoneKindView.Stack;
        chain.ZoneOwner = 0;
        chain.Controller = Them;

        GameClient client = Seated([chain, Creature(4, "Knight", 3, 3, Them)]);
        TacticalPlay bot = Bot();

        List<bool> accepted = [.. Enumerable.Range(0, 6).Select(_ =>
            Assert.IsType<MayChosenAnswer>(bot.Decide(client, new MayPromptMessage { Player = Me, Source = Handle(20), Verb = "CopySpell" })).Accept)];

        Assert.Equal([true, true, true, false, false, false], accepted);
    }

    [Fact]
    public void Does_not_cast_a_spell_it_cannot_read_at_anything()
    {
        // Whose things an unread effect is good for is not something to guess at with a card.
        GameClient client = Seated(
        [
            Land(1), Creature(2, "Mine", 2, 2), Creature(3, "Theirs", 2, 2, Them),
            InHand(10, "Riddle", CardTypeView.Instant, 1, "Target creature phases out."),
        ]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Cast(10, At(2), At(3)))));
    }

    [Fact]
    public void A_card_that_has_a_player_draw_is_aimed_at_itself()
    {
        GameClient client = Seated([Land(1), InHand(10, "Ancestral Recall", CardTypeView.Instant, 1, "Target player draws three cards.")]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10, AtPlayer(Me), AtPlayer(Them)))));

        Assert.Equal([Me], action.Targets.Select(target => target.Player));
    }

    [Fact]
    public void Divided_damage_kills_two_creatures_when_there_is_no_player_to_send_the_rest_at()
    {
        GameClient client = Seated([Creature(3, "Bear", 2, 2, Them), Creature(4, "Bear", 2, 2, Them)]);

        TargetsDividedAnswer answer = Assert.IsType<TargetsDividedAnswer>(Bot().Decide(client, new ChooseDividedTargetsPromptMessage
        {
            Player = Me,
            Text = "Fury deals 4 damage divided as you choose among any number of target creatures and/or planeswalkers.",
            Candidates = [At(3), At(4)],
            Total = 4,
        }));

        Assert.Equal([2, 2], answer.Shares.Select(share => share.Amount));
    }

    [Fact]
    public void A_mode_it_could_not_pay_for_is_not_chosen_over_one_it_could()
    {
        GameClient client = Seated([Creature(3, "Giant", 6, 6, Them)]);

        ModesChosenAnswer answer = Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, new ChooseModesPromptMessage
        {
            Player = Me,
            Modes = ["Draw two cards.", "Destroy target creature."],
            Eligible = [0, 1],
            Affordable = [0],
            Count = 1,
        }));

        Assert.Equal([0], answer.Modes);
    }

    [Fact]
    public void The_same_board_gets_the_same_answer_every_time()
    {
        List<ObjectView> table =
        [
            Creature(1, "Bear", 2, 2), Creature(2, "Hawk", 2, 1, text: "Flying"), Creature(3, "Knight", 3, 3),
            Creature(4, "Wall", 1, 4, Them), Creature(5, "Ogre", 4, 2, Them),
        ];

        List<List<int>> answers = [.. Enumerable.Range(0, 5).Select(seed =>
            Attacking(new TacticalPlay(TacticalModel.Prior(), seed).Decide(Seated(table, step: TurnStepView.DeclareAttackers), Attackers(1, 2, 3))))];

        Assert.All(answers, answer => Assert.Equal(answers[0], answer));
    }
}
