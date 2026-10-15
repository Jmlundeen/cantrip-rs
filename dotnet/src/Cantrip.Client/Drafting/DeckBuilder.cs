namespace Cantrip.Client.Drafting;

/// <summary>A deck built from a pool: which of the pool's cards, which basic lands, and what the builder made of it.</summary>
public sealed class BuiltDeck
{
    /// <summary>The places in the pool of the main deck's cards, spells and lands.</summary>
    public required IReadOnlyList<int> Main { get; init; }

    /// <summary>Basic lands added, by name.</summary>
    public required IReadOnlyDictionary<string, int> Basics { get; init; }

    /// <summary>The deck's main colours, as a mask (<see cref="DraftCard.AllColors"/>).</summary>
    public int Colors { get; init; }

    /// <summary>The colour splashed, as a mask; zero for none.</summary>
    public int Splash { get; init; }

    /// <summary>How many of the deck's cards are lands, basic or not.</summary>
    public int Lands { get; init; }

    /// <summary>Sources of each colour, white first: lands that make it, and half for each other card that does.</summary>
    public required IReadOnlyList<double> Sources { get; init; }

    /// <summary>What the builder thinks the deck is worth: its spells' power, as far as its lands cast them, less what its shape costs it.</summary>
    public double Score { get; init; }

    /// <summary>Which way the deck leans, from -1 (late: control) to 1 (early: aggro) (<see cref="DeckBuilder.Lean"/>).</summary>
    public double Lean { get; init; }

    /// <summary>
    /// What the land count was last worked out from (<see cref="ManaBase.LandsFor"/>): the spells' average mana value, the
    /// cheap ones that make mana or draw, the free mana, and the count it asked for. For saying why; zero if never asked.
    /// </summary>
    public (double Average, int Cheap, int Free, int Wanted) LandsAsked { get; init; }

    public int Size => Main.Count + Basics.Values.Sum();
}

/// <summary>The numbers a deck is built by, each one something that can be set against another in play (<c>cantrip-botlab draft</c>).</summary>
public sealed record DeckTuning
{
    /// <summary>
    /// What a third colour costs a deck beyond the sources it takes, in worth. Nothing: decks that never splashed came
    /// out behind the ones that splashed whenever the sum said to, twice, and a cost of one did no better than none.
    /// </summary>
    public double SplashCost { get; init; }

    /// <summary>How many creatures a deck of twenty-three spells wants, unless it leans late, when it wants <see cref="LateFewer"/> fewer.</summary>
    public double Creatures { get; init; } = 13;

    /// <summary>How many fewer creatures a deck that leans late (a control deck) wants.</summary>
    public double LateFewer { get; init; } = 4;

    /// <summary>What each creature a deck is short of costs it, in worth.</summary>
    public double CreatureCost { get; init; } = 0.45;

    /// <summary>How many cards of a third colour a deck may splash.</summary>
    public int SplashCards { get; init; } = 3;

    /// <summary>Lands more or fewer than the curve asks for.</summary>
    public int Lands { get; init; }

    /// <summary>What a second colour costs a deck, likewise: what a pool has to gain by its second colour to play it.</summary>
    public double SecondColorCost { get; init; }

    /// <summary>How many plays at two mana or less a deck of twenty-three spells wants, unless it leans early, when it wants three more.</summary>
    public double Cheap { get; init; } = 6;

    /// <summary>What each early play a deck is short of costs it, in worth.</summary>
    public double CheapCost { get; init; } = 0.3;

    /// <summary>
    /// How many cards at five mana or more a deck of twenty-three spells can hold: two fewer for a deck that leans
    /// early, one more for a deck that leans late.
    /// </summary>
    public double Expensive { get; init; } = 5;

    /// <summary>What each expensive card beyond those costs a deck, in worth.</summary>
    public double ExpensiveCost { get; init; } = 0.5;

    /// <summary>
    /// The share of its worth a card that cheats something into play (<see cref="DraftCard.Cheats"/>) keeps with
    /// nothing in the deck worth cheating (<see cref="DraftCard.Payload"/>); 1 is to read no such thing.
    /// </summary>
    public double Unsupported { get; init; } = 0.4;

