using Cantrip.Protocol;

namespace Cantrip.Client.Drafting;

/// <summary>The numbers a <see cref="CubeDrafter"/> drafts by, each one something that can be set against another in play (<c>cantrip-botlab draft</c>).</summary>
public sealed record DraftTuning
{
    /// <summary>
    /// How sure of its colours the drafter is for what its pool says: the higher, the sooner a pool that favours a
    /// colour pair is drafted as that pair. It rises by itself as the draft runs out of picks to change course with.
    /// </summary>
    public double Sureness { get; init; } = 1.0;

    /// <summary>
    /// What a colour's flow this round (<see cref="DraftMemory.Flow"/>) says of its picks still to come this round, in
    /// worth a pick. This and the three after it are measured, not chosen: <c>cantrip-botlab draft calibrate</c> fits
    /// what a colour went on to deliver to what had been read of it, over hundreds of pods of this drafter. Little:
    /// by the time a pack has said anything, what is left of it is its last and least picks.
    /// </summary>
    public double Signals { get; init; } = 0.12;

    /// <summary>
    /// What a round's flow says of the round after next, when packs come the same way. The reading worth most: a whole
    /// pack of looks at what the same players upstream did not take.
    /// </summary>
    public double Carry { get; init; } = 0.55;

    /// <summary>
    /// What was sent downstream in a round (<see cref="DraftMemory.Passed"/>) takes from the next, when the direction
    /// turns. Nothing, as measured: among these drafters what a seat passes does not show in what comes back to it.
    /// The drafter remembers what it passed all the same - it is how a wheel is read - and the weight is here to be
    /// set against another.
    /// </summary>
    public double Cut { get; init; }

    /// <summary>What each card of a colour seen taken over better ones (<see cref="DraftMemory.Contested"/>) costs that colour.</summary>
    public double Crowd { get; init; } = 0.2;

    /// <summary>What a land that makes two of a deck's colours adds to it, for each point of worth it has over a poor card.</summary>
    public double Fixing { get; init; } = 0.5;

    /// <summary>What a card's worth counts for by itself, whatever the pool: what settles a pick between cards no deck of this pool wants.</summary>
    public double Raw { get; init; } = 0.03;

    /// <summary>How many spells the deck being drafted will hold.</summary>
    public int Slots { get; init; } = 23;

    /// <summary>
    /// How many cards worth cheating into play (<see cref="DraftCard.Payload"/>) a whole draft still to come is
    /// counted on to bring a deck of any colours: what a card that cheats is worth while the pool has none yet.
    /// </summary>
    public double PayloadHope { get; init; } = 2;

    /// <summary>
    /// How much more than one card from the face-down stack a Winston pile has to add to the deck to be taken, in
    /// worth: the pile passed is one card bigger for the next player, and the stack is what is left at the end.
    /// </summary>
    public double WinstonMargin { get; init; }

    /// <summary>
    /// Whether passing a Winston pile that is not the last is weighed as the next pile - its cards and the one the pass
    /// adds, each a card at random - rather than as one card from the stack, which is what passing the last pile is.
    /// </summary>
    public double WinstonNext { get; init; } = 1;

    /// <summary>
    /// On the first pick of a grid, what each point of worth above a middling card the other player could take from the
    /// lines left costs a line: the best line left to them, as they would see it by worth alone. Nothing: as shipped
    /// the drafter takes what is best for itself.
    /// </summary>
    public double Deny { get; init; }
}

/// <summary>A pick, and what the drafter made of the pack: for a log, a test, or a person asking why.</summary>
public sealed class DraftChoice
{
    /// <summary>The places in the pack of the cards to take, the pick first.</summary>
    public required IReadOnlyList<int> Places { get; init; }

    /// <summary>
    /// What the drafter made of the pick, in words a player can read, a line at a time: empty unless it was asked to
    /// explain (<see cref="CubeDrafter.Choose(DraftMessage, int, bool)"/>), which changes nothing about the pick.
    /// </summary>
    public IReadOnlyList<string> Why { get; init; } = [];

    /// <summary>What each card of the pack was scored, in the pack's order: what the pool would be worth with it, over what it is worth without.</summary>
    public required IReadOnlyList<double> Scores { get; init; }

    /// <summary>How likely the drafter thought each colour combination (<see cref="DeckBuilder.Combinations"/>) to be its deck, before the pick.</summary>
    public required IReadOnlyList<double> Beliefs { get; init; }
}

