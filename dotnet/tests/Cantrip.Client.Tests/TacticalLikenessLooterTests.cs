using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// The X of Likeness Looter's "becomes a copy of target creature card in your graveyard with mana value X": announced at
/// the mana value of a creature card worth becoming, never at a 0/0 that would die as the copy (CR 704.5f), and given up
/// when there is none.
/// </summary>
public sealed class TacticalLikenessLooterTests
{
    private const string LooterText =
        "Flying\n{T}: Draw a card, then discard a card.\n{X}: This creature becomes a copy of target creature card in your graveyard with mana value X, except it has flying and this ability. Activate only as a sorcery.";

    private static TacticalPlay Bot() => new(TacticalModel.Prior(), seed: 1);

    private static ObjectView Dead(int handle, string name, int manaValue, int power, int toughness) => new()
    {
        Handle = Handle(handle),
        Zone = ZoneKindView.Graveyard,
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

    private static PayCostsPromptMessage Announcing(int most) => new()
    {
        Player = Me,
        Spell = Handle(1),
        AnnouncesXOrKicks = true,
        HasX = true,
        X = 0,
        MostAffordable = most,
        Options =
        [
            new() { Kind = ActionKindView.IncreaseX },
            new() { Kind = ActionKindView.ConfirmCasting },
            new() { Kind = ActionKindView.CancelCasting },
        ],
    };

    [Fact]
    public void With_only_a_zero_toughness_card_in_reach_the_copy_is_given_up()
    {
        // Walking Ballista has mana value 0 in the graveyard and is 0/0: the Looter would become it with no counters and die.
        GameClient client = Seated([Creature(1, "Likeness Looter", 1, 1, text: LooterText), Dead(20, "Walking Ballista", 0, 0, 0)]);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Announcing(3)));

        Assert.Equal(ActionKindView.CancelCasting, answer.Kind);
    }

    [Fact]
    public void It_raises_x_towards_the_creature_card_worth_becoming()
    {
        GameClient client = Seated(
        [
            Creature(1, "Likeness Looter", 1, 1, text: LooterText),
            Dead(20, "Walking Ballista", 0, 0, 0),
            Dead(21, "Troll", 2, 4, 4),
        ]);

        TakeActionAnswer answer = Assert.IsType<TakeActionAnswer>(Bot().Decide(client, Announcing(3)));

        Assert.Equal(ActionKindView.IncreaseX, answer.Kind);
    }
}