    /// <summary>How many cards worth cheating make such a card whole.</summary>
    public double Payloads { get; init; } = 1;

    /// <summary>
    /// The share of what a cheat from a graveyard (<see cref="DraftCard.FromGraveyard"/>) gets from its payloads that
    /// rests on the deck having a way to put them there (<see cref="DraftCard.Outlet"/>); one outlet makes it whole.
    /// Nothing: as shipped a reanimation spell is taken to find its target without help.
    /// </summary>
    public double Outlets { get; init; }

    /// <summary>
    /// How many more cards at five mana or more a deck can hold for each card that makes mana beyond two. Nothing: as
    /// shipped a deck of mana rocks and elves holds no more expensive cards than any other.
    /// </summary>
    public double Ramp { get; init; }
}

/// <summary>
/// How many cards of a deck are worth cheating into play, of each kind a card can cheat (<see cref="DraftCard.Payload"/>):
/// what a card that cheats is worth depends on it.
/// </summary>
public readonly record struct Payloads(double Creatures, double Artifacts, double Outlets = 0)
{
    /// <summary>The cards of these worth cheating.</summary>
    public static Payloads Of(IEnumerable<DraftCard> cards)
    {
        int creatures = 0;
        int artifacts = 0;
        int outlets = 0;
        foreach (DraftCard card in cards)
        {
            creatures += (card.Payload & DraftCard.CheatsCreature) != 0 ? 1 : 0;
            artifacts += (card.Payload & DraftCard.CheatsArtifact) != 0 ? 1 : 0;
            outlets += card.Outlet ? 1 : 0;
        }

        return new Payloads(creatures, artifacts, outlets);
    }

    /// <summary>The share of its power a card keeps in a deck with these: all of it but for a card that cheats with too little to cheat.</summary>
    public double Support(DraftCard card, DeckTuning tuning)
    {
        if (card.Cheats == 0 || tuning.Unsupported >= 1)
        {
            return 1;
        }

        double has = ((card.Cheats & DraftCard.CheatsCreature) != 0 ? Creatures : 0) + ((card.Cheats & DraftCard.CheatsArtifact) != 0 ? Artifacts : 0);
        double fed = card.FromGraveyard && tuning.Outlets > 0 ? 1 - (tuning.Outlets * (1 - Math.Min(1, Outlets))) : 1;
        return tuning.Unsupported + ((1 - tuning.Unsupported) * Math.Min(1, has / Math.Max(tuning.Payloads, 1e-9)) * fed);
    }
}

/// <summary>
/// Builds the deck a pool should be played as: the smallest deck allowed, never a card more.
/// <para>
/// It does not decide the colours and then fill them in. It builds every deck the pool could be - each colour alone,
/// each pair, and each of those with a third colour splashed - and keeps the best, where best is what the spells are
/// worth <i>as far as the lands can cast them</i> (<see cref="ManaBase.Castable(DraftCard, IReadOnlyList{double}, int, int)"/>):
/// a deck's colours, its splash and its lands are settled by the same sum, so a splash has to pay for the sources it
/// takes from the main colours, and a third colour is only played when the pool has the lands for it.
/// </para>
/// <para>
/// For one choice of colours: the spells are the best the colours can cast, traded one for another while a trade
/// improves the deck's shape - enough creatures, enough to do early, not too much that costs five or more. The lands
/// are as many as the curve wants (<see cref="ManaBase.LandsFor"/>): the pool's lands that make two of the deck's
/// colours, a utility land or two where the mana can carry them, and basic lands split whichever way casts the most.
/// </para>
/// </summary>
public static class DeckBuilder
{
    /// <summary>The basic land of each colour, white first.</summary>
    public static readonly IReadOnlyList<string> BasicLands = ["Plains", "Island", "Swamp", "Mountain", "Forest"];