/// <summary>
/// Drafts a cube from booster packs: the card that most improves the deck this pool is likely to become.
/// <para>
/// It keeps no colours and follows no rule about when to commit. It carries fifteen hypotheses - the deck is each
/// colour alone, or each pair - and for each works out what the deck would be worth at the end of the draft
/// (<see cref="Project"/>): the best of the cards it has that those colours cast, as far as two colours' lands cast
/// them, a third colour's best card or two splashed, and - for the picks still to come - the cards those colours can
/// expect to be passed, which is where the signals come in (<see cref="DraftMemory"/>). A pool is worth the best of
/// its hypotheses, softly: early on, when nothing is settled, a card that would be good in several of them is worth
/// more than one that is good in one; once one hypothesis is far ahead only cards that deck would play are worth
/// anything, and how much they are worth is how much better they are than what they would replace.
/// </para>
/// <para>
/// That one sum gives the behaviour drafters describe as rules. The first picks go by raw power, since every pool
/// expects to fill its slots with good cards and only a better one adds anything. A gold card is taken later than a
/// mono-coloured one of the same power, and a colourless one earlier. Two strong cards of a colour are a lean and
/// ten are a commitment. The twenty-third playable is worth taking and the twenty-eighth is not. A fifth six-drop
/// is worth less than a second two-drop. A colour the packs were rich in all through pack one is counted on in pack
/// three, and one the table was seen fighting over is not.
/// </para>
/// <para>
/// Nothing here knows a card by name: how good a card is comes from the cube's pick order where the server sends it,
/// and from reading the card where it does not (<see cref="CardWorth"/>), so a card nobody has ranked is drafted
/// for what its text says it is.
/// </para>
/// </summary>
public sealed partial class CubeDrafter
{
    /// <summary>
    /// The best card a deck of one colour, and of two, can usually still find in a pack of fifteen with this many cards
    /// left in it at a table of eight - colourless cards counted, which every deck can play. Measured
    /// (<c>cantrip-botlab draft calibrate</c>). Index 0 is an empty pack.
    /// </summary>
    internal static readonly double[][] UsualBest =
    [
        [1.00, 1.02, 1.18, 1.39, 1.60, 1.81, 2.00, 2.17, 2.33, 2.50, 2.67, 2.85, 3.04, 3.26, 3.56, 4.24],
        [1.00, 1.05, 1.30, 1.59, 1.86, 2.10, 2.33, 2.52, 2.71, 2.89, 3.07, 3.25, 3.44, 3.66, 3.95, 4.55],
    ];

    /// <summary>More than any one card adds to a deck.</summary>
    private const double OneCard = 9;

    /// <summary>Sources of each colour a two-colour deck has before any land of the pool's adds to them: half of seventeen.</summary>
    private const double PairSources = 8.5;

    private readonly Dictionary<string, DraftCard> _read = new(StringComparer.Ordinal);
    private readonly Dictionary<string, DraftCard> _ranked = new(StringComparer.Ordinal);
    private readonly Dictionary<string, int> _rankOf = new(StringComparer.Ordinal);
    private readonly Dictionary<int, string> _holder = [];

    /// <summary>The rank the server gives every card its pick order does not name: found when two different cards share one.</summary>
    private int _unranked = -1;
    private int _ranks;

    public DraftTuning Tuning { get; init; } = new();

    public DeckTuning Deck { get; init; } = new();

    public DraftMemory Memory { get; } = new();

    /// <summary>
    /// A card, read, and worth what its place in the pick order says where the drafter has been told one - in any pack
    /// it has looked at - and what the card itself says where it has not.
    /// </summary>
    public DraftCard Card(ObjectView view)
    {
        ArgumentNullException.ThrowIfNull(view);
        if (_ranked.TryGetValue(view.Name, out DraftCard? ranked))
        {
            return ranked;
        }

        if (!_read.TryGetValue(view.Name, out DraftCard? card))
        {
            _read[view.Name] = card = DraftCard.Read(view);
        }

        if (_rankOf.TryGetValue(view.Name, out int rank) && rank != _unranked)
        {
            return _ranked[view.Name] = card.WithWorth(CardWorth.FromRank(rank, _ranks));
        }

        return card;
    }

