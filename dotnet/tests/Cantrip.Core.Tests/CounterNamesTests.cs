using Xunit;

namespace Cantrip.Core.Tests;

/// <summary>
/// How a counter's name is compared and how it is spelled as a key (CR 122.1). The comparison is the important
/// half: it is what decides whether two cards are talking about the same counter.
/// </summary>
public sealed class CounterNamesTests
{
    [Theory]
    [InlineData("mining", "mining")]
    [InlineData("Mining", "mining")]
    [InlineData("  MINING  ", "mining")]
    [InlineData("+1/+1", "+1/+1")]
    [InlineData("", "")]
    [InlineData("   ", "")]
    [InlineData(null, "")]
    public void Case_and_surrounding_space_are_how_a_name_is_written_not_which_counter_it_is(string? written, string expected) =>
        Assert.Equal(expected, CounterNames.Normalize(written));

    /// <summary>
    /// The two the rules write in words keep the spelling their dump keys already had, so every scenario that
    /// checks a +1/+1 counter still names the same key; everything else is its own name, PascalCased.
    /// </summary>
    [Theory]
    [InlineData("+1/+1", "PlusOnePlusOne")]
    [InlineData("-1/-1", "MinusOneMinusOne")]
    [InlineData("mining", "Mining")]
    [InlineData("Mining", "Mining")]
    [InlineData("stun", "Stun")]
    [InlineData("+2/+2", "Plus2Plus2")]
    [InlineData("time in a bottle", "TimeInABottle")]
    [InlineData("", "")]
    public void A_name_becomes_a_key_without_losing_the_word(string written, string expected) =>
        Assert.Equal(expected, CounterNames.Token(written));

    /// <summary>
    /// The names the rules attach meaning to are all distinct, since two that collided would make one counter of
    /// two rules — a stun counter that also granted flying, say.
    /// </summary>
    [Fact]
    public void The_well_known_names_are_all_different()
    {
        string[] names =
        [
            CounterNames.PlusOnePlusOne, CounterNames.MinusOneMinusOne, CounterNames.Loyalty,
            CounterNames.Indestructible, CounterNames.Lifelink, CounterNames.Flying, CounterNames.Hexproof,
            CounterNames.FirstStrike, CounterNames.DoubleStrike, CounterNames.Deathtouch, CounterNames.Decayed, CounterNames.Haste,
            CounterNames.Menace, CounterNames.Reach, CounterNames.Shadow, CounterNames.Trample, CounterNames.Vigilance,
            CounterNames.Stun, CounterNames.Finality, CounterNames.Lore, CounterNames.Charge, CounterNames.Corpse,
        ];

        Assert.Equal(names.Length, names.Distinct(StringComparer.Ordinal).Count());
        Assert.All(names, name => Assert.Equal(name, CounterNames.Normalize(name)));
    }
}
