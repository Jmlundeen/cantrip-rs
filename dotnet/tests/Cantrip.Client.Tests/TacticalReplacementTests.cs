using Cantrip.Client.Tactics;
using Cantrip.Protocol;
using Xunit;
using static Cantrip.Client.Tests.TacticsBoard;

namespace Cantrip.Client.Tests;

/// <summary>Which of several replacement effects applies first (CR 616.1), as the seat whose event it is.</summary>
public sealed class TacticalReplacementTests
{
    private static int Chosen(string @event, params string[] hints)
    {
        GameClient client = Seated([Land(1)]);
        var prompt = new ChooseReplacementPromptMessage
        {
            Player = Me,
            Event = @event,
            Sources = [.. hints.Select((_, i) => Handle(20 + i))],
            Hints = [.. hints],
        };
        return Assert.IsType<ReplacementChosenAnswer>(new TacticalPlay(TacticalModel.Prior(), seed: 1).Decide(client, prompt)).Index;
    }

    [Fact]
    public void Adds_a_counter_before_doubling_them_on_its_own_permanent()
    {
        // Hardened Scales then Doubling Season: (1 + 1) x 2 = 4, rather than 1 x 2 + 1 = 3.
        Assert.Equal(1, Chosen("1 +1/+1 counter would be put on Walking Ballista",
            "puts twice that many counters instead", "puts that many plus one +1/+1 counters instead"));
        Assert.Equal(0, Chosen("1 +1/+1 counter would be put on Walking Ballista",
            "puts one more +1/+1 counter", "puts twice that many counters"));
    }

    [Fact]
    public void Leaves_minus_one_counters_to_the_default()
    {
        Assert.Equal(0, Chosen("1 -1/-1 counter would be put on Kitchen Finks",
            "puts twice that many counters instead", "puts that many plus one +1/+1 counters instead"));
    }

    [Fact]
    public void Prevents_damage_to_itself_before_doubling_it()
    {
        // Furnace of Rath's ruling: 5 damage with 4 prevented first and then doubled is 2, rather than 6.
        Assert.Equal(1, Chosen("5 damage from Lava Axe to you",
            "deals double that damage instead", "prevents the damage"));
    }
}
