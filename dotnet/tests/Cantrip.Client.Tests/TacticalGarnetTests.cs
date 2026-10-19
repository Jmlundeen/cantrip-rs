using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>
/// Garnet, Princess of Alexandria's "you may remove a lore counter from each of any number of Sagas" ("RemoveLoreCounter", asked
/// once per Saga with the Saga as the prompt's source): the bot takes the counter off every Saga but one a single counter short of
/// its last chapter, whose payoff it would delay.
/// </summary>
public sealed class TacticalGarnetTests
{
    private const string Song =
        "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after III.)\n"
        + "I, II — Until your next turn, creatures you control gain \"{T}: Add one mana of any color.\"\n"
        + "III — Put a +1/+1 counter on each creature you control.";

    private static ObjectView Saga(int lore) => new()
    {
        Handle = Handle(1),
        Zone = ZoneKindView.Battlefield,
        Kind = ObjectKindView.Card,
        Owner = Me,
        Controller = Me,
        Name = "Song of Freyalise",
        CardTypes = [CardTypeView.Enchantment],
        OracleText = Song,
        Counters = [new CounterView { Name = "lore", Count = lore }],
    };

    private static bool Accepts(ObjectView saga) =>
        Assert.IsType<MayChosenAnswer>(
            new TacticalPlay(TacticalModel.Prior(), seed: 1)
                .Decide(Seated([saga]), new MayPromptMessage { Player = Me, Source = Handle(1), Verb = "RemoveLoreCounter" })).Accept;

    [Fact]
    public void Takes_a_lore_counter_off_a_saga_that_is_not_one_chapter_from_its_last()
    {
        Assert.True(Accepts(Saga(1)));
    }

    [Fact]
    public void Leaves_the_counters_on_a_saga_one_chapter_from_its_last()
    {
        Assert.False(Accepts(Saga(2)));
    }
}
