namespace Cantrip.Core.Decisions;

/// <summary>
/// The rule behind "tap any number of other untapped creatures you control with total power N or greater" — saddle's
/// cost (CR 702.171a) and crew's (CR 702.122a): whether a choice of creatures adds up to enough power.
/// <para>
/// In <c>Cantrip.Core</c> for the reason <see cref="CardDistribution"/> is: the engine's prompt refuses a choice that
/// falls short, so a bad answer is re-asked rather than fatal, and the client greys its Done button by the same code.
/// The powers arrive index for index with the cards that may be chosen, read once as the question is asked, so neither
/// side needs to know what a card is.
/// </para>
/// </summary>
public static class TotalPower
{
    /// <summary>
    /// The total power of <paramref name="chosen"/>, each card counted once, read from <paramref name="powers"/> at its
    /// index in <paramref name="eligible"/>. A card that is not eligible adds nothing.
    /// </summary>
    public static int Of<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> powers, IEnumerable<T> chosen)
        where T : notnull
    {
        ArgumentNullException.ThrowIfNull(eligible);
        ArgumentNullException.ThrowIfNull(powers);
        ArgumentNullException.ThrowIfNull(chosen);

        int total = 0;
        HashSet<T> seen = [];

        foreach (T card in chosen)
        {
            if (!seen.Add(card))
            {
                continue;
            }

            for (int i = 0; i < eligible.Count && i < powers.Count; i++)
            {
                if (EqualityComparer<T>.Default.Equals(eligible[i], card))
                {
                    total += powers[i];
                    break;
                }
            }
        }

        return total;
    }

    /// <summary>
    /// Whether <paramref name="chosen"/> has total power <paramref name="minimum"/> or greater. A minimum of zero or less
    /// asks nothing, so any choice meets it.
    /// </summary>
    public static bool Meets<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> powers, IEnumerable<T> chosen, int minimum)
        where T : notnull =>
        minimum <= 0 || Of(eligible, powers, chosen) >= minimum;
}