    /// <summary>Each colour alone and each pair, as masks.</summary>
    public static readonly IReadOnlyList<int> Combinations =
    [
        .. Enumerable.Range(0, 5).Select(color => 1 << color),
        .. from first in Enumerable.Range(0, 5) from second in Enumerable.Range(first + 1, 4 - first) select (1 << first) | (1 << second),
    ];

    private static readonly DeckTuning Shipped = new();

    /// <summary>How much a deck's leaning - early or late - adds to a card that leans the same way, and takes from one that leans the other.</summary>
    public const double Synergy = 0.5;

    /// <summary>What a land that makes no coloured mana has to be worth to be played for what else it does.</summary>
    private const double UtilityLand = 4.0;

    /// <param name="pool">The pool, each card read (<see cref="DraftCard.Read"/>) and worth what the drafter knows it to be.</param>
    /// <param name="minimum">The fewest cards a deck may have, which is how many this one will.</param>
    /// <param name="tried">Given every deck that was weighed, if not null: for saying why the one built won.</param>
    public static BuiltDeck Build(IReadOnlyList<DraftCard> pool, int minimum, DeckTuning? tuning = null, List<BuiltDeck>? tried = null)
    {
        ArgumentNullException.ThrowIfNull(pool);
        tuning ??= new DeckTuning();
        BuiltDeck? best = null;
        foreach (int main in Combinations)
        {
            foreach (int splash in (int[])[0, 1, 2, 4, 8, 16])
            {
                if ((splash & main) != 0)
                {
                    continue;
                }

                BuiltDeck? deck = Try(pool, minimum, main, splash, tuning);
                if (deck is not null)
                {
                    tried?.Add(deck);
                }

                if (deck is not null && (best is null || deck.Score > best.Score + 1e-9))
                {
                    best = deck;
                }
            }
        }

        return best!;
    }

    /// <summary>
    /// What a card is worth in a deck of these colours that can cast it: its power, but for a Mox or a mana creature
    /// that makes only a colour the deck does not play, which is left paying for generic costs.
    /// </summary>
    public static double PowerIn(DraftCard card, int colors) =>
        !card.IsLand && card.MakesMana && card.Produces != 0 && (card.Produces & colors) == 0 ? card.Power * 0.6 : card.Power;

    /// <summary>
    /// What a card is worth in a deck of these colours with these cards worth cheating into play
    /// (<see cref="Payloads.Support"/>): <see cref="PowerIn(DraftCard, int)"/>, less for a card that cheats with too
    /// little to cheat.
    /// </summary>
    public static double PowerIn(DraftCard card, int colors, Payloads payloads, DeckTuning? tuning = null) =>
        PowerIn(card, colors) * payloads.Support(card, tuning ?? Shipped);

    /// <summary>What a deck's shape costs it: creatures it is short of, early plays it is short of, expensive cards beyond what it can hold.</summary>
    /// <param name="makers">How many of the spells make mana: a deck of them holds more expensive cards (<see cref="DeckTuning.Ramp"/>).</param>
    public static double ShapeCost(int spells, int creatures, int cheap, int expensive, double lean, DeckTuning? tuning = null, int makers = 0)
    {
        (double short_, double early, double heavy) = ShapeParts(spells, creatures, cheap, expensive, lean, tuning, makers);
        return short_ + early + heavy;
    }

    /// <summary>What a deck's shape costs it (<see cref="ShapeCost"/>), part by part: creatures short, early plays short, expensive cards over.</summary>
    public static (double Creatures, double Cheap, double Expensive) ShapeParts(int spells, int creatures, int cheap, int expensive, double lean, DeckTuning? tuning = null, int makers = 0)
    {
        tuning ??= Shipped;
        double scale = spells / 23.0;
        double wantCreatures = (lean < -0.25 ? tuning.Creatures - tuning.LateFewer : tuning.Creatures) * scale;
        double wantCheap = (lean > 0.25 ? tuning.Cheap + 3 : tuning.Cheap) * scale;
        double holdsExpensive = ((lean > 0.25 ? tuning.Expensive - 2 : lean < -0.25 ? tuning.Expensive + 1 : tuning.Expensive) * scale)
            + (tuning.Ramp * Math.Max(0, makers - 2));
        return (tuning.CreatureCost * Math.Max(0, wantCreatures - creatures),
            tuning.CheapCost * Math.Max(0, wantCheap - cheap),
            tuning.ExpensiveCost * Math.Max(0, expensive - holdsExpensive));
    }

