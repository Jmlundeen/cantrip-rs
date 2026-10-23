using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Enlist (CR 702.154a, 508.1g): as an attacker with enlist attacks, its controller may tap up to one creature that isn't
/// attacking. The caption says what tapping it does, declining is an answer, and every seat answers it legally.
/// </summary>
public sealed class EnlistChoiceTests
{
    private static ObjectView Creature(int handle, string name, int power) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Battlefield,
        ZoneOwner = Me,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = name,
        ManaValue = 2,
        CardTypes = [CardTypeView.Creature],
        Power = power,
        Toughness = 2,
    };

    private static ChooseCardsPromptMessage Choice(params int[] creatures) => new()
    {
        Player = Me,
        Owner = Me,
        Source = Handle(90),
        Purpose = CardChoicePurposeView.Enlist,
        Cards = [.. creatures.Select(Handle)],
        Verdict = new() { Addable = [.. creatures.Select(Handle)], Complete = true, CompleteAlone = [.. creatures.Select(Handle)] },
        Count = 1,
        CountKind = CountKindView.UpTo,
        Kind = "creature",
    };

    private static GameClient Board() => Seated([Creature(90, "Guardian of New Benalia", 2), Creature(10, "Grizzly Bears", 2), Creature(11, "Hill Giant", 3)]);

    [Fact]
    public void The_caption_says_what_enlisting_does_and_declining_is_an_answer()
    {
        GameClient client = Board();
        ChooseCardsPromptMessage prompt = Choice(10, 11);

        CardChoicePresentation none = CardChoicePresentation.Describe(client, prompt, []);
        Assert.Equal("Guardian of New Benalia has enlist: you may tap a creature you control that isn't attacking, and Guardian of New Benalia gets +X/+0 until end of turn, X its power", none.Title);
        Assert.Equal("Don't enlist", none.Confirm);
        Assert.True(none.CanSubmit);
        Assert.Contains("attacks without enlisting", none.Detail);

        CardChoicePresentation giant = Judged(client, prompt, [11]);
        Assert.Equal("Enlist it", giant.Confirm);
        Assert.Contains("Hill Giant (power 3)", giant.Detail);
        Assert.True(giant.CanSubmit);
    }

    [Fact]
    public void Every_seat_enlists_at_most_one_creature_it_was_offered()
    {
        GameClient client = Board();
        ChooseCardsPromptMessage prompt = Choice(10, 11);
        AnswerPayload tactical = new TacticalPlay(TacticalModel.Prior(), seed: 1).Decide(client, prompt);
        AnswerPayload random = new RandomPlay(seed: 1).Decide(prompt);
        foreach (AnswerPayload answer in new[] { tactical, random })
        {
            CardsChosenAnswer chosen = Assert.IsType<CardsChosenAnswer>(answer);
            Assert.True(chosen.Cards.Count <= 1);
            Assert.All(chosen.Cards, card => Assert.Contains(card, prompt.Cards));
        }
    }
}
