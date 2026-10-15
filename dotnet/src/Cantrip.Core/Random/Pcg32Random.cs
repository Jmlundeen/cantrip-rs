namespace Cantrip.Core.Random;

/// <summary>
/// PCG-XSH-RR 32-bit generator (O'Neill 2014). Chosen over <c>System.Random</c> because the
/// framework implementation is explicitly not guaranteed stable across runtimes, and a shuffle
/// that differs between the server build and a replaying client is a desync we could not
/// debug. This is pure integer arithmetic, so it is identical on every platform.
/// </summary>
public sealed class Pcg32Random : IRandom
{
    private const ulong Multiplier = 6364136223846793005UL;

    // Public fields, not properties: this object lives in game state and is walked by the
    // state shape model for cloning, hashing, and purity verification.
    public ulong State;
    public ulong Increment;

    /// <summary>Parameterless constructor exists for the state cloner.</summary>
    public Pcg32Random()
    {
    }

    public Pcg32Random(ulong seed, ulong sequence = 0xDA3E39CB94B95BDBUL)
    {
        State = 0UL;
        Increment = (sequence << 1) | 1UL;
        Step();
        State += seed;
        Step();
    }

    private void Step() => State = unchecked(State * Multiplier + Increment);

    public uint NextUInt32()
    {
        ulong old = State;
        Step();
        uint xorshifted = (uint)(((old >> 18) ^ old) >> 27);
        int rot = (int)(old >> 59);
        return unchecked((xorshifted >> rot) | (xorshifted << ((-rot) & 31)));
    }

    public int NextInt(int exclusiveMax)
    {
        if (exclusiveMax <= 0)
        {
            throw new ArgumentOutOfRangeException(nameof(exclusiveMax), exclusiveMax, "Bound must be positive.");
        }

        uint bound = (uint)exclusiveMax;

        // Rejection sampling: discard the unrepresentable low window so the result is unbiased.
        // Biased modulo would be invisible in play and visible in a 10,000-game fuzz run.
        uint threshold = unchecked(0u - bound) % bound;
        while (true)
        {
            uint r = NextUInt32();
            if (r >= threshold)
            {
                return (int)(r % bound);
            }
        }
    }

    public int NextInt(int minInclusive, int exclusiveMax)
    {
        if (exclusiveMax <= minInclusive)
        {
            throw new ArgumentOutOfRangeException(nameof(exclusiveMax), exclusiveMax, "Empty range.");
        }

        return minInclusive + NextInt(exclusiveMax - minInclusive);
    }

    public void Shuffle<T>(IList<T> items)
    {
        ArgumentNullException.ThrowIfNull(items);

        for (int i = items.Count - 1; i > 0; i--)
        {
            int j = NextInt(i + 1);
            (items[i], items[j]) = (items[j], items[i]);
        }
    }
}
