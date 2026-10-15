namespace Cantrip.Core.Random;

/// <summary>
/// Deterministic randomness. Consumers take this interface; game state stores the concrete
/// <see cref="Pcg32Random"/> so that the generator's position is part of the snapshot.
/// Draw order is itself part of the determinism contract: two runs from the same seed must
/// request values in the same sequence.
/// </summary>
public interface IRandom
{
    uint NextUInt32();

    /// <summary>Uniform in [0, exclusiveMax), unbiased.</summary>
    int NextInt(int exclusiveMax);

    /// <summary>Uniform in [minInclusive, exclusiveMax), unbiased.</summary>
    int NextInt(int minInclusive, int exclusiveMax);

    /// <summary>In-place Fisher-Yates. The only sanctioned way to shuffle a library.</summary>
    void Shuffle<T>(IList<T> items);
}