    private static BuiltDeck? Try(IReadOnlyList<DraftCard> pool, int minimum, int main, int splash, DeckTuning tuning)
    {
        int colors = main | splash;
        int splashColor = splash == 0 ? -1 : int.TrailingZeroCount(splash);

        // What the colours can cast. A splash is for cards with one symbol of it, and only the best few.
        List<int> candidates = [];
        List<int> splashed = [];
        for (int place = 0; place < pool.Count; place++)
        {
            DraftCard card = pool[place];
            if (card.IsLand || card.OffColor(colors) > 0)
            {
                continue;
            }

            int pips = splashColor < 0 ? 0 : card.Pips(splashColor, colors);
            if (pips == 0)
            {
                candidates.Add(place);
            }
            else if (pips == 1)
            {
                splashed.Add(place);
            }
        }

        if (splash != 0)
        {
            if (splashed.Count == 0)
            {
                return null;
            }

            candidates.AddRange(splashed.OrderByDescending(place => pool[place].Power).Take(tuning.SplashCards));
        }

        // What the deck has worth cheating into play: at first what it can cast of it, and then, once spells are
        // chosen, what it plays - a cheat is worth nothing extra for a Giant left in the sideboard.
        Payloads payloads = Payloads.Of(candidates.Select(place => pool[place]));

        int mainColors = int.PopCount(main);
        int lands = (int)Math.Round(ManaBase.Lands * minimum / 40.0);
        double[] sources = new double[5];
        for (int color = 0; color < 5; color++)
        {
            sources[color] = ((main >> color) & 1) != 0 ? (double)lands / mainColors : color == splashColor ? 3 : 0;
        }

        List<int> spells = [];
        List<int> landPlaces = [];
        int[] basics = new int[5];
        double score = 0;
        double leaning = 0;
        (double Average, int Cheap, int Free, int Wanted) asked = default;

        // The spells depend on the lands and the lands on the spells: a few rounds settle it.
        for (int round = 0; round < 3; round++)
        {
            (spells, double lean) = Spells(pool, candidates, sources, colors, minimum - lands, minimum, tuning, payloads);

            // The land count the curve wants; with too few spells to fill the rest, lands make up the number.
            double average = spells.Count == 0 ? 3 : spells.Average(place => (double)pool[place].ManaValue);
            int cheap = spells.Count(place => pool[place].ManaValue is 1 or 2 && (pool[place].MakesMana || pool[place].CardAdvantage));
            int free = spells.Count(place => pool[place].ManaValue == 0 && pool[place].MakesMana);
            int wanted = ManaBase.LandsFor(minimum, average, cheap, free) + tuning.Lands;
            if (round < 2)
            {
                asked = (average, cheap, free, wanted);
            }

            if (wanted != lands && round < 2)
            {
                lands = wanted;
                (spells, lean) = Spells(pool, candidates, sources, colors, minimum - lands, minimum, tuning, payloads);
            }

            lands = minimum - spells.Count;
            payloads = Payloads.Of(spells.Select(place => pool[place]));
            (landPlaces, basics, sources, double bonus) = Lands(pool, spells, lands, main, splash, minimum, payloads, tuning);
            leaning = lean;
            score = Worth(pool, spells, sources, colors, lean, minimum, tuning, payloads) + bonus
                - (splash != 0 ? tuning.SplashCost : 0) - (mainColors > 1 ? tuning.SecondColorCost : 0);
        }

        // A splash the lands came out with one source for, or none, is not a deck anyone would play.
        if (splashColor >= 0 && sources[splashColor] < 2)
        {
            return null;
        }

        Dictionary<string, int> named = new(StringComparer.Ordinal);
        for (int color = 0; color < 5; color++)
        {
            if (basics[color] > 0)
            {
                named[BasicLands[color]] = basics[color];
            }
        }

        return new BuiltDeck
        {
            Main = [.. spells.Concat(landPlaces).Order()],
            Basics = named,
            Colors = main,
            Splash = splash,
            Lands = lands,
            Sources = sources,
            Score = score,
            Lean = leaning,
            LandsAsked = asked,
        };
    }

