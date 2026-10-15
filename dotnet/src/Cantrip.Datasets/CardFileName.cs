using System.Text;

namespace Cantrip.Datasets;

/// <summary>
/// The file a card is defined in: its name, lowercased, accents folded, apostrophes dropped, and every other run of
/// non-alphanumerics a single hyphen. <c>Kolaghan's Command</c> is <c>kolaghans-command.json</c>;
/// <c>Lórien Revealed</c> is <c>lorien-revealed.json</c>.
/// <para>
/// Enforced rather than suggested, so "is this card defined?" is a file lookup, and a second definition of a card
/// cannot hide in the same dataset under another file name.
/// </para>
/// <para>
/// Accents are folded from a table rather than by Unicode decomposition, because the engine runs with invariant
/// globalization, where normalisation is not guaranteed to decompose anything — and a file name must not depend on
/// the host's ICU.
/// </para>
/// </summary>
public static class CardFileName
{
    public const string Extension = ".json";

    private const string Accented = "àáâãäåāăąçćčďèéêëēęěìíîïīñńňòóôõöøōőùúûüūůűýÿžźżß";
    private const string Folded = "aaaaaaaaacccdeeeeeeeiiiiinnnoooooooouuuuuuuyyzzzs";

    public static string For(string cardName)
    {
        ArgumentException.ThrowIfNullOrWhiteSpace(cardName);

        var slug = new StringBuilder(cardName.Length);
        bool pendingHyphen = false;

        foreach (char raw in cardName)
        {
            char c = char.ToLowerInvariant(raw);

            if (c is '\'' or '’')
            {
                continue;
            }

            int accented = Accented.IndexOf(c, StringComparison.Ordinal);
            if (accented >= 0)
            {
                c = Folded[accented];
            }

            if (char.IsAsciiLetterOrDigit(c))
            {
                if (pendingHyphen && slug.Length > 0)
                {
                    slug.Append('-');
                }

                slug.Append(c);
                pendingHyphen = false;
            }
            else
            {
                pendingHyphen = true;
            }
        }

        return slug.ToString() + Extension;
    }
}
