using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Web-slinging (CR 702.188a): the tapped creature returned to pay for the spell is chosen once its mana is paid, with a
/// caption that says so, and the tactical seat pays with a creature's mana when none of its creatures is tapped yet, then
/// returns the creature it can best spare.
/// </summary>
public sealed class WebSlingingTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private const string Text =
        "Web-slinging {W} (You may cast this spell for {W} if you also return a tapped creature you control to its owner's hand.)";

    private static ObjectView SpiderMan(ZoneKindView zone) => new()
    {
        Handle = Handle(90),
        Zone = zone,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Spider-Man, Web-Slinger",
        ManaValue = 3,
        ManaCost = "{2}{W}",
        OracleText = Text,
        CardTypes = [CardTypeView.Creature],
        Colors = [ColorView.White],
        Power = 3,
        Toughness = 3,
    };

    private static ObjectView Elves(bool tapped = false)
    {
        ObjectView elves = Creature(11, "Llanowar Elves", 1, 1, text: "{T}: Add {G}.", tapped: tapped);
        elves.ProducesMana = true;
        elves.ManaValue = 1;
        return elves;
    }

    private static ChooseCardsPromptMessage Return(params int[] creatures) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = CardChoicePurposeView.ReturnToHand,
        Cards = [.. creatures.Select(Handle)],
        Eligible = [.. creatures.Select(Handle)],
        Count = 1,
        Kind = "tapped creature",
    };

    [Fact]
    public void The_caption_names_a_tapped_creature_returned_to_pay_for_the_spell()
    {
        GameClient client = Seated([SpiderMan(ZoneKindView.Stack), Elves(tapped: true)]);

        // A cost's choice is asked cancellable, which the server words as a cost (PromptPresentation): a return to hand alone
        // could as well be an effect's (Words of Wind).
        var cost = new PromptPresentationContext
        {
            Action = PromptActionView.ReturnToHand, Cause = PromptCauseView.Cost, Chooser = Me, Owner = Me, Source = Handle(90),
            Minimum = 1, Maximum = 1,
        };
        CardChoicePresentation shown = CardChoicePresentation.Describe(client, Return(11), [], cost);
        Assert.Equal("Spider-Man, Web-Slinger: choose a tapped creature to return to its owner's hand to pay for it", shown.Title);
    }

    [Fact]
    public void The_seat_returns_the_tapped_creature_it_can_best_spare()
    {
        ObjectView angel = Creature(12, "Serra Angel", 4, 4, text: "Flying, vigilance", tapped: true);
        angel.ManaValue = 5;
        GameClient client = Seated([SpiderMan(ZoneKindView.Stack), Elves(tapped: true), angel]);

        Assert.Equal([Handle(11)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Return(12, 11))).Cards);
    }

    [Fact]
    public void The_seat_pays_web_slinging_with_an_untapped_creatures_mana_so_it_has_one_to_return()
    {
        // A Plains and Llanowar Elves, nothing tapped: {2}{W} is out of reach, web-slinging's {W} is not. Paying the {W}
        // with the Plains would leave no tapped creature to return, so the Elves pay first.
        ObjectView elves = Elves();
        GameClient client = Seated([SpiderMan(ZoneKindView.Hand), Land(1, "Plains"), elves]);
        TacticalPlay bot = Bot();

        AnswerPayload cast = bot.Decide(client, new PriorityPromptMessage
        {
            Player = Me,
            Options =
            [
                new() { Kind = ActionKindView.CastSpell, Source = Handle(90), AbilityIndex = 1, Description = "Web-slinging {W}", Affordable = true, TappedAfterMana = true },
            ],
        });
        Assert.Equal(1, Assert.IsType<TakeActionAnswer>(cast).AbilityIndex);

        AnswerPayload paid = bot.Decide(client, new PayCostsPromptMessage
        {
            Player = Me,
            Spell = Handle(90),
            Total = "{W}",
            Remaining = "{W}",
            RemainingColored = [ColorView.White],
            Options =
            [
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(1), Produces = [ManaTypeView.White], Affordable = true },
                new() { Kind = ActionKindView.ActivateAbility, Source = Handle(11), Produces = [ManaTypeView.Green], Affordable = true },
                new() { Kind = ActionKindView.CancelCasting },
            ],
        });
        Assert.Equal(Handle(11), Assert.IsType<TakeActionAnswer>(paid).Source);
    }
}