    /// <summary>
    /// Takes in where cards stand in the cube's pick order (<see cref="DraftMessage.PileRanks"/>), which is all the
    /// drafter is ever told of it: a pack at a time, as the packs are seen.
    /// </summary>
    /// <param name="total">How many cards the draft was dealt (<see cref="DraftMessage.Total"/>).</param>
    public void Learn(IReadOnlyList<ObjectView> pack, IReadOnlyList<int> ranks, int total)
    {
        ArgumentNullException.ThrowIfNull(pack);
        ArgumentNullException.ThrowIfNull(ranks);
        if (ranks.Count != pack.Count || pack.Count == 0)
        {
            return;
        }

        int before = _ranks;
        for (int place = 0; place < pack.Count; place++)
        {
            // A grid's taken places are cards nobody is shown.
            string name = pack[place].Name;
            if (!pack[place].Known || name.Length == 0)
            {
                continue;
            }

            _rankOf[name] = ranks[place];
            if (_holder.TryGetValue(ranks[place], out string? holder) && !string.Equals(holder, name, StringComparison.Ordinal) && _unranked != ranks[place])
            {
                _unranked = ranks[place];
                _ranked.Clear();
            }

            _holder[ranks[place]] = name;
        }

        // The highest rank of a card shown: a grid's taken places carry one past the order's end, and are no card.
        int highest = Enumerable.Range(0, pack.Count).Where(place => pack[place].Known && pack[place].Name.Length > 0).Select(place => ranks[place]).DefaultIfEmpty(-1).Max();
        _ranks = Math.Max(Math.Max(_ranks, total), highest + 1);
        if (_unranked >= 0)
        {
            _ranks = Math.Min(_ranks, _unranked);
        }

        if (_ranks != before)
        {
            _ranked.Clear();
        }
    }

    /// <summary>
    /// Chooses from the pack in front of this seat in a booster draft, and remembers the pack and what was taken.
    /// </summary>
    /// <param name="count">How many cards to take: more than one for a seat spending a Cogwork Librarian.</param>
    /// <param name="explain">
    /// Whether to say why (<see cref="DraftChoice.Why"/>): worked out once the pick is made, from what it was made by,
    /// and never a part of it.
    /// </param>
    public DraftChoice Choose(DraftMessage draft, int count = 1, bool explain = false)
    {
        ArgumentNullException.ThrowIfNull(draft);
        Learn(draft.Pile, draft.PileRanks, draft.Total);

        List<DraftCard> pack = [.. draft.Pile.Select(Card)];
        List<DraftCard> pool = [.. draft.Pool.Select(Card)];
        int packSize = Math.Max(draft.PackSize, pack.Count);
        int round = Math.Max(1, draft.PackNumber);
        int rounds = Math.Max(round, draft.Packs);
        DraftMemory.Sight sight = Memory.See(round, draft.PickNumber, packSize, pack);

        // The picks still to come after this one: the rest of this round, each from a pack one card smaller, and every
        // later round from the top.
        List<(int Round, int Left)> coming = [.. Enumerable.Range(1, Math.Max(0, pack.Count - 1)).Reverse().Select(left => (round, left))];
        for (int later = round + 1; later <= rounds; later++)
        {
            coming.AddRange(Enumerable.Range(1, packSize).Reverse().Select(left => (later, left)));
        }

        double sureness = Tuning.Sureness / (((double)coming.Count / Math.Max(1, rounds * packSize)) + 0.15);
        double hope = Tuning.PayloadHope * coming.Count / Math.Max(1, rounds * packSize);

        IReadOnlyList<int> combinations = DeckBuilder.Combinations;
        double[][] expected = [.. combinations.Select(colors => Expected(colors, coming, packSize))];
        double[] without = [.. combinations.Select((colors, at) => Project(pool, null, colors, expected[at], hope))];
        double standing = Soft(without, sureness);

        // A deck so far behind the best that no one card could bring it into the reckoning is left as it stands.
        double best = without.Max();
        bool[] live = [.. without.Select(worth => sureness * (worth + OneCard - best) > -10)];

        double[] scores = new double[pack.Count];
        double[] with = new double[combinations.Count];
        double[][]? kept = explain ? new double[pack.Count][] : null;
        for (int place = 0; place < pack.Count; place++)
        {
            for (int at = 0; at < combinations.Count; at++)
            {
                // A card the deck would be worse for playing is a card it does not play: never less than without it.
                with[at] = live[at] ? Math.Max(without[at], Project(pool, pack[place], combinations[at], expected[at], hope)) : without[at];
            }

            scores[place] = Soft(with, sureness) - standing + (Tuning.Raw * pack[place].Worth);
            if (kept is not null)
            {
                kept[place] = [.. with];
            }
        }

        List<int> places = [.. Enumerable.Range(0, pack.Count).OrderByDescending(place => scores[place]).ThenByDescending(place => pack[place].Worth).ThenBy(place => place).Take(Math.Max(1, count))];
        Memory.Took(sight, places);

        double[] beliefs = [.. without.Select(worth => Math.Exp(sureness * (worth - best)))];
        double all = beliefs.Sum();
        beliefs = [.. beliefs.Select(belief => belief / all)];
        return new DraftChoice
        {
            Places = places,
            Scores = scores,
            Beliefs = beliefs,
            Why = kept is null ? [] : Explaining(() => ExplainPick(new Pick(draft, pack, pool, places, scores, beliefs, without, kept, expected, hope, sight))),
        };
    }

