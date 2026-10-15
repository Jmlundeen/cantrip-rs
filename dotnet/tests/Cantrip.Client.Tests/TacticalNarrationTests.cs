using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// The tactical seat's account of its own decisions (<see cref="TacticalPlay.Narrator"/>): what it was asked, what it
/// weighed and what it answered, and never a different answer for being asked to tell it.
/// </summary>
public sealed class TacticalNarrationTests
{
    private static ActionOptionMessage Cast(int source) => new() { Kind = ActionKindView.CastSpell, Source = Handle(source), Affordable = true };

    [Fact]
    public void Tells_what_it_weighed_and_what_it_cast()
    {
        GameClient client = Seated(
        [
            Land(1), Land(2),
            InHand(10, "Hill Giant", CardTypeView.Creature, 2, power: 3, toughness: 3),
            InHand(11, "Goblin", CardTypeView.Creature, 1, power: 1, toughness: 1),
        ]);

        var told = new StringWriter();
        var prompt = new PriorityPromptMessage { Player = Me, Options = [Cast(10), Cast(11)] };

        AnswerPayload narrated = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told }.Decide(client, prompt);
        AnswerPayload silent = new TacticalPlay(TacticalModel.Prior(), seed: 1).Decide(client, prompt);

        Assert.Equal(Assert.IsType<TakeActionAnswer>(silent).Source, Assert.IsType<TakeActionAnswer>(narrated).Source);

