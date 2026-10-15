namespace Cantrip.Client.Drafting;

/// <summary>
/// What a drafter remembers of a booster draft beyond its own pool: every pack it was shown, what it took from each
/// and so what it passed on, and - when a pack comes round the table again - which of its cards did not come back.
/// From those it keeps an estimate, colour by colour, of what is <b>open</b>: how much better or worse than usual the
/// cards of that colour reaching this seat are going to be.
/// <para>
/// A pack is read as how much each colour has in it worth playing - every card's worth above a middling one's, a gold
/// card's split between its colours - set against the average of the five (<see cref="Reading"/>): a pack with two
/// strong blue cards and nothing red reads blue up and red down, whatever pick it is. Three things are kept:
/// </para>
/// <list type="bullet">
/// <item><b>What is passed to this seat</b> (<see cref="Flow"/>). A pack that has been through four players and is
/// still rich in blue was passed by four players who did not want it. It speaks for the players the pack came
/// through: for the rest of this pack, and for the next pack that comes the same way round.</item>
/// <item><b>What this seat passes</b> (<see cref="Passed"/>). The players it passes to are the ones who will be passing
/// to it when the direction turns, and they draft what they are given.</item>
/// <item><b>What does not wheel</b> (<see cref="Contested"/>). Between two sightings of a pack every other player has
/// taken a card from it. The good cards going is no news; a card taken over better ones is somebody drafting its
/// colour, and counts against that colour everywhere.</item>
/// </list>
/// <para>
/// How much each is worth as a forecast is not a matter of opinion: <c>cantrip-botlab draft calibrate</c> drafts
/// hundreds of pods and fits what a colour went on to deliver to what had been read of it, and those slopes are the
/// drafter's weights (<see cref="DraftTuning"/>).
/// </para>
/// </summary>
public sealed class DraftMemory
{
    /// <summary>One look at a pack: which pack round and pick, what was in it, and what this seat took.</summary>
    public sealed class Sight
    {
        public int Round { get; init; }

        public int Pick { get; init; }

        public required IReadOnlyList<DraftCard> Cards { get; init; }

        /// <summary>The places in the pack of what this seat took: by place, since a cube may hold two of a card.</summary>
        public List<int> TakenPlaces { get; } = [];

        public IEnumerable<DraftCard> Taken => TakenPlaces.Select(place => Cards[place]);

        /// <summary>What went on to the next seat.</summary>
        public IEnumerable<DraftCard> Passed => Cards.Where((_, place) => !TakenPlaces.Contains(place));

        /// <summary>The earlier look at this same pack, when it has come round again.</summary>
        public Sight? Before { get; init; }
    }

    private const int Colors = 5;

    /// <summary>What a colour with no card in the pack counts as: about what the last few cards of a pack are worth.</summary>
    private const double Nothing = 1.5;

    /// <summary>How many looks' worth of "nothing unusual" an estimate starts with, so that one pack does not make a signal.</summary>
    public const double Doubt = 3;

    /// <summary>What a card has to be worth to count towards what a pack holds of its colour: a middling card.</summary>
    private const double Playable = CardWorth.Middling;

    private readonly List<Sight> _sights = [];

    /// <summary>For each round (from 1): the sum, colour by colour, of the readings of the packs passed to this seat, and how many.</summary>
    private readonly Dictionary<int, (double[] Sum, int Count)> _flow = [];

    /// <summary>The same for the packs this seat passed on.</summary>
    private readonly Dictionary<int, (double[] Sum, int Count)> _passed = [];

    /// <summary>For each colour, the cards of it taken over better cards between two sightings of a pack; and how many packs came round.</summary>
    private readonly double[] _contested = new double[Colors];
    private int _wheels;

    public IReadOnlyList<Sight> Sights => _sights;

    /// <summary>Every card known to have been taken by somebody else: in a pack this seat passed, and gone when it came back.</summary>
    public List<DraftCard> TakenByOthers { get; } = [];