    /// <summary>
    /// Chooses a row (0 to 2) or column (3 to 5) of a grid draft's grid: the line the pool would be worth most with,
    /// weighed as a booster pick is (<see cref="Choose"/>) but a whole line at a time, against the grids still to come.
    /// </summary>
    /// <param name="why">Told, if given, what each line came to and which was taken: never a part of the choice.</param>
    public int ChooseLine(DraftMessage grid, List<string>? why = null)
    {
        ArgumentNullException.ThrowIfNull(grid);
        Learn(grid.Pile, grid.PileRanks, grid.Total);
        List<DraftCard> pool = [.. grid.Pool.Select(Card)];

        // The grids to come: the seat picks first from every other one, three cards from the best line of nine, and
        // second from the rest, two or three from what is left. Each card to come is weighed as the best a booster
        // pack of that many still holds - nine, six, three - which is what a line of a grid amounts to.
        List<(int Round, int Left)> coming = [];
        bool first = grid.PickNumber == 2;
        for (int later = grid.PackNumber + 1; later <= grid.Packs; later++, first = !first)
        {
            coming.AddRange(first ? [(1, 9), (1, 6), (1, 3)] : [(1, 6), (1, 3)]);
        }

        Weighing weighing = Weigh(pool, coming, 15, Math.Max(1, grid.Packs * 5 / 2));
        List<int> lines = [.. Enumerable.Range(0, 6).Where(line => GridPlaces(line).Any(place => place < grid.Pile.Count && grid.Pile[place].Known))];
        if (lines.Count == 0)
        {
            return 0;
        }

        bool Known(int place) => place < grid.Pile.Count && grid.Pile[place].Known;

        // What the other player gets from the lines left, picking second: the best of them by worth above a middling card.
        double Left(int taken)
        {
            if (Tuning.Deny == 0 || grid.PickNumber != 1)
            {
                return 0;
            }

            HashSet<int> gone = [.. GridPlaces(taken)];
            return Enumerable.Range(0, 6)
                .Where(line => line != taken)
                .Select(line => GridPlaces(line).Where(place => Known(place) && !gone.Contains(place)).Sum(place => Math.Max(0, Card(grid.Pile[place]).Power - CardWorth.Middling)))
                .DefaultIfEmpty(0)
                .Max();
        }

        List<(int Line, double Gain)> weighed = [.. lines
            .Select(line => (Line: line, Gain: Gain(weighing, pool, [.. GridPlaces(line).Where(Known).Select(place => Card(grid.Pile[place]))]) - (Tuning.Deny * Left(line))))];
        int chosen = weighed
            .OrderByDescending(pair => pair.Gain)
            .ThenBy(pair => pair.Line)
            .First().Line;
        why?.AddRange(Explaining(() => ExplainLine(grid, pool, weighing, weighed, chosen, line => [.. GridPlaces(line).Where(Known).Select(place => Card(grid.Pile[place]))], Left)));
        return chosen;
    }

