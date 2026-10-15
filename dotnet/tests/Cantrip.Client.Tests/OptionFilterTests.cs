using Cantrip.Client;
using Xunit;

namespace Cantrip.Client.Tests;

/// <summary>Finding one of a long list of named options by typing, as the dock does for "choose a creature type".</summary>
public sealed class OptionFilterTests
{
    private static readonly string[] Types = ["Djinn", "Dragon", "Drake", "Elf", "Hydra", "Time Lord", "Werewolf", "Wolf"];
    private static readonly int[] All = [0, 1, 2, 3, 4, 5, 6, 7];

    [Fact]
    public void Nothing_typed_is_every_option_in_order() =>
        Assert.Equal(All, OptionFilter.Matches(Types, All, "  "));

    [Fact]
    public void An_exact_name_comes_first_then_beginnings_then_the_rest()
    {
        Assert.Equal([1, 2, 4], OptionFilter.Matches(Types, All, "DRA"));
        Assert.Equal([7, 6], OptionFilter.Matches(Types, All, "wolf"));
    }

    [Fact]
    public void Only_eligible_options_are_matched() =>
        Assert.Equal([2, 4], OptionFilter.Matches(Types, [2, 4, 99], "dra"));

    [Fact]
    public void Enter_settles_on_an_exact_name_or_a_lone_match()
    {
        Assert.Equal(7, OptionFilter.Settled(Types, All, "Wolf"));
        Assert.Equal(5, OptionFilter.Settled(Types, All, "time"));
        Assert.Null(OptionFilter.Settled(Types, All, "dra"));
        Assert.Null(OptionFilter.Settled(Types, All, "zzz"));
        Assert.Null(OptionFilter.Settled(Types, All, string.Empty));
        Assert.Null(OptionFilter.Settled(Types, [0, 1], "Wolf"));
    }
}