    /// <summary>
    /// The best card of one colour a pack of fifteen usually still holds with this many cards left in it, at a table of
    /// eight - measured (<c>cantrip-botlab draft calibrate</c>), not guessed. Index 0 is an empty pack.
    /// </summary>
    internal static readonly double[] UsualBestOfColor =
        [1.50, 1.51, 1.56, 1.64, 1.74, 1.87, 2.00, 2.14, 2.29, 2.44, 2.59, 2.74, 2.89, 3.06, 3.24, 3.39];

    /// <summary>The usual best card of a colour in a pack with this many of <paramref name="packSize"/> cards left.</summary>
    public static double Expected(int left, int packSize) => Interpolate(UsualBestOfColor, left, packSize);

    internal static double Interpolate(double[] table, int left, int packSize)
    {
        double at = packSize <= 0 ? 0 : Math.Clamp(left, 0, packSize) * (table.Length - 1.0) / packSize;
        int below = (int)Math.Floor(at);
        int above = Math.Min(below + 1, table.Length - 1);
        return table[below] + ((table[above] - table[below]) * (at - below));
    }

    /// <summary>
    /// What a pack holds of each colour, set against the average colour: the worth above a middling card's of every
    /// card of the colour, a gold card's split among its colours, less the mean of the five. Zero all round for a pack
    /// with nothing worth playing, or with the same of everything.
    /// </summary>
    public static double[] Reading(IEnumerable<DraftCard> cards)
    {
        ArgumentNullException.ThrowIfNull(cards);
        double[] holds = new double[Colors];
        foreach (DraftCard card in cards)
        {
            if (card.IsLand || card.Colors == 0 || card.Power <= Playable)
            {
                continue;
            }

            double share = (card.Power - Playable) / int.PopCount(card.Colors);
            for (int color = 0; color < Colors; color++)
            {
                if (((card.Colors >> color) & 1) != 0)
                {
                    holds[color] += share;
                }
            }
        }

        double mean = holds.Average();
        for (int color = 0; color < Colors; color++)
        {
            holds[color] -= mean;
        }

        return holds;
    }

    /// <summary>The best card of each colour among these, a gold card counting for each of its colours at a discount.</summary>
    public static double[] BestOfEachColor(IEnumerable<DraftCard> cards)
    {
        ArgumentNullException.ThrowIfNull(cards);
        double[] best = [Nothing, Nothing, Nothing, Nothing, Nothing];
        foreach (DraftCard card in cards)
        {
            if (card.IsLand || card.Colors == 0)
            {
                continue;
            }

            double worth = card.Power - (int.PopCount(card.Colors) > 1 ? 0.7 : 0);
            for (int color = 0; color < Colors; color++)
            {
                if (((card.Colors >> color) & 1) != 0)
                {
                    best[color] = Math.Max(best[color], worth);
                }
            }
        }

        return best;
    }

