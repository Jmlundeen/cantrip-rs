using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Harmonize (CR 702.180a): the creature tapped to take its power off the generic mana of the cost is "up to one", its caption says
/// what tapping does to the cost, and the tactical seat taps one only when the mana for the cost is not there without it.
/// </summary>
public sealed class HarmonizeTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView WildRide() => new()
    {
        Handle = Handle(90),
        Zone = ZoneKindView.Stack,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Wild Ride",
        ManaValue = 1,
        OracleText = "Target creature gets +3/+0 and gains haste until end of turn.\nHarmonize {4}{R} (You may cast this card from your graveyard for its harmonize cost.)",
        CardTypes = [CardTypeView.Sorcery],
    };

    private static ObjectView Land(int handle) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Mountain",
        CardTypes = [CardTypeView.Land],
        ProducesMana = true,
    };

    private static ObjectView Creature(int handle, string name, int power) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = name,
        ManaValue = power,
        CardTypes = [CardTypeView.Creature],
        Power = power,
        Toughness = 2,
    };

    private static ChooseCardsPromptMessage Choice(params (int Handle, int Power)[] creatures) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = CardChoicePurposeView.TapToReduce,
        Cards = [.. creatures.Select(c => Handle(c.Handle))],
        Verdict = new() { Addable = [.. creatures.Select(c => Handle(c.Handle))], Complete = true },
        Powers = [.. creatures.Select(c => c.Power)],
        Count = 1,
        CountKind = CountKindView.UpTo,
        Kind = "creature",
    };

    [Fact]
    public void The_caption_says_up_to_one_and_what_tapping_takes_off_the_cost()
    {
        ObjectView bear = Creature(10, "Grizzly Bears", 2);
        GameClient client = Seated([WildRide(), bear]);
        ChooseCardsPromptMessage prompt = Choice((10, 2));

        CardChoicePresentation none = CardChoicePresentation.Describe(client, prompt, []);
        Assert.Equal("Wild Ride: tap up to one untapped creature you control to reduce the cost by generic mana equal to its power", none.Title);
        Assert.Equal("Tap nothing", none.Confirm);
        Assert.True(none.CanSubmit);
        Assert.False(none.AnswersOnLastPick);
        Assert.Contains("full cost", none.Detail);

        CardChoicePresentation tapped = Judged(client, prompt, [10]);
        Assert.Equal("Tap it (-2)", tapped.Confirm);
        Assert.Contains("2 off the generic mana", tapped.Detail);
        Assert.True(tapped.CanSubmit);
    }

    [Fact]
    public void A_cost_not_settled_yet_says_up_to_the_power_never_an_exact_amount()
    {
        // Nature's Rhythm's harmonize {X}{G}{G}{G}{G}: the creature is chosen before X is announced (CR 702.180b, 601.2b), so
        // the host says each power is the most tapping it could take off (`PowersUpTo`), and the caption must not promise
        // exactly that - nor 0, the generic mana the cost has before X.
        ObjectView elves = Creature(10, "Llanowar Elves", 1);
        GameClient client = Seated([WildRide(), elves]);
        ChooseCardsPromptMessage prompt = Choice((10, 1));
        prompt.PowersUpTo = true;

        CardChoicePresentation tapped = Judged(client, prompt, [10]);
        Assert.Equal("Tap it (up to -1)", tapped.Confirm);
        Assert.StartsWith("Tapping it takes up to 1 off the generic mana of the cost", tapped.Detail);
    }

    [Fact]
    public void A_creature_of_power_zero_is_tapped_without_promising_a_reduction()
    {
        ObjectView thopter = Creature(10, "Ornithopter", 0);
        GameClient client = Seated([WildRide(), thopter]);

        Assert.Equal("Tap it", CardChoicePresentation.Describe(client, Choice((10, 0)), [10]).Confirm);
    }

    [Fact]
    public void The_seat_taps_nothing_while_the_mana_pays_the_cost_without_it()
    {
        GameClient client = Seated([WildRide(), Land(1), Land(2), Land(3), Land(4), Land(5), Creature(10, "Grizzly Bears", 2)]);

        Assert.Empty(Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Choice((10, 2)))).Cards);
    }

    [Fact]
    public void The_seat_taps_the_creature_that_makes_up_the_shortfall_and_is_worth_least()
    {
        // Three lands for {4}{R} (five): two short. Of power 2 and 3 both make it up; the 2-power creature is worth less.
        GameClient client = Seated([WildRide(), Land(1), Land(2), Land(3), Creature(10, "Grizzly Bears", 2), Creature(11, "Hill Giant", 3)]);

        Assert.Equal([Handle(10)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Choice((10, 2), (11, 3)))).Cards);
    }

    [Fact]
    public void The_seat_taps_the_strongest_when_none_alone_makes_up_the_shortfall()
    {
        // Two lands: three short, and neither creature has the power; the stronger gets it nearest.
        GameClient client = Seated([WildRide(), Land(1), Land(2), Creature(10, "Grizzly Bears", 2), Creature(11, "Hill Giant", 3)]);

        Assert.Equal([Handle(11)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Choice((10, 2), (11, 3)))).Cards);
    }

    [Fact]
    public void The_seat_does_not_tap_the_creature_whose_mana_it_needs()
    {
        // Three lands and Llanowar Elves are four mana for the five; the Elves (power 1) would leave three for {3}{R}, the bears (power 2) four for {2}{R}.
        ObjectView elves = Creature(11, "Llanowar Elves", 1);
        elves.ProducesMana = true;
        GameClient client = Seated([WildRide(), Land(1), Land(2), Land(3), Creature(10, "Grizzly Bears", 2), elves]);

        Assert.Equal([Handle(10)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Choice((11, 1), (10, 2)))).Cards);
    }
}