    /// <summary>What each candidate is worth in this deck: its power as far as the lands cast it, and how it suits the deck's leaning.</summary>
    private static double[] Each(IReadOnlyList<DraftCard> pool, List<int> candidates, IReadOnlyList<double> sources, int colors, double lean, int size, DeckTuning tuning, Payloads payloads)
    {
        double[] worth = new double[candidates.Count];
        for (int at = 0; at < candidates.Count; at++)
        {
            DraftCard card = pool[candidates[at]];
            worth[at] = (PowerIn(card, colors, payloads, tuning) * ManaBase.Castable(card, sources, colors, size)) + (Synergy * card.Speed * lean);
        }

        return worth;
    }

    /// <summary>Which way a set of spells leans, from -1 (late) to 1 (early).</summary>
    public static double Lean(IEnumerable<DraftCard> spells)
    {
        double sum = 0;
        int count = 0;
        foreach (DraftCard card in spells)
        {
            sum += card.Speed;
            count++;
        }

        return Math.Clamp(sum * 2.5 / (count + 6), -1, 1);
    }

    /// <summary>The best spells the colours can cast, as many as asked for or as there are, and which way they lean.</summary>
    private static (List<int> Spells, double Lean) Spells(IReadOnlyList<DraftCard> pool, List<int> candidates, IReadOnlyList<double> sources, int colors, int count, int size, DeckTuning tuning, Payloads payloads)
    {
        count = Math.Min(count, candidates.Count);

        // Which way the deck leans is read from its best cards, and then every card is weighed knowing it.
        double[] plain = Each(pool, candidates, sources, colors, 0, size, tuning, payloads);
        double lean = Lean(Enumerable.Range(0, candidates.Count).OrderByDescending(at => plain[at]).Take(count).Select(at => pool[candidates[at]]));
        double[] worth = Each(pool, candidates, sources, colors, lean, size, tuning, payloads);

        List<int> order = [.. Enumerable.Range(0, candidates.Count).OrderByDescending(at => worth[at]).ThenBy(at => candidates[at])];
        List<int> chosen = order[..count];
        List<int> left = order[count..];

        // What a card does to the deck's shape is whether it is a creature, cheap, or expensive: eight kinds of card.
        int Kind(int at)
        {
            DraftCard card = pool[candidates[at]];
            return (card.IsCreature ? 1 : 0) | (card.ManaValue <= 2 ? 2 : 0) | (card.ManaValue >= 5 ? 4 : 0);
        }

        int creatures = chosen.Count(at => (Kind(at) & 1) != 0);
        int cheap = chosen.Count(at => (Kind(at) & 2) != 0);
        int expensive = chosen.Count(at => (Kind(at) & 4) != 0);

        // Trade a chosen card for one left out while a trade makes the deck better for its shape. Only the worst
        // chosen card of each kind and the best left out of each kind can be the trade to make.
        int[] worstIn = new int[8];
        int[] bestOut = new int[8];
        for (int trades = 0; trades < 40 && left.Count > 0; trades++)
        {
            Array.Fill(worstIn, -1);
            Array.Fill(bestOut, -1);
            for (int o = 0; o < chosen.Count; o++)
            {
                int kind = Kind(chosen[o]);
                if (worstIn[kind] < 0 || worth[chosen[o]] < worth[chosen[worstIn[kind]]])
                {
                    worstIn[kind] = o;
                }
            }

            for (int i = 0; i < left.Count; i++)
            {
                int kind = Kind(left[i]);
                if (bestOut[kind] < 0 || worth[left[i]] > worth[left[bestOut[kind]]])
                {
                    bestOut[kind] = i;
                }
            }

            int makers = tuning.Ramp == 0 ? 0 : chosen.Count(at => pool[candidates[at]].MakesMana);
            int Maker(int at) => tuning.Ramp != 0 && pool[candidates[at]].MakesMana ? 1 : 0;
            double cost = ShapeCost(count, creatures, cheap, expensive, lean, tuning, makers);
            (int Out, int In, double Gain) best = (-1, -1, 1e-9);
            for (int goes = 0; goes < 8; goes++)
            {
                for (int comes = 0; comes < 8 && worstIn[goes] >= 0; comes++)
                {
                    if (bestOut[comes] < 0 || comes == goes)
                    {
                        continue;
                    }

                    double after = ShapeCost(
                        count,
                        creatures + (comes & 1) - (goes & 1),
                        cheap + ((comes >> 1) & 1) - ((goes >> 1) & 1),
                        expensive + ((comes >> 2) & 1) - ((goes >> 2) & 1),
                        lean,
                        tuning,
                        makers + Maker(left[bestOut[comes]]) - Maker(chosen[worstIn[goes]]));
                    double gain = worth[left[bestOut[comes]]] - worth[chosen[worstIn[goes]]] + cost - after;
                    if (gain > best.Gain)
                    {
                        best = (worstIn[goes], bestOut[comes], gain);
                    }
                }
            }

            if (best.Out < 0)
            {
                break;
            }

            int leaving = Kind(chosen[best.Out]);
            int joining = Kind(left[best.In]);
            creatures += (joining & 1) - (leaving & 1);
            cheap += ((joining >> 1) & 1) - ((leaving >> 1) & 1);
            expensive += ((joining >> 2) & 1) - ((leaving >> 2) & 1);
            (chosen[best.Out], left[best.In]) = (left[best.In], chosen[best.Out]);
        }

        return ([.. chosen.Select(at => candidates[at])], lean);
    }