    /// <summary>
    /// A pack is in front of this seat. Looked at once: a second call for the same pick of the same round changes nothing.
    /// </summary>
    /// <returns>The look, to say what was taken from it (<see cref="Took"/>).</returns>
    public Sight See(int round, int pick, int packSize, IReadOnlyList<DraftCard> cards)
    {
        ArgumentNullException.ThrowIfNull(cards);
        if (_sights.LastOrDefault() is { } last && last.Round == round && last.Pick == pick && last.Cards.Count == cards.Count)
        {
            return last;
        }

        // The same pack again: the earlier look this round whose passed cards these all are - but for one, which a
        // Cogwork Librarian may have put there since. One card in common at the least: a last card is still a pack.
        HashSet<string> names = [.. cards.Select(card => card.Name)];
        Sight? before = cards.Count == 0 ? null : _sights.LastOrDefault(sight => sight.Round == round && sight.Cards.Count > cards.Count
            && sight.Passed.Count(card => names.Contains(card.Name)) >= Math.Max(1, cards.Count - 1));

        var seen = new Sight { Round = round, Pick = pick, Cards = cards, Before = before };
        _sights.Add(seen);

        // What was passed to this seat. The first pick of a pack nobody has passed, and its last cards say nothing.
        if (pick > 1 && cards.Count >= 3)
        {
            Add(_flow, round, Reading(cards));
        }

        if (before is not null)
        {
            // Everyone else took one card each from what this seat passed. Had they gone by worth alone, the best
            // of it would be what is missing; a lesser card missing instead was taken for its colour.
            List<DraftCard> passed = [.. before.Passed];
            List<DraftCard> gone = [.. passed.Where(card => !names.Contains(card.Name))];
            HashSet<DraftCard> best = [.. passed.OrderByDescending(card => card.Power).Take(gone.Count)];
            TakenByOthers.AddRange(gone);
            _wheels++;
            foreach (DraftCard card in gone.Where(card => !best.Contains(card) && !card.IsLand && card.Colors != 0))
            {
                double share = 1.0 / int.PopCount(card.Colors);
                for (int color = 0; color < Colors; color++)
                {
                    if (((card.Colors >> color) & 1) != 0)
                    {
                        _contested[color] += share;
                    }
                }
            }
        }

        return seen;
    }

    /// <summary>This seat took cards from the pack it was looking at, by their places in it; the rest goes on.</summary>
    public void Took(Sight sight, IEnumerable<int> places)
    {
        ArgumentNullException.ThrowIfNull(sight);
        ArgumentNullException.ThrowIfNull(places);
        if (sight.TakenPlaces.Count > 0)
        {
            return;
        }

        sight.TakenPlaces.AddRange(places);
        List<DraftCard> passed = [.. sight.Passed];
        if (passed.Count >= 3)
        {
            Add(_passed, sight.Round, Reading(passed));
        }
    }

    private static void Add(Dictionary<int, (double[] Sum, int Count)> into, int round, double[] reading)
    {
        (double[] sum, int count) = into.TryGetValue(round, out (double[] Sum, int Count) kept) ? kept : (new double[Colors], 0);
        for (int color = 0; color < Colors; color++)
        {
            sum[color] += reading[color];
        }

        into[round] = (sum, count + 1);
    }

    private static double Mean(Dictionary<int, (double[] Sum, int Count)> of, int round, int color) =>
        of.TryGetValue(round, out (double[] Sum, int Count) kept) ? kept.Sum[color] / (kept.Count + Doubt) : 0;

    /// <summary>
    /// How much more of a colour than of the average colour the packs passed to this seat in a round have held, a pack:
    /// held towards nothing while few have been seen. Zero with none.
    /// </summary>
    public double Flow(int color, int round) => Mean(_flow, round, color);

    /// <summary>The same of the packs this seat passed on in a round.</summary>
    public double Passed(int color, int round) => Mean(_passed, round, color);

    /// <summary>
    /// How many more cards of a colour than of the average colour, for each pack that came round, were taken over
    /// better cards.
    /// </summary>
    public double Contested(int color) => (_contested[color] - _contested.Average()) / (_wheels + 2.0);

    /// <summary>
    /// How open a colour is expected to be in a round, in worth a pick above or below the usual: what has been passed
    /// to this seat that round, at <paramref name="signals"/>, and two rounds before, when packs came the same way, at
    /// <paramref name="carry"/>; less <paramref name="cut"/> of what this seat sent downstream the round before, to the
    /// players now upstream; less <paramref name="crowd"/> for each card of the colour the table has been seen taking
    /// over better ones.
    /// </summary>
    public double Open(int color, int round, double signals, double carry, double cut, double crowd) =>
        (signals * Flow(color, round)) + (carry * Flow(color, round - 2)) - (cut * Passed(color, round - 1)) - (crowd * Contested(color));
}
