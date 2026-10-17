namespace Cantrip.Core.Decisions;

/// <summary>
/// The rule behind "exile any number of cards from your graveyard with total mana value N or greater" — collect
/// evidence's cost (CR 701.59a) — and any effect that asks for a choice of cards whose mana values add up to enough:
/// the twin of <see cref="TotalPower"/>, beside it for the same reason. The engine's prompt refuses a choice that falls
/// short, so a bad answer is re-asked rather than fatal, and the client greys its Done button by the same code.
/// <para>
/// The mana values arrive index for index with the cards that may be chosen, read once as the question is asked, so
/// neither side needs to know what a card is.
/// </para>
/// </summary>
public static class TotalManaValue
{
    /// <summary>
    /// The total mana value of <paramref name="chosen"/>, each card counted once, read from <paramref name="manaValues"/>
    /// at its index in <paramref name="eligible"/>. A card that is not eligible adds nothing.
    /// </summary>
    public static int Of<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> manaValues, IEnumerable<T> chosen)
        where T : notnull =>
        TotalPower.Of(eligible, manaValues, chosen);

    /// <summary>
    /// Whether <paramref name="chosen"/> has total mana value <paramref name="minimum"/> or greater. A minimum of zero or
    /// less asks nothing, so any choice meets it.
    /// </summary>
    public static bool Meets<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> manaValues, IEnumerable<T> chosen, int minimum)
        where T : notnull =>
        minimum <= 0 || Of(eligible, manaValues, chosen) >= minimum;

    /// <summary>
    /// Whether <paramref name="chosen"/> has total mana value <paramref name="most"/> or less ("up to two creature cards with
    /// total mana value 6 or less", CR 202.3). No limit, null, allows any choice.
    /// </summary>
    public static bool Within<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> manaValues, IEnumerable<T> chosen, int? most)
        where T : notnull =>
        most is not { } limit || Of(eligible, manaValues, chosen) <= limit;

    /// <summary>
    /// Whether <paramref name="card"/> can be added to <paramref name="chosen"/> and stay within <paramref name="most"/>: a card
    /// already chosen always can (picking it again puts it back).
    /// </summary>
    public static bool Fits<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> manaValues, IReadOnlyCollection<T> chosen, T card, int? most)
        where T : notnull =>
        chosen.Contains(card) || Within(eligible, manaValues, chosen.Append(card), most);

    /// <summary>
    /// The first of <paramref name="eligible"/>, in order, that fit together within <paramref name="most"/>, at most
    /// <paramref name="count"/> of them: an answer always accepted, for a player who answers without reading the cards.
    /// </summary>
    public static List<T> FirstFitting<T>(IReadOnlyList<T> eligible, IReadOnlyList<int> manaValues, int count, int most)
        where T : notnull
    {
        ArgumentNullException.ThrowIfNull(eligible);
        ArgumentNullException.ThrowIfNull(manaValues);
        List<T> taken = [];
        int left = most;
        for (int index = 0; index < eligible.Count && taken.Count < count; index++)
        {
            int value = index < manaValues.Count ? manaValues[index] : 0;
            if (value <= left)
            {
                left -= value;
                taken.Add(eligible[index]);
            }
        }

        return taken;
    }
}
