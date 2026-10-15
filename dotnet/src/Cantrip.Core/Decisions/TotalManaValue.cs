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
}