    /// <summary>What a deck's spells are worth with these sources, less what its shape costs.</summary>
    private static double Worth(IReadOnlyList<DraftCard> pool, List<int> spells, IReadOnlyList<double> sources, int colors, double lean, int size, DeckTuning tuning, Payloads payloads)
    {
        double worth = 0;
        foreach (int place in spells)
        {
            DraftCard card = pool[place];
            worth += (PowerIn(card, colors, payloads, tuning) * ManaBase.Castable(card, sources, colors, size)) + (Synergy * card.Speed * lean);
        }

        return worth - ShapeCost(
            spells.Count,
            spells.Count(place => pool[place].IsCreature),
            spells.Count(place => pool[place].ManaValue <= 2),
            spells.Count(place => pool[place].ManaValue >= 5),
            lean,
            tuning,
            spells.Count(place => pool[place].MakesMana));
    }

    /// <summary>
    /// The lands for a set of spells: the pool's that make two of the deck's colours, then those that are a basic land
    /// and something more, then basic lands - split every way there is, keeping the split that casts the most - and a
    /// utility land or two in place of a basic where the deck is better for it.
    /// </summary>
    private static (List<int> Places, int[] Basics, double[] Sources, double Bonus) Lands(IReadOnlyList<DraftCard> pool, List<int> spells, int lands, int main, int splash, int size, Payloads payloads, DeckTuning tuning)
    {
        int colors = main | splash;
        List<int> fixing = [];
        List<int> utility = [];
        for (int place = 0; place < pool.Count; place++)
        {
            DraftCard card = pool[place];
            if (!card.IsLand || !card.MakesMana)
            {
                continue;
            }

            int makes = int.PopCount(card.Produces & colors);
            if (makes >= 2 || (makes == 1 && (card.Produces & main) != 0 && card.Power >= UtilityLand))
            {
                fixing.Add(place);
            }
            else if (makes == 0 && card.Power >= UtilityLand)
            {
                utility.Add(place);
            }
        }

        fixing = [.. fixing.OrderByDescending(place => int.PopCount(pool[place].Produces & colors)).ThenByDescending(place => pool[place].Power).Take(lands)];
        utility = [.. utility.OrderByDescending(place => pool[place].Power).Take(2)];

        (List<int> Places, int[] Basics, double[] Sources, double Bonus) best = ([], new int[5], new double[5], 0);
        double bestWorth = double.NegativeInfinity;
        for (int extra = 0; extra <= utility.Count; extra++)
        {
            List<int> places = [.. fixing, .. utility.Take(extra)];
            if (places.Count > lands)
            {
                break;
            }

            // What the deck has before a basic land is added: its other lands, and half for each spell that makes mana.
            double[] given = new double[5];
            foreach (int place in fixing)
            {
                Count(given, pool[place].Produces & colors, 1);
            }

            foreach (int place in spells)
            {
                if (pool[place].MakesMana)
                {
                    Count(given, pool[place].Produces & colors, 0.5);
                }
            }

            // A utility land is worth what it does, and costs what the spells lose by the basic land it replaces.
            double bonus = utility.Take(extra).Sum(place => 0.6 * (pool[place].Power - CardWorth.Middling));
            (int[] basics, double[] sources, double cast) = Split(pool, spells, lands - places.Count, given, main, splash, size, payloads, tuning);
            if (cast + bonus > bestWorth + 1e-9)
            {
                best = (places, basics, sources, bonus);
                bestWorth = cast + bonus;
            }
        }

        return best;
    }

