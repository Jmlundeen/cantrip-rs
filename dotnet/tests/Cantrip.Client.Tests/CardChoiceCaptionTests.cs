using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// What the dock says of a choice of cards: the verb is the card's own (exile is exile, a free cast is a free cast), the kind of
/// card is named where the asking card named one, and "any number" is said where the rules text says it.
/// </summary>
public sealed class CardChoiceCaptionTests
{
    private static ObjectView Card(int handle, string name, ZoneKindView zone, int owner, CardTypeView type = CardTypeView.Creature) => new()
    {
        Handle = Handle(handle),
        Zone = zone,
        ZoneOwner = zone == ZoneKindView.Battlefield ? 0 : owner,
        Kind = ObjectKindView.Card,
        Owner = owner,
        Controller = owner,
        Name = name,
        CardTypes = [type],
    };

    private static ChooseCardsPromptMessage Choice(CardChoicePurposeView purpose, int count, bool upTo, params ObjectView[] cards) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = purpose,
        Cards = [.. cards.Select(card => card.Handle)],
        Eligible = [.. cards.Select(card => card.Handle)],
        Count = count,
        UpTo = upTo,
    };

    private static ObjectView Spell() => new()
    {
        Handle = Handle(90),
        Zone = ZoneKindView.Stack,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Strategic Betrayal",
        CardTypes = [CardTypeView.Sorcery],
    };

    [Fact]
    public void An_exile_of_creatures_on_the_battlefield_says_creature_and_exile()
    {
        ObjectView elves = Card(3, "Llanowar Elves", ZoneKindView.Battlefield, Them);
        GameClient client = Seated([Spell(), elves]);
        ChooseCardsPromptMessage prompt = Choice(CardChoicePurposeView.Exile, 1, false, elves);
        prompt.Kind = "creature";

        CardChoicePresentation shown = CardChoicePresentation.Describe(client, prompt, []);

        Assert.Equal("Strategic Betrayal: choose a creature to exile", shown.Title);
        Assert.Equal("Exile nothing", shown.Confirm);
    }

    [Fact]
    public void An_exile_from_a_hand_names_the_card_kind_and_says_card()
    {
        ObjectView elves = Card(3, "Llanowar Elves", ZoneKindView.Hand, Them);
        GameClient client = Seated([Spell(), elves]);
        ChooseCardsPromptMessage prompt = Choice(CardChoicePurposeView.Exile, 1, false, elves);
        prompt.Kind = "artifact or creature";

        Assert.Equal("Strategic Betrayal: choose an artifact or creature card to exile", CardChoicePresentation.Describe(client, prompt, []).Title);
    }

    [Fact]
    public void Any_number_is_said_where_the_rules_text_names_no_number()
    {
        ObjectView one = Card(3, "Ornithopter", ZoneKindView.Graveyard, Them, CardTypeView.Artifact);
        ObjectView two = Card(4, "Ornithopter", ZoneKindView.Graveyard, Them, CardTypeView.Artifact);
        GameClient client = Seated([Spell(), one, two]);

        Assert.Equal(
            "Strategic Betrayal: choose any number of cards to exile",
            CardChoicePresentation.Describe(client, Choice(CardChoicePurposeView.Exile, byte.MaxValue, true, one, two), []).Title);

        // A count the rules text gave is "up to" that many, even when every card shown could be chosen.
        Assert.Equal(
            "Strategic Betrayal: choose up to 2 cards to exile",
            CardChoicePresentation.Describe(client, Choice(CardChoicePurposeView.Exile, 2, true, one, two), []).Title);
    }

    [Fact]
    public void A_cast_without_paying_says_so_and_a_cast_of_copies_says_copies_and_what_is_left_to_spend()
    {
        ObjectView giant = Card(3, "Bonecrusher Giant", ZoneKindView.Exile, Me);
        GameClient client = Seated([Spell(), giant]);

        Assert.Equal(
            "Strategic Betrayal: choose up to 1 card to cast without paying its mana cost",
            CardChoicePresentation.Describe(client, Choice(CardChoicePurposeView.CastFree, 1, true, giant), []).Title);

        ChooseCardsPromptMessage copies = Choice(CardChoicePurposeView.CastCopyFree, 1, true, giant);
        copies.MaximumManaValue = 4;
        CardChoicePresentation shown = CardChoicePresentation.Describe(client, copies, []);
        Assert.Equal(
            "Strategic Betrayal: choose up to 1 copy to cast without paying its mana cost (total mana value 4 or less left to spend)",
            shown.Title);
        Assert.Equal("Cast nothing", shown.Confirm);

        Assert.Equal(
            "Strategic Betrayal: choose up to 1 card to cast, paying its costs",
            CardChoicePresentation.Describe(client, Choice(CardChoicePurposeView.Cast, 1, true, giant), []).Title);
    }

    [Fact]
    public void A_choice_of_the_permanent_to_put_counters_on_says_which_counters_and_never_keep()
    {
        ObjectView token = Card(3, "Jace", ZoneKindView.Battlefield, Me, CardTypeView.Planeswalker);
        ObjectView copy = Card(4, "Jace, the Mind Sculptor", ZoneKindView.Battlefield, Me, CardTypeView.Planeswalker);
        GameClient client = Seated([Spell(), token, copy]);
        ChooseCardsPromptMessage prompt = Choice(CardChoicePurposeView.PutCounters, 1, false, token, copy);
        prompt.Kind = "Jace planeswalker token";
        prompt.Counter = "loyalty";
        prompt.CounterCount = 2;

        CardChoicePresentation shown = CardChoicePresentation.Describe(client, prompt, []);

        Assert.Equal("Strategic Betrayal: choose a Jace planeswalker token to put 2 loyalty counters on", shown.Title);
        Assert.Equal("Put 2 loyalty counters on it", shown.Confirm);
        // One permanent on the battlefield, chosen as a target is: the click on it answers.
        Assert.True(shown.AnswersOnLastPick);

        // Amass's Army, one +1/+1 counter.
        ChooseCardsPromptMessage army = Choice(CardChoicePurposeView.PutCounters, 1, false, token, copy);
        army.Kind = "Army";
        army.Counter = "+1/+1";
        army.CounterCount = 1;
        CardChoicePresentation amass = CardChoicePresentation.Describe(client, army, []);
        Assert.Equal("Strategic Betrayal: choose an Army to put a +1/+1 counter on", amass.Title);
        Assert.Equal("Put a +1/+1 counter on it", amass.Confirm);
    }
}
