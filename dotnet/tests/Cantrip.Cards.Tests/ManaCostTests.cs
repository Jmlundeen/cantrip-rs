using Cantrip.Cards.Model;
using Xunit;

namespace Cantrip.Cards.Tests;

/// <summary>Monocolored hybrid symbols in the printed cost model (CR 107.4e, 202.2d, 202.3f).</summary>
public sealed class ManaCostTests
{
    /// <summary>
    /// Kin-Tree Severance's {2/W}{2/B}{2/G}: mana value 6, white, black and green, and written back as printed. Paid with
    /// mana of any type each symbol is one mana (its colored half).
    /// </summary>
    [Fact]
    public void MonocoloredHybridCountsTwoAndIsItsColor()
    {
        ManaCost cost = ManaCost.Parse("{2/W}{2/B}{2/G}");

        Assert.Equal(6, cost.ManaValue);
        Assert.Equal([ManaColor.White, ManaColor.Black, ManaColor.Green], cost.Colors);
        Assert.Equal("{2/W}{2/B}{2/G}", cost.ToString());
        Assert.All(cost.Hybrid, symbol => Assert.Equal(2, symbol.GenericCost));
        Assert.Equal(3, cost.AnyType().ManaValue);
        Assert.True(cost.Hybrid[0].Accepts(ManaColor.White));
        Assert.False(cost.Hybrid[0].Accepts(ManaColor.Blue));
    }

    /// <summary>{C/W}: one colorless mana or one white pays it; mana value 1, white.</summary>
    [Fact]
    public void ColorlessHybridIsPaidWithColorlessOrItsColor()
    {
        ManaCost cost = ManaCost.Parse("{1}{C/W}");

        Assert.Equal(2, cost.ManaValue);
        Assert.Equal([ManaColor.White], cost.Colors);
        Assert.Equal("{1}{C/W}", cost.ToString());
        Assert.True(cost.Hybrid[0].PaidWithColorless);
        Assert.Null(cost.Hybrid[0].GenericCost);
    }
}
