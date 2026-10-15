using Cantrip.Core.Random;
using Xunit;

namespace Cantrip.Core.Tests;

public sealed class Pcg32RandomTests
{
    [Fact]
    public void Same_seed_gives_the_same_sequence()
    {
        var a = new Pcg32Random(12345);
        var b = new Pcg32Random(12345);

        for (int i = 0; i < 100; i++)
        {
            Assert.Equal(a.NextUInt32(), b.NextUInt32());
        }
    }

    [Fact]
    public void Different_seeds_diverge()
    {
        var a = new Pcg32Random(1);
        var b = new Pcg32Random(2);

        bool diverged = false;
        for (int i = 0; i < 20 && !diverged; i++)
        {
            diverged = a.NextUInt32() != b.NextUInt32();
        }

        Assert.True(diverged);
    }

    [Theory]
    [InlineData(1)]
    [InlineData(2)]
    [InlineData(3)]
    [InlineData(7)]
    [InlineData(40)]
    [InlineData(60)]
    public void NextInt_stays_in_range(int bound)
    {
        var rng = new Pcg32Random(99);

        for (int i = 0; i < 10_000; i++)
        {
            int value = rng.NextInt(bound);
            Assert.InRange(value, 0, bound - 1);
        }
    }

    [Fact]
    public void NextInt_rejects_a_nonpositive_bound()
    {
        var rng = new Pcg32Random(1);

        Assert.Throws<ArgumentOutOfRangeException>(() => rng.NextInt(0));
        Assert.Throws<ArgumentOutOfRangeException>(() => rng.NextInt(-1));
        Assert.Throws<ArgumentOutOfRangeException>(() => rng.NextInt(5, 5));
    }

    [Fact]
    public void NextInt_is_close_to_uniform()
    {
        // Not a statistical proof, just enough to catch a modulo-bias regression, which is
        // invisible in a single game and obvious across a fuzz run.
        var rng = new Pcg32Random(2024);
        const int buckets = 7;
        const int draws = 140_000;
        int[] counts = new int[buckets];

        for (int i = 0; i < draws; i++)
        {
            counts[rng.NextInt(buckets)]++;
        }

        const int expected = draws / buckets;
        foreach (int count in counts)
        {
            Assert.InRange(count, (int)(expected * 0.95), (int)(expected * 1.05));
        }
    }

    [Fact]
    public void Shuffle_is_a_permutation_and_is_reproducible()
    {
        List<int> first = [.. Enumerable.Range(0, 52)];
        List<int> second = [.. Enumerable.Range(0, 52)];

        new Pcg32Random(4242).Shuffle(first);
        new Pcg32Random(4242).Shuffle(second);

        Assert.Equal(first, second);
        Assert.Equal(Enumerable.Range(0, 52), first.Order());
        Assert.NotEqual(Enumerable.Range(0, 52), first);
    }

    [Fact]
    public void Shuffle_handles_degenerate_sizes()
    {
        var rng = new Pcg32Random(1);

        List<int> empty = [];
        rng.Shuffle(empty);
        Assert.Empty(empty);

        List<int> single = [9];
        rng.Shuffle(single);
        Assert.Equal([9], single);
    }

    /// <summary>
    /// Frozen output for a known seed. The point is not that these particular numbers are
    /// special, it is that they must never change: a player-hosted server and a client replaying
    /// its decision log have to shuffle identically, so a "harmless" tweak to the generator is a
    /// breaking change and should fail here loudly.
    /// </summary>
    [Fact]
    public void Output_for_a_known_seed_is_frozen()
    {
        var rng = new Pcg32Random(42);
        uint[] actual = new uint[4];
        for (int i = 0; i < actual.Length; i++)
        {
            actual[i] = rng.NextUInt32();
        }

        uint[] expected = [1898997482u, 1014631766u, 4096008554u, 633901381u];

        Assert.Equal(expected, actual);
    }
}