    private static void Count(double[] sources, int colors, double each)
    {
        for (int color = 0; color < 5; color++)
        {
            if (((colors >> color) & 1) != 0)
            {
                sources[color] += each;
            }
        }
    }

    /// <summary>
    /// Basic lands split among the deck's colours whichever way its spells are cast best; among splits that cast them
    /// equally, the one nearest the colours' share of the symbols.
    /// </summary>
    private static (int[] Basics, double[] Sources, double Cast) Split(IReadOnlyList<DraftCard> pool, List<int> spells, int count, double[] given, int main, int splash, int size, Payloads payloads, DeckTuning tuning)
    {
        int colors = main | splash;
        List<int> among = [.. Enumerable.Range(0, 5).Where(color => ((colors >> color) & 1) != 0)];
        double[] pips = new double[5];
        foreach (int place in spells)
        {
            foreach (int color in among)
            {
                pips[color] += pool[place].Pips(color, colors);
            }
        }

        double allPips = Math.Max(1, pips.Sum());

        // Each spell's power, mana value and symbols of each colour, read once: every split asks for them again.
        double[] power = [.. spells.Select(place => PowerIn(pool[place], colors, payloads, tuning))];
        int[] manaValue = [.. spells.Select(place => pool[place].ManaValue)];
        int[][] asks = [.. spells.Select(place => among.Select(color => pool[place].Pips(color, colors)).ToArray())];
        double per = 40.0 / Math.Max(size, 1);
        (int[] Basics, double[] Sources, double Cast) best = (new int[5], [.. given], double.NegativeInfinity);
        int[] basics = new int[5];

        void Deal(int at, int left)
        {
            if (at == among.Count - 1)
            {
                basics[among[at]] = left;
                if (splash != 0 && basics[int.TrailingZeroCount(splash)] > 4)
                {
                    return;
                }

                double[] sources = [.. given];
                foreach (int color in among)
                {
                    sources[color] += basics[color];
                }

                double cast = 0;
                for (int spell = 0; spell < power.Length; spell++)
                {
                    double worst = 1;
                    for (int color = 0; color < among.Count; color++)
                    {
                        if (asks[spell][color] > 0)
                        {
                            worst = Math.Min(worst, ManaBase.Castable(sources[among[color]] * per, manaValue[spell], asks[spell][color]));
                        }
                    }

                    cast += power[spell] * worst;
                }

                foreach (int color in among)
                {
                    cast -= 0.01 * Math.Abs(basics[color] - (count * pips[color] / allPips));
                }

                if (cast > best.Cast + 1e-9)
                {
                    best = ([.. basics], sources, cast);
                }

                return;
            }

            for (int take = 0; take <= left; take++)
            {
                basics[among[at]] = take;
                Deal(at + 1, left - take);
            }
        }

        Deal(0, Math.Max(0, count));
        return best;
    }
}