        string text = told.ToString();
        Assert.Contains("T5 PrecombatMain mine | life 20-20", text, StringComparison.Ordinal);
        Assert.Contains("priority: cast Hill Giant", text, StringComparison.Ordinal);
        Assert.Contains("weighed (mana 2", text, StringComparison.Ordinal);
        Assert.Contains("cast Goblin", text, StringComparison.Ordinal);
    }

    [Fact]
    public void Answers_a_targeted_spell_an_attack_and_a_block_the_same_whether_telling_or_not()
    {
        static (AnswerPayload Narrated, AnswerPayload Silent, string Told) Both(GameClient client, PromptPayload prompt)
        {
            var told = new StringWriter();
            AnswerPayload narrated = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told }.Decide(client, prompt);
            AnswerPayload silent = new TacticalPlay(TacticalModel.Prior(), seed: 1).Decide(client, prompt);
            return (narrated, silent, told.ToString());
        }

        // A bolt, aimed.
        GameClient bolt = Seated([Land(1), InHand(10, "Lightning Bolt", CardTypeView.Instant, 1, "Lightning Bolt deals 3 damage to any target."),
            Creature(20, "Bear", 2, 2, Them)]);
        var cast = new PriorityPromptMessage
        {
            Player = Me,
            Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true, TargetsRequired = 1, TargetCandidates = [At(20), AtPlayer(Them)] }],
        };
        (AnswerPayload narrated, AnswerPayload silent, string told) = Both(bolt, cast);
        Assert.Equal(Assert.IsType<TakeActionAnswer>(silent).Targets.Select(target => (target.Player, target.Object)),
            Assert.IsType<TakeActionAnswer>(narrated).Targets.Select(target => (target.Player, target.Object)));
        Assert.Contains("cast Lightning Bolt ->", told, StringComparison.Ordinal);

        // An attack.
        GameClient attack = Seated([Creature(1, "Giant", 4, 4), Creature(2, "Goblin", 1, 1), Creature(20, "Bear", 2, 2, Them)], step: TurnStepView.DeclareAttackers);
        var attackers = new DeclareAttackersPromptMessage { Player = Me, Eligible = [Handle(1), Handle(2)], AttackablePlayers = [Them] };
        (narrated, silent, told) = Both(attack, attackers);
        Assert.Equal(Assert.IsType<AttackersDeclaredAnswer>(silent).Attacks.Select(one => one.Attacker),
            Assert.IsType<AttackersDeclaredAnswer>(narrated).Attacks.Select(one => one.Attacker));
        Assert.Contains("attack scored", told, StringComparison.Ordinal);

        // A block.
        GameClient block = Seated([Creature(1, "Wall", 0, 5), Creature(20, "Giant", 4, 4, Them, tapped: true)], active: Them, step: TurnStepView.DeclareBlockers,
            combat: new CombatView { Attackers = [new AttackerView { Attacker = Handle(20), TargetKind = AttackTargetKindView.Player, TargetPlayer = Me }] });
        var blockers = new DeclareBlockersPromptMessage
        {
            Player = Me,
            Attackers = [Handle(20)],
            Eligible = [Handle(1)],
            Allowed = [new BlockMessage { Blocker = Handle(1), Attacker = Handle(20) }],
            MinimumBlockers = [1],
        };
        (narrated, silent, told) = Both(block, blockers);
        Assert.Equal(Assert.IsType<BlockersDeclaredAnswer>(silent).Blocks.Select(one => (one.Blocker, one.Attacker)),
            Assert.IsType<BlockersDeclaredAnswer>(narrated).Blocks.Select(one => (one.Blocker, one.Attacker)));
        Assert.Contains("block:", told, StringComparison.Ordinal);
    }

    [Fact]
    public void Tells_the_board_and_its_hand_once_a_turn()
    {
        GameClient client = Seated(
        [
            Land(1), Creature(2, "Wall", 0, 5), Creature(20, "Giant", 4, 4, Them, tapped: true),
            InHand(10, "Hill Giant", CardTypeView.Creature, 4, power: 3, toughness: 3),
        ], step: TurnStepView.DeclareAttackers);

        var told = new StringWriter();
        var bot = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told };
        var prompt = new DeclareAttackersPromptMessage { Player = Me, Eligible = [Handle(2)], AttackablePlayers = [Them] };

        bot.Decide(client, prompt);
        bot.Decide(client, prompt);

        string text = told.ToString();
        Assert.Contains("--- Turn 5, mine ---", text, StringComparison.Ordinal);
        Assert.Contains("board me  : 20 life", text, StringComparison.Ordinal);
        Assert.Contains("| Wall 0/5", text, StringComparison.Ordinal);
        Assert.Contains("| Giant 4/4 (T)", text, StringComparison.Ordinal);
        Assert.Contains("my hand: Hill Giant", text, StringComparison.Ordinal);
        Assert.Equal(1, text.Split("board me").Length - 1);
    }

    [Fact]
    public void Tells_a_spell_it_did_not_weigh_and_the_same_pass_once_a_turn()
    {
        GameClient client = Seated([Land(1), InHand(10, "Riddle", CardTypeView.Instant, 1, "Target player does something nobody can read.")]);
        var told = new StringWriter();
        var bot = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told };
        var prompt = new PriorityPromptMessage
        {
            Player = Me,
            Options = [new() { Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true, TargetsRequired = 1, TargetCandidates = [AtPlayer(Them)] }],
        };

        Assert.IsType<PassAnswer>(bot.Decide(client, prompt));
        Assert.IsType<PassAnswer>(bot.Decide(client, prompt));

        string text = told.ToString();
        Assert.Contains("castable, not weighed (unread, or nothing for it to do): Riddle", text, StringComparison.Ordinal);
        Assert.Equal(1, text.Split("castable, not weighed").Length - 1);
    }

    [Fact]
    public void A_record_that_cannot_be_written_is_given_up_and_the_game_goes_on()
    {
        GameClient client = Seated([Land(1), InHand(10, "Goblin", CardTypeView.Creature, 1, power: 1, toughness: 1)]);
        var closed = new StringWriter();
        closed.Dispose();
        var bot = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = closed };

        Assert.IsType<TakeActionAnswer>(bot.Decide(client, new PriorityPromptMessage { Player = Me, Options = [Cast(10)] }));
        Assert.Null(bot.Narrator);
    }

    [Fact]
    public void Says_nothing_of_a_pass_with_nothing_to_weigh_or_of_tapping_a_land()
    {
        GameClient client = Seated([Land(1), InHand(10, "Goblin", CardTypeView.Creature, 1, power: 1, toughness: 1)]);
        var told = new StringWriter();
        var bot = new TacticalPlay(TacticalModel.Prior(), seed: 1) { Narrator = told };

        Assert.IsType<PassAnswer>(bot.Decide(client, new PriorityPromptMessage { Player = Me }));
        bot.Decide(client, new PayCostsPromptMessage
        {
            Player = Me,
            Spell = Handle(10),
            Total = "{1}",
            Remaining = "{1}",
            RemainingGeneric = 1,
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(1), Produces = [ManaTypeView.Red], Affordable = true },
                new() { Kind = ActionKindView.CancelCasting },
            ],
        });

        Assert.Equal(string.Empty, told.ToString());
    }
}
