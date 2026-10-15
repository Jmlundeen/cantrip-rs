namespace Cantrip.Client.Drafting;

/// <summary>
/// What a deck's lands have to be for its spells to be cast: how likely a cost is to be paid on time with so many
/// sources of its colour, and how many lands a curve wants.
/// <para>
/// The first is counted, not looked up: with so many sources in forty cards, the chance of having drawn as many as
/// the cost has symbols by the turn it could first be cast, and three turns later. It is what Frank Karsten's tables
/// of sources are made from - his "ten sources for {W} on turn one" is this sum at nine chances in ten - kept as the
/// chance itself rather than a threshold, so that the third source of a splash is seen to matter more than the
/// fourteenth of a main colour. The second is his regression on winning decks.
/// </para>
/// </summary>
public static class ManaBase
{
    /// <summary>Lands in a forty-card deck, before the curve has its say.</summary>
    public const int Lands = 17;

    private const int Deck = 40;

    /// <summary>The chance of at least one, two and three of so many sources among so many of a deck's forty cards.</summary>
    private static readonly double[,,] AtLeast = Counted();

    private static double[,,] Counted()
    {
        // Pascal's triangle, far enough for forty choose twenty.
        double[,] choose = new double[Deck + 1, Deck + 1];
        for (int n = 0; n <= Deck; n++)
        {
            choose[n, 0] = 1;
            for (int k = 1; k <= n; k++)
            {
                choose[n, k] = choose[n - 1, k - 1] + (k <= n - 1 ? choose[n - 1, k] : 0);
            }
        }

        double[,,] atLeast = new double[Deck + 1, Deck + 1, 3];
        for (int sources = 0; sources <= Deck; sources++)
        {
            for (int seen = 0; seen <= Deck; seen++)
            {
                double fewer = 0;
                for (int pips = 1; pips <= 3; pips++)
                {
                    int exactly = pips - 1;
                    if (exactly <= sources && seen - exactly >= 0 && seen - exactly <= Deck - sources)
                    {
                        fewer += choose[sources, exactly] * choose[Deck - sources, seen - exactly] / choose[Deck, seen];
                    }

                    atLeast[sources, seen, pips - 1] = Math.Clamp(1 - fewer, 0, 1);
                }
            }
        }

        return atLeast;
    }

    /// <summary>
    /// The chance that, of a forty-card deck with this many sources of a colour, the cards seen so far hold at least as
    /// many of them as a cost has symbols. Sources and cards seen may be fractions - a mana creature is half a
    /// source, and a turn is seven and a half cards on average - and are read between the whole numbers.
    /// </summary>
    public static double Odds(double sources, double seen, int pips)
    {
        if (pips <= 0)
        {
            return 1;
        }

        pips = Math.Min(pips, 3);
        sources = Math.Clamp(sources, 0, Deck);
        seen = Math.Clamp(seen, 0, Deck);
        int s = Math.Min((int)sources, Deck - 1);
        int n = Math.Min((int)seen, Deck - 1);
        double sf = sources - s;
        double nf = seen - n;
        return (AtLeast[s, n, pips - 1] * (1 - sf) * (1 - nf)) + (AtLeast[s + 1, n, pips - 1] * sf * (1 - nf))
            + (AtLeast[s, n + 1, pips - 1] * (1 - sf) * nf) + (AtLeast[s + 1, n + 1, pips - 1] * sf * nf);
    }

    /// <summary>
    /// How much of a card is left when its colour has only these sources (of forty cards): the chance its coloured
    /// symbols are paid for on the turn its mana value says it could be cast, and - counting for less - three turns
    /// after. A {3}{R} splashed on three Mountains is worth about two thirds of itself; a {W}{W} two-drop on nine
    /// Plains the same; either on fourteen, nearly all.
    /// </summary>
    public static double Castable(double sources, int manaValue, int pips)
    {
        if (pips <= 0)
        {
            return 1;
        }

        // Asked a great many times of the same few numbers: whole and half sources are kept.
        double doubled = sources * 2;
        int half = (int)doubled;
        if (half == doubled && half is >= 0 and <= Deck * 2)
        {
            return Kept[half, Math.Min(manaValue, MostManaValue), Math.Min(pips, 3) - 1];
        }

        return Counting(sources, manaValue, pips);
    }

    private const int MostManaValue = 16;

    private static readonly double[,,] Kept = Keep();

    private static double[,,] Keep()
    {
        double[,,] kept = new double[(Deck * 2) + 1, MostManaValue + 1, 3];
        for (int half = 0; half <= Deck * 2; half++)
        {
            for (int manaValue = 0; manaValue <= MostManaValue; manaValue++)
            {
                for (int pips = 1; pips <= 3; pips++)
                {
                    kept[half, manaValue, pips - 1] = Counting(half / 2.0, manaValue, pips);
                }
            }
        }

        return kept;
    }

    private static double Counting(double sources, int manaValue, int pips)
    {
        // Seven cards and one a turn, less the half a card the player on the play is behind.
        double onCurve = 6.5 + Math.Max(Math.Max(Math.Min(manaValue, MostManaValue), pips), 1);
        return (0.6 * Odds(sources, onCurve, pips)) + (0.4 * Odds(sources, onCurve + 3, pips));
    }

    /// <summary>How much of a card is left in a deck with these sources of each colour: the least castable of its colours'.</summary>
    /// <param name="sources">Sources of each colour, white first.</param>
    /// <param name="deck">The deck's colours, splash included, as a mask.</param>
    /// <param name="size">How many cards the deck has: the sources are counted as so many in forty.</param>
    public static double Castable(DraftCard card, IReadOnlyList<double> sources, int deck, int size = Deck)
    {
        ArgumentNullException.ThrowIfNull(card);
        ArgumentNullException.ThrowIfNull(sources);
        if (card.OffColor(deck) > 0)
        {
            return 0;
        }

        double worst = 1;
        for (int color = 0; color < 5; color++)
        {
            int pips = card.Pips(color, deck);
            if (pips > 0)
            {
                worst = Math.Min(worst, Castable(sources[color] * Deck / Math.Max(size, 1), card.ManaValue, pips));
            }
        }

        return worst;
    }

    /// <summary>
    /// How many lands a deck of <paramref name="size"/> cards wants: Karsten's count for the curve - two thirds of
    /// 19.59 + 1.9 times the average mana value, in forty cards - less a little for each cheap card that makes mana or
    /// draws, and a land less for each free mana rock, which is a land in all but name. Between fifteen and eighteen
    /// in forty, as nearly every deck that wins is.
    /// </summary>
    public static int LandsFor(int size, double averageManaValue, int cheapManaOrDraw, int freeMana)
    {
        double forty = (2.0 / 3.0 * (19.59 + (1.9 * averageManaValue))) - (0.28 * cheapManaOrDraw) - freeMana;
        double scale = size / 40.0;
        return (int)Math.Round(Math.Clamp(forty, 15, 18) * scale);
    }
}