    /// <summary>
    /// Whether to take the Winston pile this seat is looking at rather than pass it: whether the pool would be worth
    /// more with it than with one card from the face-down stack (<see cref="DraftTuning.WinstonMargin"/>). A pile that
    /// cannot be passed is taken.
    /// </summary>
    /// <param name="why">Told, if given, what the pile and passing it came to: never a part of the choice.</param>
    public bool TakePile(DraftMessage winston, List<string>? why = null)
    {
        ArgumentNullException.ThrowIfNull(winston);
        if (!winston.CanPass || winston.Pile.Count == 0)
        {
            why?.AddRange(Explaining(() => ExplainPile(winston, null, 0, 0, 0, true)));
            return true;
        }

        Learn(winston.Pile, winston.PileRanks, winston.Total);
        List<DraftCard> pool = [.. winston.Pool.Select(Card)];

        // The cards to come: this seat's share of the stack and the piles, each the best of a few to choose from.
        int seats = Math.Max(2, winston.Seats.Count);
        int left = Math.Max(0, ((winston.Stack + winston.Piles.Sum()) / seats) - winston.Pile.Count);
        List<(int Round, int Left)> coming = [.. Enumerable.Repeat((1, 3), left)];
        Weighing weighing = Weigh(pool, coming, 15, Math.Max(1, winston.Total / seats));

        // What passing it comes to: the next pile, with the card the pass adds to it, or - past the last pile - one card
        // from the stack. Each card of it one more to come, as good as a pack of one: a card at random.
        int next = Enumerable.Range(winston.Looking + 1, Math.Max(0, winston.Piles.Count - winston.Looking - 1)).FirstOrDefault(at => winston.Piles[at] > 0, -1);
        int instead = Tuning.WinstonNext > 0 && next >= 0 ? winston.Piles[next] + 1 : 1;
        Weighing blind = Weigh(pool, [.. coming, .. Enumerable.Repeat((1, 1), instead)], 15, Math.Max(1, winston.Total / seats), weighing.Hope);

        // Weighed by what each adds to the deck alone, without the pick's tie-break on worth (DraftTuning.Raw), which
        // a card from the stack has no worth to be given. Where neither adds anything the plain way decides: a pile of
        // two or more is taken.
        double stack = Soft(blind.Without, weighing.Sureness) - weighing.Standing;
        double pile = Gain(weighing, pool, [.. winston.Pile.Select(Card)], raw: false);
        bool take = Math.Abs(pile - stack) < 1e-9 ? winston.Pile.Count >= 2 : pile > stack + Tuning.WinstonMargin;
        why?.AddRange(Explaining(() => ExplainPile(winston, weighing, pile, stack, next >= 0 && Tuning.WinstonNext > 0 ? instead : 0, take)));
        return take;
    }

    /// <summary>What a pool is worth now, each way it could go, with these picks still to come: what a set of cards is weighed against.</summary>
    private sealed record Weighing(double[][] Expected, double[] Without, double Standing, double Sureness, double Hope);

    /// <param name="hoped">The cards worth cheating the picks to come are counted on for, where not this many picks' share.</param>
    private Weighing Weigh(List<DraftCard> pool, List<(int Round, int Left)> coming, int packSize, int picks, double? hoped = null)
    {
        double share = (double)coming.Count / Math.Max(1, picks);
        double sureness = Tuning.Sureness / (share + 0.15);
        double hope = hoped ?? Tuning.PayloadHope * Math.Min(1, share);
        coming = [.. coming.OrderByDescending(pick => pick.Left)];
        double[][] expected = [.. DeckBuilder.Combinations.Select(colors => Expected(colors, coming, packSize))];
        double[] without = [.. DeckBuilder.Combinations.Select((colors, at) => Project(pool, null, colors, expected[at], hope))];
        return new Weighing(expected, without, Soft(without, sureness), sureness, hope);
    }

    /// <summary>What a pool gains by several cards at once, on the scale of <see cref="Choose"/>'s scores.</summary>
    private double Gain(Weighing weighing, List<DraftCard> pool, IReadOnlyList<DraftCard> cards, bool raw = true)
    {
        List<DraftCard> with = [.. pool, .. cards];
        double[] worths = new double[weighing.Without.Length];
        for (int at = 0; at < worths.Length; at++)
        {
            worths[at] = Math.Max(weighing.Without[at], Project(with, null, DeckBuilder.Combinations[at], weighing.Expected[at], weighing.Hope));
        }

        return Soft(worths, weighing.Sureness) - weighing.Standing + (raw ? Tuning.Raw * cards.Sum(card => card.Worth) : 0);
    }

    /// <summary>The grid's places in a line: a row, 0 to 2, or a column, 3 to 5.</summary>
    private static IEnumerable<int> GridPlaces(int line) => line < 3
        ? Enumerable.Range(0, 3).Select(column => (line * 3) + column)
        : Enumerable.Range(0, 3).Select(row => (row * 3) + (line - 3));

