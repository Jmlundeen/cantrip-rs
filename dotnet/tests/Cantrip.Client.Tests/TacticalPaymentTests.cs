using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Which source pays (<c>policy.payplan</c>): the one that leaves open the mana for the rest of the turn's plan, and of those a
/// plain land before a land that does something else, and never a source of two for one generic.
/// </summary>
public sealed class TacticalPaymentTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ActionOptionMessage Tap(int source, params ManaTypeView[] makes) =>
        new() { Kind = ActionKindView.ActivateAbility, Source = Handle(source), Produces = [.. makes], Affordable = true };

    private static PayCostsPromptMessage Owing(int spell, string total, List<ColorView> colored, int generic, params ActionOptionMessage[] sources) => new()
    {
        Player = Me,
        Spell = Handle(spell),
        Total = total,
        Remaining = total,
        RemainingColored = colored,
        RemainingGeneric = generic,
        Options = [.. sources, new() { Kind = ActionKindView.CancelCasting }],
    };

    [Fact]
    public void A_plain_land_pays_before_a_land_that_becomes_a_creature()
    {
        ObjectView den = Land(1, "Den");
        den.Subtypes = [];
        den.Supertypes = [];
        den.OracleText = "{T}: Add {R}.\n{3}{R}: Until end of turn, this land becomes a 3/2 red Goblin creature with haste. It's still a land.";
        GameClient client = Seated([den, Land(2, "Mountain"), InHand(90, "Shock", CardTypeView.Instant, 1)]);

        AnswerPayload paid = Bot().Decide(client, Owing(90, "{R}", [ColorView.Red], 0, Tap(1, ManaTypeView.Red), Tap(2, ManaTypeView.Red)));
        Assert.Equal(Handle(2), Assert.IsType<TakeActionAnswer>(paid).Source);
    }

    [Fact]
    public void One_generic_is_paid_with_a_source_of_one_and_not_with_a_source_of_two()
    {
        ObjectView ring = new()
        {
            Handle = Handle(1),
            Zone = ZoneKindView.Battlefield,
            Kind = ObjectKindView.Card,
            Owner = Me,
            Controller = Me,
            Name = "Sol Ring",
            CardTypes = [CardTypeView.Artifact],
            OracleText = "{T}: Add {C}{C}.",
            ProducesMana = true,
        };
        GameClient client = Seated([ring, Land(2, "Mountain"), InHand(90, "Shock", CardTypeView.Instant, 2)]);

        AnswerPayload paid = Bot().Decide(client, Owing(90, "{1}", [], 1,
            Tap(1, ManaTypeView.Colorless, ManaTypeView.Colorless), Tap(2, ManaTypeView.Red)));
        Assert.Equal(Handle(2), Assert.IsType<TakeActionAnswer>(paid).Source);
    }

    [Fact]
    public void The_first_spell_of_the_plan_is_paid_from_the_source_the_next_one_does_not_need()
    {
        // Scrubland and Plateau, two mana: a 3/3 for {1} and a 1/1 for {B} are the turn's plan, the 3/3 first. Either land
        // pays the {1}; only the Scrubland pays the {B}, so the Plateau pays first.
        ObjectView golem = InHand(10, "Golem", CardTypeView.Creature, 1, power: 3, toughness: 3);
        golem.Colors = [];
        ObjectView rat = InHand(11, "Rat", CardTypeView.Creature, 1, power: 1, toughness: 1, color: ColorView.Black);
        rat.ManaCost = "{B}";
        ObjectView dragon = InHand(12, "Dragon", CardTypeView.Creature, 5, power: 5, toughness: 5);
        dragon.ManaCost = "{3}{R}{R}";
        GameClient client = Seated([Land(1, "Scrubland"), Land(2, "Plateau"), golem, rat, dragon]);
        TacticalPlay bot = Bot();

        AnswerPayload cast = bot.Decide(client, new PriorityPromptMessage
        {
            Player = Me,
            Options =
            [
                new() { Kind = ActionKindView.CastSpell, Source = Handle(10), Affordable = true },
                new() { Kind = ActionKindView.CastSpell, Source = Handle(11), Affordable = true },
            ],
        });
        Assert.Equal(Handle(10), Assert.IsType<TakeActionAnswer>(cast).Source);

        AnswerPayload paid = bot.Decide(client, Owing(10, "{1}", [], 1,
            Tap(1, ManaTypeView.White), Tap(1, ManaTypeView.Black), Tap(2, ManaTypeView.Red), Tap(2, ManaTypeView.White)));
        Assert.Equal(Handle(2), Assert.IsType<TakeActionAnswer>(paid).Source);
    }
}
