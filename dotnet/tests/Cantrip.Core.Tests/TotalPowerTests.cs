using Cantrip.Core.Decisions;
using Xunit;

namespace Cantrip.Core.Tests;

/// <summary>Saddle's "total power N or greater" (CR 702.171a), the rule the engine's prompt and the client's button share.</summary>
public sealed class TotalPowerTests
{
    private static readonly int[] Eligible = [4, 5, 6];
    private static readonly int[] Powers = [2, 1, 4];

    [Fact]
    public void Counts_each_chosen_card_once_and_ignores_cards_not_offered()
    {
        Assert.Equal(3, TotalPower.Of(Eligible, Powers, [4, 5, 5, 9]));
        Assert.False(TotalPower.Meets(Eligible, Powers, [4], 3));
        Assert.True(TotalPower.Meets(Eligible, Powers, [4, 5], 3));
        Assert.True(TotalPower.Meets(Eligible, Powers, [6], 3));
    }

    [Fact]
    public void A_minimum_of_zero_asks_nothing() =>
        Assert.True(TotalPower.Meets(Eligible, Powers, [], 0));
}