    /// <summary>Builds the deck for a pool (<see cref="DeckBuilder"/>), each card worth what the draft taught this drafter it is.</summary>
    /// <param name="why">Told, if given, the deck and why it is that one: never a part of the choice.</param>
    public BuiltDeck Build(DeckBuildingMessage building, List<string>? why = null)
    {
        ArgumentNullException.ThrowIfNull(building);
        List<DraftCard> pool = [.. building.Pool.Select(Card)];
        List<BuiltDeck>? tried = why is null ? null : [];
        BuiltDeck deck = DeckBuilder.Build(pool, building.Minimum, Deck, tried);
        why?.AddRange(Explaining(() => ExplainDeck(pool, building.Minimum, deck, tried!)));
        return deck;
    }

    /// <summary>The best of several outcomes, softly: their maximum when one stands clear, more than it when several are close.</summary>
    private static double Soft(double[] worths, double sureness)
    {
        double top = worths.Max();
        double sum = 0;
        foreach (double worth in worths)
        {
            sum += Math.Exp(sureness * (worth - top));
        }

        return top + (Math.Log(sum / worths.Length) / sureness);
    }

    /// <summary>What a deck of these colours can expect from each pick still to come, best first.</summary>
    private double[] Expected(int colors, List<(int Round, int Left)> coming, int packSize)
    {
        bool pair = int.PopCount(colors) >= 2;
        double[] usual = UsualBest[pair ? 1 : 0];

        // A card to come is a card to cast, on the lands a deck of these colours has: the same lands the pool's are cast on.
        double cast = pair ? ManaBase.Castable(PairSources, 3, 1) : 1;
        Dictionary<int, double> open = [];
        double[] expected = new double[coming.Count];
        for (int at = 0; at < coming.Count; at++)
        {
            (int round, int left) = coming[at];
            if (!open.TryGetValue(round, out double signal))
            {
                double sum = 0;
                for (int color = 0; color < 5; color++)
                {
                    if (((colors >> color) & 1) != 0)
                    {
                        sum += Memory.Open(color, round, Tuning.Signals, Tuning.Carry, Tuning.Cut, Tuning.Crowd);
                    }
                }

                open[round] = signal = sum / int.PopCount(colors);
            }

            expected[at] = (DraftMemory.Interpolate(usual, left, packSize) + signal) * cast;
        }

        Array.Sort(expected);
        Array.Reverse(expected);
        return expected;
    }

    /// <summary>
    /// What the deck of these colours would be worth at the end of the draft, from this pool - and one more card, if
    /// given: its best spells and the best of what is still expected, as many as a deck holds; what its lands add; less
    /// what its shape costs.
    /// </summary>
    /// <param name="expected">What each pick to come is expected to bring these colours, best first.</param>
    /// <param name="hope">How many cards worth cheating into play the picks to come are counted on for (<see cref="DraftTuning.PayloadHope"/>).</param>
    public double Project(List<DraftCard> pool, DraftCard? and, int colors, double[] expected, double hope = 0) =>
        Project(pool, and, colors, expected, hope, null);

