using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the tactical seat does with the permanents that are not creatures: artifacts that make mana or hold a card,
/// planeswalkers, and Vehicles.
/// <para>
/// As <see cref="TacticalPlayTests"/>: each test writes its board and asks for the one answer, on the hand-written
/// prior. Each behaviour here is behind a <c>policy.*</c> weight, so that one build can play itself with it on and
/// off; a test that sets one to zero is saying what the weight turns off.
/// </para>
/// </summary>
public sealed class TacticalPermanentsTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static TacticalPlay Without(string policy) =>
        new(TacticalModel.Prior().With([KeyValuePair.Create("policy." + policy, 0.0)]), seed: 1);

    private static PriorityPromptMessage Priority(params ActionOptionMessage[] options) => new() { Player = Me, Options = [.. options] };

    private static ActionOptionMessage Cast(int source) => new() { Kind = ActionKindView.CastSpell, Source = Handle(source), Affordable = true };

    private static ActionOptionMessage Ability(int source, int index, string description, params TargetMessage[] candidates) => new()
    {
        Kind = ActionKindView.ActivateAbility,
        Source = Handle(source),
        AbilityIndex = index,
        Affordable = true,
        Description = description,
        TargetsRequired = candidates.Length > 0 ? 1 : 0,
        TargetCandidates = [.. candidates],
    };

    private static ObjectView Artifact(int handle, string name, string text = "", int controller = Me, bool token = false, bool makesMana = false) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = token ? ObjectKindView.Token : ObjectKindView.Card,
        Owner = controller,
        Controller = controller,
        Name = name,
        CardTypes = [CardTypeView.Artifact],
        OracleText = text,
        ProducesMana = makesMana,
    };

    private static ObjectView Walker(int handle, string name, int loyalty, int controller = Me) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = controller,
        Controller = controller,
        Name = name,
        CardTypes = [CardTypeView.Planeswalker],
        Counters = [new CounterView { Name = "loyalty", Count = loyalty }],
    };

    // --- Artifacts that make mana, and ones that hold a card ---------------------------------------------------------------

    private const string Mox = "{T}: Add {G}.";
    private const string Bauble =
        "{T}, Sacrifice this artifact: Look at the top card of target player's library. Draw a card at the beginning of the next turn's upkeep.";

    [Theory]
    [InlineData("{T}: Add {G}.", 1, false)]
    [InlineData("{T}: Add {C}{C}.", 2, false)]
    [InlineData("{T}: Add {C}.\n{T}: Add {U} or {B}. This artifact deals 1 damage to you.", 1, false)]
    [InlineData("{T}: Add two mana in any combination of {U}, {B}, and/or {R}.", 2, false)]
    [InlineData("{T}, Sacrifice Black Lotus: Add three mana of any one color.", 3, true)]
    [InlineData("This artifact doesn't untap during your untap step.\n{T}: Add {C}{C}{C}.\n{3}: Untap this artifact.", 3, true)]
    [InlineData("Metalcraft — {T}: Add one mana of any color. Activate only if you control three or more artifacts.", 1, false)]
    // It gives back the mana it took.
    [InlineData("{1}, {T}, Sacrifice this artifact: Add one mana of any color.", 0, false)]
    // Each of these asks for a card, which is a trade this seat does not price.
    [InlineData("Discard your hand, Sacrifice this artifact: Add three mana of any one color. Activate only as an instant.", 0, false)]
    [InlineData("If this artifact would enter, you may discard a land card instead.\n{T}: Add one mana of any color.", 0, false)]
    // A loyalty ability is not a mana rock.
    [InlineData("+1: Add {R}{R}.", 0, false)]
    [InlineData("Flying", 0, false)]
    public void The_mana_a_permanent_makes_is_read_off_its_text(string text, int amount, bool once) =>
        Assert.Equal((amount, once), CardReading.ManaMade(text));

    [Fact]
    public void Casts_a_Mox_which_costs_nothing_and_is_a_land_more()
    {
        GameClient client = Seated([Land(1), InHand(10, "Mox Emerald", CardTypeView.Artifact, 0, Mox)]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10))));
        Assert.Equal(Handle(10), action.Source);

        // By its type and its cost alone an artifact that costs nothing is worth less than the card it is.
        Assert.IsType<PassAnswer>(Without("rocks").Decide(client, Priority(Cast(10))));
    }

    [Fact]
    public void Casts_the_mana_rock_before_the_creature_while_it_is_short_of_lands()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2),
            InHand(10, "Sol Ring", CardTypeView.Artifact, 1, "{T}: Add {C}{C}."),
            InHand(11, "Goblin", CardTypeView.Creature, 1, power: 1, toughness: 1),
        ]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Cast(10), Cast(11))));

        Assert.Equal(Handle(10), action.Source);
    }

    [Fact]
    public void Casts_a_Bauble_for_the_card_it_holds_and_cashes_it_in_at_an_opponent()
    {
        GameClient hand = Seated([Land(1), InHand(10, "Mishra's Bauble", CardTypeView.Artifact, 0, Bauble)]);

        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(Bot().Decide(hand, Priority(Cast(10)))).Source);
        Assert.IsType<PassAnswer>(Without("stored").Decide(hand, Priority(Cast(10))));

        GameClient table = Seated([Land(1), Artifact(3, "Mishra's Bauble", Bauble)]);
        TakeActionAnswer cashed = Assert.IsType<TakeActionAnswer>(Bot().Decide(table, Priority(Ability(3, 0, Bauble, AtPlayer(Me), AtPlayer(Them)))));

        Assert.Equal(ActionKindView.ActivateAbility, cashed.Kind);
        Assert.Equal([Them], cashed.Targets.Select(target => target.Player));
    }

    [Theory]
    [InlineData(1, true)]
    [InlineData(6, false)]
    public void A_Lantern_is_given_up_for_a_card_and_a_Mind_Stone_only_once_its_mana_is_not_needed(int landsBeside, bool lanternOnly)
    {
        const string Lantern = "{1}, {T}, Sacrifice this artifact: Draw a card.";

        ObjectView soulGuide = Artifact(20, "Soul-Guide Lantern", "When this artifact enters, exile target card from a graveyard.\n" + Lantern);
        soulGuide.ManaValue = 1;

        List<ObjectView> board = [.. Enumerable.Range(1, landsBeside).Select(i => Land(i))];
        board.Add(soulGuide);
        board.Add(Artifact(21, "Mind Stone", "{T}: Add {C}.\n" + Lantern, makesMana: true));
        GameClient client = Seated(board);

        // The Lantern does nothing else where it is. By its type and cost it used to be "worth" more than the card.
        TakeActionAnswer lantern = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Ability(20, 1, Lantern))));
        Assert.Equal(Handle(20), lantern.Source);
        Assert.IsType<PassAnswer>(Without("stored").Decide(client, Priority(Ability(20, 1, Lantern))));

        // Mind Stone is a land while lands are short.
        Assert.Equal(lanternOnly, Bot().Decide(client, Priority(Ability(21, 1, Lantern))) is PassAnswer);
    }

    [Theory]
    [InlineData(20, 2, true)]
    [InlineData(10, 2, false)]
    [InlineData(4, 0, false)]
    public void Takes_another_card_off_the_One_Ring_only_while_it_has_the_life_the_Ring_will_cost(int life, int burden, bool taken)
    {
        const string Ring = "{T}: Put a burden counter on The One Ring, then draw a card for each burden counter on The One Ring.";

        ObjectView ring = Artifact(20, "The One Ring",
            "Indestructible\nAt the beginning of your upkeep, you lose 1 life for each burden counter on The One Ring.\n" + Ring);
        ring.Counters = [new CounterView { Name = "burden", Count = burden }];

        GameClient client = Seated([Land(1), ring], myLife: life);

        Assert.Equal(taken, Bot().Decide(client, Priority(Ability(20, 0, Ring))) is TakeActionAnswer);
        Assert.IsType<TakeActionAnswer>(Without("burden").Decide(client, Priority(Ability(20, 0, Ring))));
    }

    [Fact]
    public void A_spell_that_makes_a_player_discard_two_is_worth_two_cards_and_is_aimed_at_the_opponent()
    {
        GameClient client = Seated([Land(1), Land(2),
            InHand(10, "Hymn to Tourach", CardTypeView.Sorcery, 2, "Target player discards two cards at random.", color: ColorView.Black)]);

        ActionOptionMessage hymn = Cast(10);
        hymn.TargetsRequired = 1;
        hymn.TargetCandidates = [AtPlayer(Me), AtPlayer(Them)];

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(hymn)));
        Assert.Equal([Them], action.Targets.Select(target => target.Player));
    }

    [Theory]
    [InlineData(5, ActionKindView.ActivateAbility)]
    [InlineData(1, ActionKindView.CancelCasting)]
    public void Does_not_pay_the_last_of_its_life_for_mana(int life, ActionKindView expected)
    {
        ObjectView confluence = Land(1, "Mana Confluence");
        confluence.OracleText = "{T}, Pay 1 life: Add one mana of any color.";

        GameClient client = Seated([confluence, InHand(10, "Bear", CardTypeView.Creature, 1, power: 2, toughness: 2)], myLife: life);

        TakeActionAnswer paid = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, new PayCostsPromptMessage
        {
            Player = Me,
            Spell = Handle(10),
            Total = "{1}",
            Remaining = "{1}",
            RemainingGeneric = 1,
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(1), Produces = [ManaTypeView.Red], LifeCost = 1, Affordable = true },
                new() { Kind = ActionKindView.CancelCasting },
            ],
        }));

        Assert.Equal(expected, paid.Kind);
    }

    // --- Paying in life, and a card found for nothing ------------------------------------------------------------------------

    /// <summary>
    /// Which Phyrexian symbols to pay with life is announced with the spell (CR 601.2b): the seat takes the server's
    /// advice - the fewest for the mana to cover the rest - while its life stays above the floor, and none otherwise.
    /// </summary>
    [Theory]
    [InlineData(20, new[] { 0, 1 }, new[] { 0, 1 })]
    [InlineData(20, new[] { 0 }, new[] { 0 })]
    [InlineData(20, new int[0], new int[0])]
    [InlineData(9, new[] { 0, 1 }, new int[0])]
    public void Announces_life_for_the_Phyrexian_symbols_the_server_advises_while_it_has_the_life(int life, int[] advised, int[] expected)
    {
        ObjectView dismember = InHand(10, "Dismember", CardTypeView.Instant, 3, "Target creature gets -5/-5 until end of turn.", color: ColorView.Black);
        dismember.ManaCost = "{1}{B/P}{B/P}";
        Assert.Equal(2, CardReading.Phyrexian(dismember.ManaCost));

        GameClient client = Seated([Land(1, tapped: true), dismember], myLife: life);

        var prompt = new ChooseModesPromptMessage
        {
            Player = Me,
            Source = Handle(10),
            Modes = ["Pay 2 life for {B/P}", "Pay 2 life for {B/P}"],
            Eligible = [0, 1],
            Affordable = [.. advised],
            Count = 0,
            AtMost = 2,
            AdditionalCost = true,
        };

        Assert.Equal(expected, Assert.IsType<ModesChosenAnswer>(Bot().Decide(client, prompt)).Modes);
        Assert.Empty(Assert.IsType<ModesChosenAnswer>(Without("phyrexian").Decide(client, prompt)).Modes);
    }

    [Fact]
    public void A_search_that_puts_its_card_onto_the_battlefield_takes_the_biggest_whatever_it_costs()
    {
        ObjectView InLibrary(int handle, string name, int manaValue, int power, int toughness) => new()
        {
            Handle = Handle(handle),
            Zone = ZoneKindView.Library,
            ZoneOwner = Me,
            Kind = ObjectKindView.Card,
            Owner = Me,
            Controller = Me,
            Name = name,
            ManaValue = manaValue,
            CardTypes = [CardTypeView.Creature],
            Power = power,
            Toughness = toughness,
        };

        GameClient client = Seated([Land(1), InLibrary(20, "Wurm", 6, 5, 5), InLibrary(21, "Ape", 2, 3, 3)]);

        SearchLibraryPromptMessage Search(SearchDestinationView destination) => new()
        {
            Player = Me,
            Count = 1,
            Library = [Handle(20), Handle(21)],
            Eligible = [Handle(20), Handle(21)],
            Destination = destination,
        };

        // To hand, on one land, the one it can cast sooner; onto the battlefield, the one worth more there.
        Assert.Equal([Handle(21)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Search(SearchDestinationView.Hand))).Cards);
        Assert.Equal([Handle(20)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Search(SearchDestinationView.Battlefield))).Cards);
        Assert.Equal([Handle(21)], Assert.IsType<CardsChosenAnswer>(Without("fetch").Decide(client, Search(SearchDestinationView.Battlefield))).Cards);
    }

    // --- Planeswalkers -----------------------------------------------------------------------------------------------------

    private const string Food = "+2: Create a Food token.";
    private const string Elk = "+1: Target artifact or creature loses all abilities and becomes a green Elk creature with base power and toughness 3/3.";

    [Theory]
    [InlineData("+2: Create a Food token.", 2)]
    [InlineData("0: Draw three cards, then put two cards from your hand on top of your library in any order.", 0)]
    [InlineData("−5: Exchange control of target artifact or creature you control and target creature an opponent controls.", -5)]
    [InlineData("-3: Destroy target creature.", -3)]
    public void What_a_loyalty_ability_costs_is_read_off_the_front_of_it(string ability, int change) =>
        Assert.Equal(change, CardReading.Loyalty(ability));

    [Fact]
    public void An_ability_that_is_not_a_loyalty_ability_has_no_loyalty_cost()
    {
        Assert.Null(CardReading.Loyalty("{T}: Add {G}."));
        Assert.Null(CardReading.Loyalty("Crew 3"));
    }

    [Fact]
    public void Oko_makes_an_Elk_of_his_own_Food_and_not_of_an_opponents_small_creature()
    {
        ObjectView food = Artifact(3, "Food Token", token: true);
        GameClient client = Seated([Walker(2, "Oko, Thief of Crowns", 4), food, Creature(5, "Goblin", 2, 1, Them)]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(
            Ability(2, 0, Food), Ability(2, 1, Elk, At(3), At(5)))));

        Assert.Equal(1, action.AbilityIndex);
        Assert.Equal([Handle(3)], action.Targets.Select(target => target.Object));
    }

    [Fact]
    public void Oko_makes_an_Elk_of_an_opponents_dragon_and_a_Food_when_there_is_only_a_raider()
    {
        GameClient dragon = Seated([Walker(2, "Oko, Thief of Crowns", 4), Creature(5, "Dragon", 5, 5, Them, "Flying")]);
        TakeActionAnswer shrunk = Assert.IsType<TakeActionAnswer>(Bot().Decide(dragon, Priority(Ability(2, 0, Food), Ability(2, 1, Elk, At(5)))));
        Assert.Equal([Handle(5)], shrunk.Targets.Select(target => target.Object));

        // A 3/1 made a 3/3 is a gift: the old way, which read "loses all abilities" as harm and nothing else, gave it.
        GameClient raider = Seated([Walker(2, "Oko, Thief of Crowns", 4), Creature(5, "Raider", 3, 1, Them, "Haste")]);
        Assert.Equal(0, Assert.IsType<TakeActionAnswer>(Bot().Decide(raider, Priority(Ability(2, 0, Food), Ability(2, 1, Elk, At(5))))).AbilityIndex);
        Assert.Equal(1, Assert.IsType<TakeActionAnswer>(Without("loyalty").Decide(raider, Priority(Ability(2, 0, Food), Ability(2, 1, Elk, At(5))))).AbilityIndex);
    }

    [Theory]
    [InlineData(3, 1)]
    [InlineData(1, 0)]
    public void A_planeswalker_does_not_spend_its_last_loyalty_on_a_token(int loyalty, int expected)
    {
        GameClient client = Seated([Walker(2, "The Wandering Emperor", loyalty), Creature(4, "Bear", 2, 2)]);

        ActionOptionMessage counter = Ability(2, 0, "+1: Put a +1/+1 counter on up to one target creature. It gains first strike until end of turn.", At(4));
        counter.TargetsOptional = true;

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(
            counter, Ability(2, 1, "−1: Create a 2/2 white Samurai creature token with vigilance."))));

        Assert.Equal(expected, action.AbilityIndex);
    }

    [Theory]
    [InlineData(5, true)]
    [InlineData(2, false)]
    public void An_ability_that_spends_loyalty_for_a_card_is_used_while_the_planeswalker_survives_it(int loyalty, bool used)
    {
        GameClient client = Seated([Walker(2, "Narset, Parter of Veils", loyalty)]);

        AnswerPayload answer = Bot().Decide(client, Priority(Ability(2, 0,
            "−2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand. Put the rest on the bottom of your library in a random order.")));

        Assert.Equal(used, answer is TakeActionAnswer);

        // The old way never used an ability that did not add loyalty unless it was aimed at something.
        Assert.IsType<PassAnswer>(Without("loyalty").Decide(client, Priority(Ability(2, 0,
            "−2: Look at the top four cards of your library. You may reveal a noncreature, nonland card from among them and put it into your hand."))));
    }

    [Fact]
    public void An_ability_that_costs_no_loyalty_is_used_whatever_it_says()
    {
        GameClient client = Seated([Walker(2, "Comet, Stellar Pup", 5)]);

        Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Ability(2, 0, "0: Roll a six-sided die."))));
    }

    [Fact]
    public void An_unread_ability_that_spends_loyalty_is_left_alone()
    {
        GameClient client = Seated([Walker(2, "Minsc & Boo, Timeless Heroes", 5), Creature(4, "Boo", 1, 1)]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Ability(2, 1,
            "−2: Sacrifice a creature. When you do, Minsc & Boo deals X damage to any target, where X is that creature's power."))));
    }

    // --- Attacking planeswalkers ---------------------------------------------------------------------------------------------

    private static DeclareAttackersPromptMessage Attackers(int[] eligible, params int[] walkers) => new()
    {
        Player = Me,
        Eligible = [.. eligible.Select(Handle)],
        AttackablePlayers = [Them],
        AttackablePlaneswalkers = [.. walkers.Select(Handle)],
    };

    [Fact]
    public void Sends_the_attacker_nobody_can_block_at_a_planeswalker_rather_than_at_twenty_life()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Walker(5, "Jace", 2, Them)], step: TurnStepView.DeclareAttackers);

        AttackMessage attack = Assert.Single(Assert.IsType<AttackersDeclaredAnswer>(Bot().Decide(client, Attackers([1], 5))).Attacks);
        Assert.Equal(Handle(5), attack.TargetPermanent);

        AttackMessage old = Assert.Single(Assert.IsType<AttackersDeclaredAnswer>(Without("walkers").Decide(client, Attackers([1], 5))).Attacks);
        Assert.True(old.TargetPermanent.IsNone);
        Assert.Equal(Them, old.TargetPlayer);
    }

    [Fact]
    public void Sends_it_at_the_player_when_that_ends_the_game()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Walker(5, "Jace", 2, Them)], theirLife: 2, step: TurnStepView.DeclareAttackers);

        AttackMessage attack = Assert.Single(Assert.IsType<AttackersDeclaredAnswer>(Bot().Decide(client, Attackers([1], 5))).Attacks);

        Assert.True(attack.TargetPermanent.IsNone);
    }

    [Fact]
    public void A_creature_that_must_attack_a_player_is_not_sent_at_a_planeswalker()
    {
        GameClient client = Seated([Creature(1, "Bear", 2, 2), Walker(5, "Jace", 2, Them)], step: TurnStepView.DeclareAttackers);

        DeclareAttackersPromptMessage prompt = Attackers([1], 5);
        prompt.Requirements.Add(new AttackRequirementMessage { Attacker = Handle(1), Players = [Them] });

        AttackMessage attack = Assert.Single(Assert.IsType<AttackersDeclaredAnswer>(Bot().Decide(client, prompt)).Attacks);

        Assert.True(attack.TargetPermanent.IsNone);
        Assert.True(prompt.Requirements[0].MetBy([attack]));
    }

    [Fact]
    public void Damage_to_a_planeswalker_is_worth_its_share_of_the_planeswalker_and_costs_the_player_no_life()
    {
        TacticalModel model = TacticalModel.Prior();
        Unit attacker = new() { Handle = Handle(1), Power = 3, Toughness = 3, Worth = 1 };

        Combat.Result atWalker = new Combat(model, [attacker], [], 20, 20, walkers: [(6, 1.2)], aims: [0]).BestBlocks(1);
        Assert.Equal(0, atWalker.Damage);
        Assert.Equal(0.6, atWalker.Value, 3);

        // Three damage at three life is the game; at a planeswalker it is not.
        Assert.False(new Combat(model, [attacker], [], 20, 3, walkers: [(6, 1.2)], aims: [0]).BestBlocks(1).Lethal);
        Assert.True(new Combat(model, [attacker], [], 20, 3, walkers: [(6, 1.2)], aims: [-1]).BestBlocks(1).Lethal);
    }

    [Fact]
    public void Throws_a_creature_in_front_of_what_would_kill_its_planeswalker_though_not_in_front_of_five_damage()
    {
        // A 1/1 against a 5/5: at twenty life the five damage is taken. Aimed at a planeswalker on five loyalty, it
        // is the planeswalker that is lost, and that is worth the 1/1.
        List<ObjectView> board = [Creature(1, "Goblin", 1, 1), Walker(6, "Teferi", 5), Creature(4, "Giant", 5, 5, Them, tapped: true)];

        var prompt = new DeclareBlockersPromptMessage
        {
            Player = Me,
            Attackers = [Handle(4)],
            Eligible = [Handle(1)],
            Allowed = [new BlockMessage { Blocker = Handle(1), Attacker = Handle(4) }],
            MinimumBlockers = [1],
        };

        GameClient atPlayer = Seated(board, active: Them, step: TurnStepView.DeclareBlockers, combat: new CombatView
        {
            Attackers = [new AttackerView { Attacker = Handle(4), TargetKind = AttackTargetKindView.Player, TargetPlayer = Me }],
        });

        Assert.Empty(Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(atPlayer, prompt)).Blocks);

        GameClient atWalker = Seated(board, active: Them, step: TurnStepView.DeclareBlockers, combat: new CombatView
        {
            Attackers = [new AttackerView { Attacker = Handle(4), TargetKind = AttackTargetKindView.Planeswalker, TargetPermanent = Handle(6) }],
        });

        Assert.Single(Assert.IsType<BlockersDeclaredAnswer>(Bot().Decide(atWalker, prompt)).Blocks);
    }

    // --- Vehicles ------------------------------------------------------------------------------------------------------------

    private const string Copter = "Flying\nWhenever this Vehicle attacks or blocks, you may draw a card. If you do, discard a card.\nCrew 1";

    private static ObjectView Vehicle(int handle, string name, int power, int toughness, string text, bool sick = false)
    {
        ObjectView vehicle = Artifact(handle, name, text);
        vehicle.Subtypes = ["Vehicle"];
        vehicle.Power = power;
        vehicle.Toughness = toughness;
        vehicle.SummoningSick = sick;
        return vehicle;
    }

    private static ObjectView Sick(ObjectView creature)
    {
        creature.SummoningSick = true;
        return creature;
    }

    [Fact]
    public void The_power_a_crew_ability_asks_for_is_read_off_it()
    {
        Assert.Equal(3, CardReading.Crew("Crew 3"));
        Assert.Equal(1, CardReading.Crew("Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"));
        Assert.Equal(0, CardReading.Crew("{T}: Add {G}."));
    }

    [Fact]
    public void Crews_a_Vehicle_with_the_creature_that_cannot_attack_this_turn()
    {
        GameClient client = Seated([Vehicle(2, "Smuggler's Copter", 3, 3, Copter), Sick(Creature(3, "Goblin", 1, 1))]);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Ability(2, 0, "Crew 1"))));
        Assert.Equal(Handle(2), action.Source);

        Assert.IsType<PassAnswer>(Without("crew").Decide(client, Priority(Ability(2, 0, "Crew 1"))));
    }

    [Fact]
    public void Does_not_crew_a_Vehicle_that_arrived_this_turn_or_one_that_would_be_eaten()
    {
        GameClient arrived = Seated([Vehicle(2, "Smuggler's Copter", 3, 3, Copter, sick: true), Sick(Creature(3, "Goblin", 1, 1))]);
        Assert.IsType<PassAnswer>(Bot().Decide(arrived, Priority(Ability(2, 0, "Crew 1"))));

        GameClient eaten = Seated(
            [Vehicle(2, "Smuggler's Copter", 3, 3, Copter), Sick(Creature(3, "Goblin", 1, 1)), Creature(5, "Angel", 4, 4, Them, "Flying")]);
        Assert.IsType<PassAnswer>(Bot().Decide(eaten, Priority(Ability(2, 0, "Crew 1"))));
    }

    // --- Station ------------------------------------------------------------------------------------------------------------

    private const string StationText = "Station (Tap another creature you control: Put charge counters equal to its power on this Spacecraft. Station only as a sorcery. It's an artifact creature at 4+.)";

    private static ObjectView Spacecraft(int handle, int charge)
    {
        ObjectView ship = Artifact(handle, "Wurmwall Sweeper", "When this Spacecraft enters, surveil 2.\n" + StationText + "\n4+ | Flying");
        ship.Subtypes = ["Spacecraft"];
        ship.Counters = charge > 0 ? [new CounterView { Name = "charge", Count = charge }] : [];
        return ship;
    }

    [Fact]
    public void The_last_station_symbol_is_read_off_a_station_cards_text()
    {
        Assert.Equal(12, CardReading.LastStationThreshold("This land enters tapped.\n{T}: Add {U}.\nStation (...)\n12+ | {U}, {T}: Add {U} for each artifact you control."));
        Assert.Equal(20, CardReading.LastStationThreshold("Station (...)\n10+ | Whenever you attack, ...\n20+ | Flying"));
        Assert.Equal(0, CardReading.LastStationThreshold("Crew 1"));
    }

    [Fact]
    public void Stations_after_combat_with_the_creature_of_most_power_and_taps_that_one_for_it()
    {
        GameClient client = Seated([Spacecraft(2, 0), Creature(3, "Goblin", 1, 1), Creature(4, "Giant", 5, 5)], step: TurnStepView.PostcombatMain);

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(Ability(2, 1, StationText))));
        Assert.Equal(Handle(2), action.Source);

        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Source = Handle(2),
            Purpose = CardChoicePurposeView.TapToPay,
            Cards = [Handle(3), Handle(4)],
            Eligible = [Handle(3), Handle(4)],
            Count = 1,
        };
        Assert.Equal([Handle(4)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards);

        Assert.IsType<PassAnswer>(Without("station").Decide(client, Priority(Ability(2, 1, StationText))));
    }

    [Fact]
    public void Does_not_station_before_combat_past_its_last_symbol_with_no_power_or_with_a_needed_blocker()
    {
        Assert.IsType<PassAnswer>(Bot().Decide(Seated([Spacecraft(2, 0), Creature(4, "Giant", 5, 5)]), Priority(Ability(2, 1, StationText))));

        GameClient full = Seated([Spacecraft(2, 4), Creature(4, "Giant", 5, 5)], step: TurnStepView.PostcombatMain);
        Assert.IsType<PassAnswer>(Bot().Decide(full, Priority(Ability(2, 1, StationText))));

        GameClient wall = Seated([Spacecraft(2, 0), Creature(4, "Wall", 0, 4)], step: TurnStepView.PostcombatMain);
        Assert.IsType<PassAnswer>(Bot().Decide(wall, Priority(Ability(2, 1, StationText))));

        // The Giant is what keeps the Bear out on the way back.
        GameClient guard = Seated([Spacecraft(2, 0), Creature(4, "Giant", 5, 5), Creature(5, "Bear", 2, 2, Them)], step: TurnStepView.PostcombatMain);
        Assert.IsType<PassAnswer>(Bot().Decide(guard, Priority(Ability(2, 1, StationText))));
    }

    [Fact]
    public void Collects_evidence_with_the_fewest_cards_that_reach_the_total()
    {
        // Collect evidence 6 (CR 701.59a) over mana values 3, 0, 4 and 2: the 4 and the 3 do it, two cards of four.
        GameClient client = Seated([Creature(9, "Bear", 2, 2)]);
        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Purpose = CardChoicePurposeView.ExileToPay,
            Cards = [Handle(3), Handle(4), Handle(5), Handle(6)],
            Eligible = [Handle(3), Handle(4), Handle(5), Handle(6)],
            Count = 4,
            UpTo = true,
            MinimumManaValue = 6,
            ManaValues = [3, 0, 4, 2],
        };

        CardsChosenAnswer chosen = Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt));
        Assert.Equal([Handle(3), Handle(5)], chosen.Cards.Order());
    }

    [Fact]
    public void Crafts_with_the_fewest_materials_a_graveyard_card_before_a_permanent()
    {
        // Craft (CR 702.167a): the materials are exiled for good, so a card already in the graveyard goes before a creature
        // still on the battlefield, and "one or more" takes one.
        ObjectView dead = Creature(4, "Elf", 1, 1);
        dead.Zone = ZoneKindView.Graveyard;
        dead.ZoneOwner = Me;
        GameClient client = Seated([Creature(3, "Bear", 2, 2), dead, Creature(5, "Giant", 5, 5)]);
        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Purpose = CardChoicePurposeView.Craft,
            Cards = [Handle(3), Handle(4), Handle(5)],
            Eligible = [Handle(3), Handle(4), Handle(5)],
            Count = 3,
            UpTo = true,
            Minimum = 1,
            Kind = "creature",
        };

        Assert.Equal([Handle(4)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards);

        prompt.UpTo = false;
        prompt.Minimum = 0;
        prompt.Count = 2;
        Assert.Equal([Handle(4), Handle(3)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards);
    }

    [Fact]
    public void Crafts_into_a_better_face_and_not_into_a_worse_one()
    {
        // Craft (CR 702.167a) turns the card into its other face: a 6/6 is worth the trade for a bare artifact; the same
        // artifact turning into a lesser artifact is not.
        ObjectView inverted = Artifact(2, "Inverted Iceberg", "Craft with artifact {4}{U}{U}");
        inverted.OtherFace = Creature(0, "Iceberg Titan", 6, 6);
        GameClient client = Seated([inverted, Artifact(3, "Ornithopter")]);
        ActionOptionMessage craft = Ability(2, 0, "Craft with artifact {4}{U}{U}");

        TakeActionAnswer taken = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(craft)));
        Assert.Equal(Handle(2), taken.Source);

        ObjectView worse = Artifact(2, "Inverted Iceberg", "Craft with artifact {4}{U}{U}");
        worse.OtherFace = Artifact(0, "Iceberg Shard");
        GameClient other = Seated([worse, Artifact(3, "Ornithopter")]);
        Assert.IsType<PassAnswer>(Bot().Decide(other, Priority(craft)));
    }

    [Fact]
    public void Does_not_tap_a_bigger_attacker_to_crew_a_smaller_Vehicle()
    {
        GameClient client = Seated([Vehicle(2, "Cart", 2, 2, "Crew 2"), Creature(3, "Giant", 5, 5)]);

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(Ability(2, 0, "Crew 2"))));
    }

    [Fact]
    public void Pays_a_crew_cost_with_the_creatures_that_give_up_least_and_no_more_than_it_takes()
    {
        GameClient client = Seated(
        [
            Vehicle(2, "Reckoner Bankbuster", 4, 4, "Crew 3"),
            Creature(3, "Knight", 3, 3),
            Sick(Creature(4, "Goblin", 1, 1)),
            Sick(Creature(5, "Bear", 2, 2)),
        ]);

        var prompt = new ChooseCardsPromptMessage
        {
            Player = Me,
            Owner = Me,
            Source = Handle(2),
            Purpose = CardChoicePurposeView.TapToPay,
            Cards = [Handle(3), Handle(4), Handle(5)],
            Eligible = [Handle(3), Handle(4), Handle(5)],
            Count = 3,
            UpTo = true,
            MinimumPower = 3,
            Powers = [3, 1, 2],
        };

        // The two that arrived this turn add up to three, and the Knight keeps its attack. The prompt's own answer is
        // all three of them.
        CardsChosenAnswer chosen = Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt));
        Assert.Equal([Handle(4), Handle(5)], chosen.Cards.Order());

        Assert.Equal(3, Assert.IsType<CardsChosenAnswer>(Without("crew").Decide(client, prompt)).Cards.Count);
    }

    [Fact]
    public void Does_not_crew_a_Vehicle_cast_this_turn_though_the_server_sends_no_summoning_sickness_for_it()
    {
        // The server sends summoning sickness only for creatures (a Vehicle is not one until crewed), so the seat
        // tells by what it saw before its turn began.
        static List<ObjectView> Board(bool withCopter)
        {
            List<ObjectView> board = [Sick(Creature(3, "Goblin", 1, 1)), Land(1)];
            if (withCopter)
            {
                board.Add(Vehicle(2, "Smuggler's Copter", 3, 3, Copter));
            }

            return board;
        }

        var cast = new Table(Board(withCopter: false), active: Them, turn: 4);
        TacticalPlay castBot = Bot();
        castBot.Decide(cast.Client, Priority());
        cast.Show(Board(withCopter: true), turn: 5);
        Assert.IsType<PassAnswer>(castBot.Decide(cast.Client, Priority(Ability(2, 0, "Crew 1"))));

        var kept = new Table(Board(withCopter: true), active: Them, turn: 4);
        TacticalPlay keptBot = Bot();
        keptBot.Decide(kept.Client, Priority());
        kept.Show(Board(withCopter: true), turn: 5);
        Assert.Equal(Handle(2), Assert.IsType<TakeActionAnswer>(keptBot.Decide(kept.Client, Priority(Ability(2, 0, "Crew 1")))).Source);
    }

    [Theory]
    [InlineData(20, true)]
    [InlineData(5, false)]
    public void Does_not_crew_with_the_creature_that_has_to_block_the_attack_back(int life, bool crewed)
    {
        // The Goblin blocks one of three 2/2s on the way back. Tapped to crew, it blocks none: six at five life.
        GameClient client = Seated(
        [
            Vehicle(2, "Smuggler's Copter", 3, 3, Copter),
            Sick(Creature(3, "Goblin", 1, 1)),
            Creature(5, "Bear", 2, 2, Them), Creature(6, "Bear", 2, 2, Them), Creature(7, "Bear", 2, 2, Them),
        ], myLife: life);

        Assert.Equal(crewed, Bot().Decide(client, Priority(Ability(2, 0, "Crew 1"))) is TakeActionAnswer);
    }

    [Fact]
    public void A_crewed_Vehicle_is_not_counted_on_to_block_the_attack_back()
    {
        // At three life, against a 3/3: staying home, the Copter would be no blocker on their turn, so it attacks.
        ObjectView copter = Vehicle(2, "Smuggler's Copter", 3, 3, Copter);
        copter.CardTypes = [CardTypeView.Artifact, CardTypeView.Creature];

        GameClient client = Seated([copter, Creature(5, "Bear", 3, 3, Them)], myLife: 3, step: TurnStepView.DeclareAttackers);
        var prompt = new DeclareAttackersPromptMessage { Player = Me, Eligible = [Handle(2)], AttackablePlayers = [Them] };

        Assert.Single(Assert.IsType<AttackersDeclaredAnswer>(Bot().Decide(client, prompt)).Attacks);
        Assert.Empty(Assert.IsType<AttackersDeclaredAnswer>(Without("crew").Decide(client, prompt)).Attacks);
    }

    [Fact]
    public void Casts_a_Phyrexian_spell_the_server_calls_unaffordable_when_life_pays_the_rest()
    {
        ObjectView dismember = InHand(10, "Dismember", CardTypeView.Instant, 3, "Target creature gets -5/-5 until end of turn.", color: ColorView.Black);
        dismember.ManaCost = "{1}{B/P}{B/P}";

        GameClient client = Seated([Land(1, "Swamp"), dismember, Creature(5, "Giant", 4, 4, Them)]);

        // The server's word is about mana alone.
        ActionOptionMessage cast = Cast(10);
        cast.Affordable = false;
        cast.TargetsRequired = 1;
        cast.TargetCandidates = [At(5)];

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(cast)));
        Assert.Equal(Handle(10), action.Source);
        Assert.Equal([Handle(5)], action.Targets.Select(target => target.Object));

        Assert.IsType<PassAnswer>(Without("phyrexian").Decide(client, Priority(cast)));

        // Nor with too little life to pay for both symbols: the mana is short by two.
        Assert.IsType<PassAnswer>(Bot().Decide(Seated([Land(1, "Swamp"), dismember, Creature(5, "Giant", 4, 4, Them)], myLife: 11), Priority(cast)));
    }

    private const string Blaze = "Blaze deals X damage to any target.";

    private static List<ObjectView> Confluence(int mountains)
    {
        ObjectView confluence = Land(1, "Mana Confluence");
        confluence.OracleText = "{T}, Pay 1 life: Add one mana of any color.";

        List<ObjectView> board = [confluence, .. Enumerable.Range(2, mountains).Select(i => Land(i))];
        board.Add(InHand(10, "Blaze", CardTypeView.Sorcery, 1, Blaze));
        board.Add(Creature(20, "Goblin", 1, 1, Them));
        return board;
    }

    // The announcement (CR 601.2b) as the server sends it: before any payment, MostAffordable counting every source,
    // X where it was left, and no source to tap.
    private static PayCostsPromptMessage Announcement(int x, int most) => new()
    {
        Player = Me,
        Spell = Handle(10),
        Total = "{X}{R}",
        HasX = true,
        AnnouncesXOrKicks = true,
        X = x,
        MostAffordable = most,
        Options =
        [
            new() { Kind = ActionKindView.IncreaseX },
            new() { Kind = ActionKindView.DecreaseX },
            new() { Kind = ActionKindView.ConfirmCasting },
            new() { Kind = ActionKindView.CancelCasting },
        ],
    };

    [Theory]
    // Two Mountains and a Confluence: X is 2 counting the Confluence, 1 without it.
    [InlineData(2, 5, 2, ActionKindView.ConfirmCasting)]
    [InlineData(2, 1, 2, ActionKindView.DecreaseX)]
    [InlineData(2, 1, 1, ActionKindView.ConfirmCasting)]
    // One Mountain and a Confluence: at one life there is nothing to announce, and the card is kept.
    [InlineData(1, 5, 1, ActionKindView.ConfirmCasting)]
    [InlineData(1, 1, 1, ActionKindView.CancelCasting)]
    public void Announces_X_without_the_mana_that_would_cost_the_last_of_its_life(int mountains, int life, int x, ActionKindView expected)
    {
        GameClient client = Seated(Confluence(mountains), myLife: life);

        Assert.Equal(expected, Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Announcement(x, most: mountains))).Kind);
        Assert.Equal(ActionKindView.ConfirmCasting, Assert.IsType<TakeActionAnswer>(Without("lastlife").Decide(client, Announcement(mountains, mountains))).Kind);
    }

    [Theory]
    [InlineData(5, true)]
    [InlineData(1, false)]
    public void Does_not_cast_an_X_spell_whose_X_only_the_last_of_its_life_would_pay_for(int life, bool cast)
    {
        GameClient client = Seated(Confluence(1), myLife: life);

        ActionOptionMessage blaze = new()
        {
            Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true, HasX = true, MaxX = 1,
            TargetsRequired = 1, TargetCandidates = [At(20), AtPlayer(Them)],
        };

        Assert.Equal(cast, Bot().Decide(client, Priority(blaze)) is TakeActionAnswer);
    }

    [Fact]
    public void Two_painful_sources_at_two_life_are_one_too_many()
    {
        ObjectView First() { ObjectView land = Land(1, "Mana Confluence"); land.OracleText = "{T}, Pay 1 life: Add one mana of any color."; return land; }
        ObjectView second = Land(2, "City of Brass");
        second.OracleText = "Whenever this land becomes tapped, it deals 1 damage to you.\n{T}: Add one mana of any color.";

        GameClient client = Seated([First(), second, InHand(10, "Blaze", CardTypeView.Sorcery, 1, Blaze)], myLife: 2);

        // X counted 1 with both; one of them is the last life, so X is announced at 0, and the card kept.
        Assert.Equal(ActionKindView.CancelCasting, Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Announcement(1, 1))).Kind);
    }

    [Theory]
    [InlineData(5, ActionKindView.ActivateAbility)]
    [InlineData(1, ActionKindView.DeclineCost)]
    public void Does_not_pay_the_last_of_its_life_to_keep_a_spell_from_being_countered(int life, ActionKindView expected)
    {
        ObjectView confluence = Land(1, "Mana Confluence");
        confluence.OracleText = "{T}, Pay 1 life: Add one mana of any color.";

        GameClient client = Seated([confluence], myLife: life);

        TakeActionAnswer paid = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, new UnlessPaysPromptMessage
        {
            Player = Me,
            Source = Handle(30),
            Total = "{1}",
            Remaining = "{1}",
            RemainingGeneric = 1,
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(1), Produces = [ManaTypeView.Red], LifeCost = 1, Affordable = true },
                new() { Kind = ActionKindView.DeclineCost },
            ],
        }));

        Assert.Equal(expected, paid.Kind);
    }

    [Fact]
    public void Does_not_pay_four_in_the_upkeep_to_untap_a_Mana_Vault()
    {
        ObjectView vault = Artifact(3, "Mana Vault", "This artifact doesn't untap during your untap step.\n{T}: Add {C}{C}{C}.", makesMana: true);
        vault.Tapped = true;

        GameClient client = Seated([Land(1), Land(2), Land(4), Land(5), vault], step: TurnStepView.Upkeep);

        UnlessPaysPromptMessage Untap() => new()
        {
            Player = Me,
            Source = Handle(3),
            Verb = "UntapSource",
            Total = "{4}",
            Remaining = "{4}",
            RemainingGeneric = 4,
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(1), Produces = [ManaTypeView.Red], Affordable = true },
                new() { Kind = ActionKindView.DeclineCost },
            ],
        };

        Assert.Equal(ActionKindView.DeclineCost, Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Untap())).Kind);
        Assert.Equal(ActionKindView.ActivateAbility, Assert.IsType<TakeActionAnswer>(Without("rocks").Decide(client, Untap())).Kind);
    }

    [Theory]
    // A card of mana value 3 or less in its graveyard for the payment to return, and life to spare.
    [InlineData(2, 20, ActionKindView.PayCost)]
    // Nothing the reflexive ability could return: the payment buys nothing.
    [InlineData(4, 20, ActionKindView.DeclineCost)]
    // Two life is too much of what is left.
    [InlineData(2, 7, ActionKindView.DeclineCost)]
    public void Pays_mana_and_life_for_a_card_back_only_with_one_to_return_and_life_to_spare(int manaValue, int life, ActionKindView expected)
    {
        // Zoraline, Cosmos Caller's "you may pay {W}{B} and 2 life. When you do, return target nonland permanent card with
        // mana value 3 or less from your graveyard to the battlefield": the card is chosen once paid, so the seat looks for one.
        ObjectView dead = Creature(4, "Bear", 2, 2);
        dead.Zone = ZoneKindView.Graveyard;
        dead.ZoneOwner = Me;
        dead.ManaValue = manaValue;
        GameClient client = Seated([dead], myLife: life);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, new UnlessPaysPromptMessage
        {
            Player = Me,
            Source = Handle(30),
            Subject = Handle(30),
            Verb = "ReturnPermanentCard",
            Amount = 3,
            Total = "{W}{B}",
            Remaining = "{0}",
            Life = 2,
            Options = [new() { Kind = ActionKindView.PayCost }, new() { Kind = ActionKindView.DeclineCost }],
        }));

        Assert.Equal(expected, answer.Kind);
    }

    [Theory]
    // Ward—Pay 3 life (CR 702.21a): the spell kept, while the life left is not short.
    [InlineData("Countered", 10, ActionKindView.PayCost)]
    [InlineData("Countered", 6, ActionKindView.DeclineCost)]
    // A shockland's 2 life with nothing in hand to cast: entering tapped costs nothing.
    [InlineData("EnterUntapped", 20, ActionKindView.DeclineCost)]
    // Lim-Dûl's Vault's look again: never.
    [InlineData("LookAgain", 20, ActionKindView.DeclineCost)]
    public void Weighs_a_payment_of_life_by_what_paying_does(string verb, int life, ActionKindView expected)
    {
        GameClient client = Seated([], myLife: life);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, new UnlessPaysPromptMessage
        {
            Player = Me,
            Source = Handle(30),
            Subject = Handle(31),
            Verb = verb,
            Total = "{0}",
            Remaining = "{0}",
            Life = 3,
            Options = [new() { Kind = ActionKindView.PayCost }, new() { Kind = ActionKindView.DeclineCost }],
        }));

        Assert.Equal(expected, answer.Kind);
    }

    [Fact]
    public void Oko_does_not_make_an_Elk_of_what_makes_its_own_mana()
    {
        ObjectView elves = Creature(3, "Llanowar Elves", 1, 1, text: "{T}: Add {G}.");
        elves.ProducesMana = true;

        GameClient dork = Seated([Walker(2, "Oko, Thief of Crowns", 4), elves]);
        Assert.Equal(0, Assert.IsType<TakeActionAnswer>(Bot().Decide(dork, Priority(Ability(2, 0, Food), Ability(2, 1, Elk, At(3))))).AbilityIndex);

        GameClient mox = Seated([Walker(2, "Oko, Thief of Crowns", 4), Artifact(3, "Mox Emerald", Mox, makesMana: true)]);
        Assert.Equal(0, Assert.IsType<TakeActionAnswer>(Bot().Decide(mox, Priority(Ability(2, 0, Food), Ability(2, 1, Elk, At(3))))).AbilityIndex);
    }

    [Fact]
    public void A_planeswalker_that_returns_up_to_one_target_to_its_owners_hand_bounces_it()
    {
        const string Teferi = "−3: Return up to one target artifact, creature, or enchantment to its owner's hand. Draw a card.";
        Assert.Equal(EffectKind.Bounce, CardReading.Effect(Teferi).Kind);
        Assert.Equal(1, CardReading.Draws("+2: You gain 2 life and draw a card."));

        GameClient client = Seated([Walker(2, "Teferi, Time Raveler", 5), Creature(4, "Bear", 2, 2), Creature(5, "Dragon", 5, 5, Them, "Flying")]);

        ActionOptionMessage bounce = Ability(2, 0, Teferi, At(4), At(5));
        bounce.TargetsOptional = true;

        TakeActionAnswer action = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Priority(bounce)));
        Assert.Equal([Handle(5)], action.Targets.Select(target => target.Object));
    }

    [Fact]
    public void A_spell_that_makes_a_player_discard_is_worth_nothing_against_an_empty_hand()
    {
        GameClient client = Seated([Land(1), Land(2),
            InHand(10, "Hymn to Tourach", CardTypeView.Sorcery, 2, "Target player discards two cards at random.", color: ColorView.Black)]);
        client.View!.Players.Find(player => player.Seat == Them)!.HandCount = 0;

        ActionOptionMessage hymn = Cast(10);
        hymn.TargetsRequired = 1;
        hymn.TargetCandidates = [AtPlayer(Me), AtPlayer(Them)];

        Assert.IsType<PassAnswer>(Bot().Decide(client, Priority(hymn)));

        // The old way read it as harm whatever the hand held.
        Assert.IsType<TakeActionAnswer>(Without("discard").Decide(client, Priority(hymn)));
    }
}
