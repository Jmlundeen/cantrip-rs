using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// A card an effect exiles to become plotted (CR 702.170a, 702.170d; Jace Reawakened): its caption says the card is exiled and
/// plotted and what plotting lets its owner do, and the tactical seat treats it as a gain - the card whose mana cost is most
/// worth not paying - never as a card given up, which an up-to-one exile would decline.
/// </summary>
public sealed class PlotChoiceTests
{
    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Jace() => new()
    {
        Handle = Handle(90),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Jace Reawakened",
        ManaValue = 2,
        OracleText = "+1: You may exile a nonland card with mana value 3 or less from your hand. If you do, it becomes plotted.",
        CardTypes = [CardTypeView.Planeswalker],
    };

    private static ChooseCardsPromptMessage Choice(bool upTo, params int[] cards) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = CardChoicePurposeView.Plot,
        Cards = [.. cards.Select(Handle)],
        Eligible = [.. cards.Select(Handle)],
        Count = 1,
        UpTo = upTo,
        Kind = "nonland",
    };

    private static IReadOnlyList<ViewHandle> Chosen(GameClient client, ChooseCardsPromptMessage prompt) =>
        Assert.IsType<CardsChosenAnswer>(Bot().Decide(client, prompt)).Cards;

    [Fact]
    public void The_caption_says_exile_and_plot_and_what_a_plotted_card_may_do()
    {
        GameClient client = Seated([Jace(), InHand(10, "Hill Giant", CardTypeView.Creature, 3, power: 3, toughness: 3)]);
        ChooseCardsPromptMessage prompt = Choice(true, 10);

        CardChoicePresentation none = CardChoicePresentation.Describe(client, prompt, []);
        Assert.Equal("Jace Reawakened: choose up to 1 nonland card to exile and plot", none.Title);
        Assert.Contains("cast as a sorcery on a later turn without paying its mana cost", none.Detail);
        Assert.Equal("Plot nothing", none.Confirm);
        Assert.True(none.CanSubmit);

        CardChoicePresentation one = CardChoicePresentation.Describe(client, prompt, [10]);
        Assert.Equal("Exile and plot it", one.Confirm);
        Assert.True(one.CanSubmit);
    }

    [Fact]
    public void The_seat_plots_a_card_rather_than_declining()
    {
        GameClient client = Seated([Jace(), InHand(10, "Grizzly Bears", CardTypeView.Creature, 2, power: 2, toughness: 2)]);

        Assert.Equal([Handle(10)], Chosen(client, Choice(true, 10)));
    }

    [Fact]
    public void The_seat_plots_the_card_of_highest_mana_value()
    {
        GameClient client = Seated([
            Jace(),
            InHand(10, "Shock", CardTypeView.Instant, 1, "Shock deals 2 damage to any target."),
            InHand(11, "Hill Giant", CardTypeView.Creature, 3, power: 3, toughness: 3),
            InHand(12, "Grizzly Bears", CardTypeView.Creature, 2, power: 2, toughness: 2),
        ]);

        Assert.Equal([Handle(11)], Chosen(client, Choice(true, 10, 11, 12)));
    }

    [Fact]
    public void Among_equals_the_seat_keeps_its_instant_for_instant_speed()
    {
        // Plotted, an instant can be cast only as a sorcery (CR 702.170d): of two of mana value 2, the sorcery-speed card is plotted.
        GameClient client = Seated([
            Jace(),
            InHand(10, "Counterspell", CardTypeView.Instant, 2, "Counter target spell.", color: ColorView.Blue),
            InHand(11, "Grizzly Bears", CardTypeView.Creature, 2, power: 2, toughness: 2),
        ]);

        Assert.Equal([Handle(11)], Chosen(client, Choice(true, 10, 11)));
    }

    [Fact]
    public void A_card_of_mana_value_zero_saves_nothing_and_is_not_plotted_when_the_seat_may_decline()
    {
        GameClient client = Seated([Jace(), InHand(10, "Ornithopter", CardTypeView.Artifact, 0, power: 0, toughness: 2)]);

        Assert.Empty(Chosen(client, Choice(true, 10)));
    }

    [Fact]
    public void A_choice_that_must_be_made_still_takes_a_card()
    {
        GameClient client = Seated([Jace(), InHand(10, "Ornithopter", CardTypeView.Artifact, 0, power: 0, toughness: 2)]);

        Assert.Equal([Handle(10)], Chosen(client, Choice(false, 10)));
    }
}