    /// <param name="parts">Told, if given, what the sum was made of: for saying why, and never a part of it.</param>
    private double Project(List<DraftCard> pool, DraftCard? and, int colors, double[] expected, double hope, Parts? parts)
    {
        bool pair = int.PopCount(colors) >= 2;
        int count = pool.Count + (and is null ? 0 : 1);
        DraftCard At(int index) => index < pool.Count ? pool[index] : and!;

        // The lands first: they say what the spells can count on.
        int duals = 0;
        double landWorth = 0;
        double utilityBest = 0;
        double utilitySecond = 0;
        Span<int> fixers = stackalloc int[5];
        for (int index = 0; index < count; index++)
        {
            DraftCard card = At(index);
            if (!card.IsLand || !card.MakesMana)
            {
                continue;
            }

            int makes = int.PopCount(card.Produces & colors);
            if (pair && makes >= 2)
            {
                duals++;
                if (duals <= 5)
                {
                    landWorth += Tuning.Fixing * Math.Max(0, card.Worth - 2);
                }
            }
            else if (makes == 0 && card.Produces == 0)
            {
                double utility = 0.6 * Math.Max(0, card.Power - CardWorth.Middling);
                if (utility > utilityBest)
                {
                    (utilityBest, utilitySecond) = (utility, utilityBest);
                }
                else if (utility > utilitySecond)
                {
                    utilitySecond = utility;
                }
            }

            if (makes >= 1)
            {
                for (int color = 0; color < 5; color++)
                {
                    if (((card.Produces >> color) & 1) != 0 && ((colors >> color) & 1) == 0)
                    {
                        fixers[color]++;
                    }
                }
            }
        }

        landWorth += utilityBest + utilitySecond;

        Span<double> sources = stackalloc double[5];
        for (int color = 0; color < 5; color++)
        {
            sources[color] = ((colors >> color) & 1) == 0 ? Math.Min(3 + fixers[color], 5) : pair ? PairSources + (0.5 * Math.Min(duals, 4)) : 16;
        }

        // What the colours have worth cheating into play, and what the picks to come are counted on for.
        double bigCreatures = hope;
        double bigArtifacts = hope;
        double outlets = hope;
        for (int index = 0; index < count; index++)
        {
            DraftCard card = At(index);
            if ((card.Payload != 0 || card.Outlet) && card.OffColor(colors) == 0)
            {
                bigCreatures += (card.Payload & DraftCard.CheatsCreature) != 0 ? 1 : 0;
                bigArtifacts += (card.Payload & DraftCard.CheatsArtifact) != 0 ? 1 : 0;
                outlets += card.Outlet ? 1 : 0;
            }
        }

        var payloads = new Payloads(bigCreatures, bigArtifacts, outlets);

        // The spells these colours cast, and for each other colour the ones it would take a splash to.
        double[] worth = new double[count];
        int[] kept = new int[count];
        int spells = 0;
        Span<double> splashBest = stackalloc double[5];
        Span<double> splashSecond = stackalloc double[5];
        Span<int> splashBestAt = stackalloc int[5];
        Span<int> splashSecondAt = stackalloc int[5];
        splashBestAt.Fill(-1);
        splashSecondAt.Fill(-1);

        for (int index = 0; index < count; index++)
        {
            DraftCard card = At(index);
            if (card.IsLand)
            {
                continue;
            }

            int off = card.OffColor(colors);
            if (off == 0)
            {
                worth[spells] = DeckBuilder.PowerIn(card, colors, payloads, Deck) * Castable(card, sources, colors);
                kept[spells++] = index;
                if (parts is not null && index == pool.Count)
                {
                    parts.Cast = Castable(card, sources, colors);
                    parts.Support = payloads.Support(card, Deck);
                    parts.Colored = DeckBuilder.PowerIn(card, colors) / Math.Max(card.Power, 1e-9);
                }
            }
            else if (off == 1)
            {
                for (int color = 0; color < 5; color++)
                {
                    int splashed = colors | (1 << color);
                    if (((colors >> color) & 1) != 0 || card.OffColor(splashed) != 0 || card.Pips(color, splashed) != 1)
                    {
                        continue;
                    }

                    // A splash costs the main colours a source or two as well as being hard to cast itself.
                    double value = card.Power * payloads.Support(card, Deck) * Castable(card, sources, splashed) * 0.9;
                    if (parts is not null && index == pool.Count)
                    {
                        parts.SplashCast[color] = Castable(card, sources, splashed);
                        parts.Support = payloads.Support(card, Deck);
                    }

                    if (value > splashBest[color])
                    {
                        (splashSecond[color], splashSecondAt[color]) = (splashBest[color], splashBestAt[color]);
                        (splashBest[color], splashBestAt[color]) = (value, index);
                    }
                    else if (value > splashSecond[color])
                    {
                        (splashSecond[color], splashSecondAt[color]) = (value, index);
                    }
                }
            }
        }

        int splash = -1;
        for (int color = 0; color < 5; color++)
        {
            if (splashBest[color] > 0 && (splash < 0 || splashBest[color] + splashSecond[color] > splashBest[splash] + splashSecond[splash]))
            {
                splash = color;
            }
        }

        if (splash >= 0)
        {
            worth[spells] = splashBest[splash];
            kept[spells++] = splashBestAt[splash];
            if (splashSecondAt[splash] >= 0)
            {
                worth[spells] = splashSecond[splash];
                kept[spells++] = splashSecondAt[splash];
            }
        }

        // Which way the deck leans, from its best cards; then each card weighed knowing it.
        int[] order = new int[spells];
        double[] worths = worth;
        for (int at = 0; at < spells; at++)
        {
            order[at] = at;
        }

        Sort(order, worths, spells, -1);
        double speed = 0;
        int leaning = Math.Min(spells, Tuning.Slots);
        for (int at = 0; at < leaning; at++)
        {
            speed += At(kept[order[at]]).Speed;
        }

        double lean = Math.Clamp(speed * 2.5 / (leaning + 6), -1, 1);
        for (int at = 0; at < spells; at++)
        {
            worths[at] += DeckBuilder.Synergy * At(kept[at]).Speed * lean;
        }

        // The card being weighed goes in whatever it is worth: a deck can be better for a two-drop that is a little
        // worse than what it would otherwise have had. Whether the deck is better for it is the caller's to compare.
        int weighed = and is null ? -1 : Array.IndexOf(kept, pool.Count, 0, spells);
        Sort(order, worths, spells, weighed);
        if (parts is not null)
        {
            parts.Lean = lean;
            parts.Splash = splash;
            parts.Value = weighed >= 0 ? worths[weighed] : double.NaN;
            parts.Splashed = weighed >= 0 && and!.OffColor(colors) == 1;
            if (parts.Splashed)
            {
                parts.Cast = parts.SplashCast[splash];
            }
        }

        // The deck: the best of what it has and what it expects, as many as it holds.
        double total = 0;
        int have = 0;
        int come = 0;
        int creatures = 0;
        int cheap = 0;
        int expensive = 0;
        int makers = 0;
        for (int slot = 0; slot < Tuning.Slots; slot++)
        {
            double mine = have < spells ? worths[order[have]] : double.NegativeInfinity;
            double theirs = come < expected.Length ? expected[come] : double.NegativeInfinity;
            if (double.IsNegativeInfinity(mine) && double.IsNegativeInfinity(theirs))
            {
                break;
            }

            if (mine >= theirs || (have == 0 && weighed >= 0))
            {
                parts?.Played.Add(kept[order[have]]);
                DraftCard card = At(kept[order[have++]]);
                total += mine;
                creatures += card.IsCreature ? 1 : 0;
                cheap += card.ManaValue <= 2 ? 1 : 0;
                expensive += card.ManaValue >= 5 ? 1 : 0;
                makers += card.MakesMana ? 1 : 0;
            }
            else
            {
                total += theirs;
                come++;
            }
        }

        double shape = DeckBuilder.ShapeCost(have, creatures, cheap, expensive, lean, Deck, makers);
        if (parts is not null)
        {
            parts.Expected = expected.Take(come).Sum();
            parts.Own = total - parts.Expected;
            parts.Come = come;
            parts.Lands = landWorth;
            parts.Shape = shape;
            parts.Creatures = creatures;
            parts.Cheap = cheap;
            parts.Costly = expensive;
            parts.ShapeBy = DeckBuilder.ShapeParts(have, creatures, cheap, expensive, lean, Deck, makers);

            // The first left out: what a card put in instead pushed out of the deck.
            double mine = have < spells ? worths[order[have]] : double.NegativeInfinity;
            double theirs = come < expected.Length ? expected[come] : double.NegativeInfinity;
            if (have + come >= Tuning.Slots && (mine > double.NegativeInfinity || theirs > double.NegativeInfinity))
            {
                (parts.Out, parts.OutAt) = mine >= theirs ? (mine, kept[order[have]]) : (theirs, -1);
            }
        }

        return total + landWorth - shape;
    }

    /// <summary>
    /// Puts the best first, and one of them before all the rest if asked. An insertion sort: there are a few dozen
    /// at most, and the second time they are nearly in order already.
    /// </summary>
    private static void Sort(int[] order, double[] worths, int count, int first)
    {
        for (int at = 1; at < count; at++)
        {
            int moving = order[at];
            double key = moving == first ? double.PositiveInfinity : worths[moving];
            int to = at - 1;
            while (to >= 0 && (order[to] == first ? double.PositiveInfinity : worths[order[to]]) < key)
            {
                order[to + 1] = order[to];
                to--;
            }

            order[to + 1] = moving;
        }
    }

    private static double Castable(DraftCard card, ReadOnlySpan<double> sources, int deck)
    {
        double worst = 1;
        for (int color = 0; color < 5; color++)
        {
            int pips = card.Pips(color, deck);
            if (pips > 0)
            {
                worst = Math.Min(worst, ManaBase.Castable(sources[color], card.ManaValue, pips));
            }
        }

        return worst;
    }
}
