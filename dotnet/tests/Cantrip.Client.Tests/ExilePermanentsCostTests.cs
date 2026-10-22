using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// "Exile a creature you control" as a cost (CR 118.3, 601.2h: The Soul Stone, Food Chain, Soul Exchange): the choice is
/// captioned as permanents exiled to pay, picked on the battlefield; the tactical seat exiles the creature it can best
/// spare, and does not activate an ability that costs one of its creatures for nothing it can price.
/// </summary>
public sealed class ExilePermanentsCostTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Stone(string text) => new()
    {
        Handle = Handle(90),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "The Soul Stone",
        CardTypes = [CardTypeView.Artifact],
        OracleText = text,
    };

    private static ChooseCardsPromptMessage Exile(params int[] creatures) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = CardChoicePurposeView.ExileToPay,
        Cards = [.. creatures.Select(Handle)],
        Verdict = new() { Addable = [.. creatures.Select(Handle)] },
        Count = 1,
        Kind = "creature",
    };

    private static ObjectView Angel()
    {
        ObjectView angel = Creature(12, "Serra Angel", 4, 4, text: "Flying, vigilance");
        angel.ManaValue = 5;
        return angel;
    }

    private static ObjectView Elves()
    {
        ObjectView elves = Creature(11, "Llanowar Elves", 1, 1, text: "{T}: Add {G}.");
        elves.ManaValue = 1;
        return elves;
    }

    [Fact]
    public void The_caption_names_a_creature_exiled_to_pay_and_picked_on_the_battlefield()
    {
        GameClient client = Seated([Stone(""), Elves(), Angel()]);

        CardChoicePresentation shown = CardChoicePresentation.Describe(client, Exile(11, 12), []);
        Assert.Equal("The Soul Stone: choose a creature to exile to pay for it", shown.Title);
        Assert.Equal(ZoneKindView.Battlefield, shown.BoardZone);
        Assert.True(shown.AnswersOnLastPick);
    }

    [Fact]
    public void The_seat_exiles_the_creature_it_can_best_spare()
    {
        GameClient client = Seated([Stone(""), Elves(), Angel()]);

        Assert.Equal([Handle(11)], Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, Exile(12, 11))).Cards);
    }

    [Fact]
    public void The_seat_does_not_give_up_a_creature_to_an_ability_it_cannot_price()
    {
        // The same ability for {T} is taken (the positive control); for a creature exiled, it is not.
        const string Tapped = "{T}: Draw a card.";
        const string Exiled = "Exile a creature you control: Draw a card.";
        GameClient tapping = Seated([Stone(Tapped), Elves(), Angel()]);
        Assert.IsType<TakeActionAnswer>(Bot().Decide(tapping, Priority(Tapped)));

        GameClient exiling = Seated([Stone(Exiled), Elves(), Angel()]);
        Assert.IsType<PassAnswer>(Bot().Decide(exiling, Priority(Exiled)));
    }

    private static PriorityPromptMessage Priority(string description) => new()
    {
        Player = Me,
        Options =
        [
            new() { Kind = ActionKindView.ActivateAbility, Source = Handle(90), AbilityIndex = 0, Affordable = true, Description = description },
        ],
    };
}
