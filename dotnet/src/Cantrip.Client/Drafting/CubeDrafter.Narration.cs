using System.Globalization;
using Cantrip.Protocol;

namespace Cantrip.Client.Drafting;

/// <summary>
/// The drafter's own account of a pick, a grid line, a Winston pile or a deck: what it weighed and why the one it
/// chose came out on top, in lines a player can read. Worked out after the choice is made, from the numbers it was made
/// by, and only when asked: nothing here is a part of any choice.
/// </summary>
public sealed partial class CubeDrafter
{
    /// <summary>What a projection of a deck (<see cref="Project(List{DraftCard}, DraftCard?, int, double[], double, Parts?)"/>) was made of.</summary>
    private sealed class Parts
    {
        /// <summary>The pool's own cards in the deck's slots, and the picks still to come that filled the rest.</summary>
        public double Own { get; set; }

        public double Expected { get; set; }

        public int Come { get; set; }

        /// <summary>What the pool's lands add: fixing and utility lands.</summary>
        public double Lands { get; set; }

        /// <summary>What the deck's shape costs it (<see cref="DeckBuilder.ShapeCost"/>).</summary>
        public double Shape { get; set; }

        public int Creatures { get; set; }

        public int Cheap { get; set; }

        public int Costly { get; set; }

        /// <summary>The shape's cost part by part (<see cref="DeckBuilder.ShapeParts"/>).</summary>
        public (double Creatures, double Cheap, double Expensive) ShapeBy { get; set; }

        public double Lean { get; set; }

        public int Splash { get; set; } = -1;

        /// <summary>The pool's cards in the deck (an index of the pool, or its count for the card weighed).</summary>
        public List<int> Played { get; } = [];

        /// <summary>What the card weighed is worth in the deck; NaN when the deck cannot play it.</summary>
        public double Value { get; set; } = double.NaN;

        public double Cast { get; set; } = 1;

        public double[] SplashCast { get; } = new double[5];

        public double Support { get; set; } = 1;

        /// <summary>The share of its power a mana card keeps for making none of the deck's colours.</summary>
        public double Colored { get; set; } = 1;

        public bool Splashed { get; set; }

        /// <summary>The first thing left out of the deck's slots: a card of the pool (its index) or a pick to come (-1).</summary>
        public double Out { get; set; } = double.NaN;

        public int OutAt { get; set; } = -1;
    }

    /// <summary>A booster pick as it was weighed: everything <see cref="Choose"/> worked out, kept for saying why.</summary>
    private sealed record Pick(
        DraftMessage Draft,
        List<DraftCard> Pack,
        List<DraftCard> Pool,
        List<int> Places,
        double[] Scores,
        double[] Beliefs,
        double[] Without,
        double[][] With,
        double[][] Expected,
        double Hope,
        DraftMemory.Sight Sight);

    private static readonly CultureInfo Invariant = CultureInfo.InvariantCulture;

    /// <summary>
    /// An explanation, or - should working one out fail - a line saying so: telling is never a reason for a choice not
    /// to be made.
    /// </summary>
    private static List<string> Explaining(Func<List<string>> explain)
    {
        try
        {
            return explain();
        }
        catch (Exception error) when (error is not OutOfMemoryException)
        {
            return [$"  (could not say why: {error.GetType().Name}: {error.Message})"];
        }
    }

    /// <summary>A gain or a score, signed, two places.</summary>
    private static string Signed(double value) => value.ToString("+0.00;-0.00;0.00", Invariant);

    /// <summary>A worth, one place.</summary>
    private static string One(double value) => value.ToString("0.0", Invariant);

    private static string Percent(double share) => share.ToString("0%", Invariant);

    private static string Cards(int count) => count == 1 ? "1 card" : $"{count} cards";

    /// <summary>A colour mask as its letters, white first; C for none.</summary>
    public static string ColorText(int mask) => mask == 0 ? "C" : string.Concat(Enumerable.Range(0, 5).Where(color => ((mask >> color) & 1) != 0).Select(color => "WUBRG"[color]));

    /// <summary>A deck's colours, with its splash after a plus in small letters: "UR+b".</summary>
    public static string DeckText(int main, int splash) => ColorText(main) + (splash == 0 ? string.Empty : "+" + ColorText(splash).ToLowerInvariant());

    /// <summary>Where a card stands in the cube's pick order, as far as this drafter was told.</summary>
    private string RankText(DraftCard card) =>
        card.Ranked && _rankOf.TryGetValue(card.Name, out int rank) && rank != _unranked ? $"#{rank + 1}" : "unranked";

    /// <summary>The hypotheses most believed: "UR 41%, U 18%".</summary>
    private static string BeliefText(double[] beliefs)
    {
        IEnumerable<string> told = Enumerable.Range(0, beliefs.Length)
            .OrderByDescending(at => beliefs[at])
            .Take(4)
            .Where(at => beliefs[at] >= 0.04)
            .Select(at => $"{ColorText(DeckBuilder.Combinations[at])} {Percent(beliefs[at])}");
        return string.Join(", ", told);
    }

    private static double[] Believed(double[] worths, double sureness)
    {
        double best = worths.Max();
        double[] beliefs = [.. worths.Select(worth => Math.Exp(sureness * (worth - best)))];
        double all = beliefs.Sum();
        return [.. beliefs.Select(belief => belief / all)];
    }

    /// <summary>What draft this is, and how to read what is told of it: the first lines of a seat's narration.</summary>
    public static IReadOnlyList<string> Opening(DraftMessage draft)
    {
        ArgumentNullException.ThrowIfNull(draft);
        return draft.Format switch
        {
            "grid" =>
            [
                $"Grid draft: {draft.Packs} grids of nine, a row or a column each.",
                "  Each line is scored by what the pool's likely decks gain with its cards, in worth; a card's worth follows it.",
                "  Worth is the cube's scale: 3 a middling card, 5 one of its best twenty, 7 the Power Nine.",
            ],
            "winston" =>
            [
                $"Winston draft: {draft.Total} cards, {draft.Piles.Count} piles.",
                "  A pile is taken when the pool's likely decks gain more by it than by passing it, in worth.",
                "  Worth is the cube's scale: 3 a middling card, 5 one of its best twenty, 7 the Power Nine.",
            ],
            _ =>
            [
                $"Booster draft: {draft.Packs} packs of {draft.PackSize}{(draft.Seats.Count > 0 ? $", {draft.Seats.Count} seats" : string.Empty)}.",
                "  Worth is the cube's scale, from the card's place (#) in the pick order: 3 a middling card, 5 one of its best twenty, 7 the Power Nine.",
                "  A card's score is roughly what the decks the pool could become gain with it, each weighed by how likely it is, and a little for its worth to break ties.",
                "  \"UR +2.30 (41%)\": a UR deck gains 2.3 with it, and the pool is 41% likely to be UR. \"in UR:\" says what that is made of.",
                "  open: how much better than usual each colour's coming picks are expected to be, in worth a pick; after \"<-\" what that was",
                "  read from - each colour's share of the playables in the packs, against the average colour - and the weight it is given.",
            ],
        };
    }

    /// <summary>
    /// The lines that tell a booster pick: the pack and what was taken; which decks the pool was believed to be; which
    /// colours were read as open and from what; what came back on a wheel; and the cards that came closest, each with
    /// what it would add to the decks it fits and, for the deck it fits best, what that is made of.
    /// </summary>
    private List<string> ExplainPick(Pick pick)
    {
        DraftMessage draft = pick.Draft;
        List<string> lines =
        [
            $"Pack {draft.PackNumber} pick {draft.PickNumber}: {Cards(pick.Pack.Count)}, pool {pick.Pool.Count}. Took {string.Join(" and ", pick.Places.Select(place => pick.Pack[place].Name))}.",
        ];

        if (pick.Pool.Count > 0)
        {
            lines.Add($"  deck so far: {BeliefText(pick.Beliefs)}");
        }

        if (OpenText(Math.Max(1, draft.PackNumber)) is { } open)
        {
            lines.Add($"  open, worth a pick: {open}");
        }

        if (WheelText(pick.Sight) is { } wheel)
        {
            lines.Add($"  wheel: {wheel}");
        }

        // The closest cards in full, and the best card by the pick order whatever it scored: why it was passed is the
        // question a player asks first.
        List<int> byScore = [.. Enumerable.Range(0, pick.Pack.Count).OrderByDescending(place => pick.Scores[place]).ThenByDescending(place => pick.Pack[place].Worth).ThenBy(place => place)];
        List<int> told = [.. byScore.Take(3).Union(pick.Places)];
        int bestWorth = Enumerable.Range(0, pick.Pack.Count).OrderByDescending(place => pick.Pack[place].Worth).ThenBy(place => place).First();
        if (!told.Contains(bestWorth))
        {
            told.Add(bestWorth);
        }

        int width = Math.Min(28, told.Max(place => pick.Pack[place].Name.Length));
        foreach (int place in told)
        {
            lines.AddRange(CandidateText(pick, place, width));
        }

        List<int> rest = [.. byScore.Where(place => !told.Contains(place)).Take(6)];
        if (rest.Count > 0)
        {
            lines.Add($"     also: {string.Join(", ", rest.Select(place => $"{pick.Pack[place].Name} {Signed(pick.Scores[place])}"))}");
        }

        return lines;
    }

    /// <summary>One card of a pack: its score and worth, the decks it adds most to, and what it adds to the first of them.</summary>
    private IEnumerable<string> CandidateText(Pick pick, int place, int width)
    {
        DraftCard card = pick.Pack[place];
        string mark = pick.Places.IndexOf(place) switch { 0 => "->", > 0 => "+>", _ => "  " };
        string name = card.Name.Length > width ? card.Name[..(width - 1)] + "." : card.Name.PadRight(width);

        // What each deck it could be gains by the card, weighed by how likely that deck is: very nearly the score.
        int[] fits = [.. Enumerable.Range(0, pick.Without.Length)
            .Where(at => pick.With[place][at] - pick.Without[at] > 0.005)
            .OrderByDescending(at => pick.Beliefs[at] * (pick.With[place][at] - pick.Without[at]))];
        // Decks it adds the same to are one: a first pick's card fits each of its colour's pairs alike.
        List<List<int>> groups = [];
        foreach (int at in fits)
        {
            double gain = pick.With[place][at] - pick.Without[at];
            if (groups.FirstOrDefault(group => Math.Abs(pick.With[place][group[0]] - pick.Without[group[0]] - gain) < 0.01) is { } same)
            {
                same.Add(at);
            }
            else if (groups.Count < 2)
            {
                groups.Add([at]);
            }
        }

        string fitText = fits.Length == 0
            ? "adds to no deck it could be"
            : string.Join(", ", groups.Select(group => $"{DecksText(group)} {Signed(pick.With[place][group[0]] - pick.Without[group[0]])} ({Percent(group.Sum(at => pick.Beliefs[at]))})"));

        yield return $"  {mark} {name} {Signed(pick.Scores[place]),6}  worth {One(card.Worth)} {RankText(card),-9} {fitText}";
        if (fits.Length > 0)
        {
            yield return $"{new string(' ', 6 + width)}{InDeckText(pick, place, fits[0])}";
        }
        else
        {
            yield return $"{new string(' ', 6 + width)}{NowhereText(pick, card)}";
        }
    }

    /// <summary>Several of the fifteen decks as a player would say them: "any pair", "W pairs", "WU/WB".</summary>
    private static string DecksText(List<int> group)
    {
        List<int> masks = [.. group.Select(at => DeckBuilder.Combinations[at])];
        int pairs = masks.Count(mask => int.PopCount(mask) == 2);
        int alone = masks.Count - pairs;
        if (pairs == 10 && alone == 5)
        {
            return "any deck";
        }

        if (pairs == 10 && alone == 0)
        {
            return "any pair";
        }

        if (pairs == 0 && alone == 5)
        {
            return "any one colour";
        }

        // Every pair with one colour in it, and that colour alone or not.
        for (int color = 0; color < 5; color++)
        {
            int with = 1 << color;
            if (pairs == 4 && masks.All(mask => (mask & with) != 0) && alone <= 1)
            {
                return $"{"WUBRG"[color]} pairs{(alone == 1 ? $" and {"WUBRG"[color]}" : string.Empty)}";
            }
        }

        return masks.Count <= 3
            ? string.Join("/", masks.Select(ColorText))
            : $"{masks.Count} decks ({string.Join("/", masks.Take(2).Select(ColorText))}...)";
    }

    /// <summary>Why a card adds nothing: no likely deck casts it, or every deck that does is better without it.</summary>
    private string NowhereText(Pick pick, DraftCard card)
    {
        int likeliest = Array.IndexOf(pick.Beliefs, pick.Beliefs.Max());
        int colors = DeckBuilder.Combinations[likeliest];
        if (card.IsLand)
        {
            return $"a land {ColorText(colors)} has no use for";
        }

        return card.OffColor(colors) switch
        {
            0 => $"no better than what {ColorText(colors)} has or expects",
            1 => $"a splash in {ColorText(colors)} at best, and not worth one",
            _ => $"off {ColorText(colors)}'s colours",
        };
    }

    /// <summary>
    /// What a card adds to one deck: what it plays as, what it takes the place of, what it does to the lands and the
    /// shape, and what it does to the rest of the deck - a fatty to the cheats, a land to a splash.
    /// </summary>
    private string InDeckText(Pick pick, int place, int at)
    {
        DraftCard card = pick.Pack[place];
        int colors = DeckBuilder.Combinations[at];
        var with = new Parts();
        var without = new Parts();
        double after = Project(pick.Pool, card, colors, pick.Expected[at], pick.Hope, with);
        double before = Project(pick.Pool, null, colors, pick.Expected[at], pick.Hope, without);

        List<string> parts = [];
        double rest = after - before;
        // A land is told by what it does to the lands and to the spells, below.
        if (double.IsNaN(with.Value) && !card.IsLand)
        {
            parts.Add("not played");
        }
        else if (!card.IsLand)
        {
            List<string> how = [];
            if (with.Splashed)
            {
                how.Add($"splashed {ColorText(1 << with.Splash).ToLowerInvariant()}");
            }

            if (with.Cast < 0.95)
            {
                how.Add($"cast {Percent(with.Cast)}");
            }

            if (with.Support < 0.99)
            {
                how.Add($"too little to cheat in, {Percent(with.Support)}");
            }

            if (with.Colored < 0.99)
            {
                how.Add($"makes none of the deck's colours, {Percent(with.Colored)}");
            }

            double synergy = DeckBuilder.Synergy * card.Speed * with.Lean;
            if (Math.Abs(synergy) >= 0.05)
            {
                how.Add($"{(synergy > 0 ? "suits" : "against")} its lean {Signed(synergy)}");
            }

            parts.Add($"plays as {One(with.Value)}{(how.Count > 0 ? $" ({string.Join(", ", how)})" : string.Empty)}");
            double instead = double.IsNaN(with.Out) ? 0 : with.Out;
            parts.Add(double.IsNaN(with.Out) ? "in a free slot"
                : with.OutAt < 0 ? $"in place of a pick to come {One(with.Out)}"
                : with.OutAt < pick.Pool.Count ? $"in place of {pick.Pool[with.OutAt].Name} {One(with.Out)}"
                : $"in a slot {One(with.Out)}");
            rest -= with.Value - instead;
        }

        double lands = with.Lands - without.Lands;
        rest -= lands;
        if (Math.Abs(lands) >= 0.05)
        {
            parts.Add($"{(!card.IsLand ? "lands" : card.Produces != 0 ? "fixing" : "a utility land")} {Signed(lands)}");
        }

        // The shape, part by part: what it costs the deck to be short of creatures or early plays, or over on five-drops.
        rest -= without.Shape - with.Shape;
        // Every spell counts towards the shape a deck of so many wants: a non-creature is a creature short, a spell of
        // four or less leaves room for one more five-drop.
        bool spell = !card.IsLand && !double.IsNaN(with.Value);
        double creatures = without.ShapeBy.Creatures - with.ShapeBy.Creatures;
        double early = without.ShapeBy.Cheap - with.ShapeBy.Cheap;
        double top = without.ShapeBy.Expensive - with.ShapeBy.Expensive;
        (string What, double Gain)[] shape =
        [
            (spell && card.IsCreature && creatures > 0 ? "a creature it wants" : spell && !card.IsCreature && creatures < 0 ? "not a creature" : "creatures", creatures),
            (spell && card.ManaValue <= 2 && early > 0 ? "an early play" : spell && card.ManaValue > 2 && early < 0 ? "not an early play" : "early plays", early),
            (spell && card.ManaValue >= 5 && top < 0 ? "a five-drop too many" : spell && card.ManaValue < 5 && top > 0 ? "room for its five-drops" : "five-drops", top),
        ];

        foreach ((string what, double gain) in shape.Where(part => Math.Abs(part.Gain) >= 0.05))
        {
            parts.Add($"{what} {Signed(gain)}");
        }

        // Everything else the card changes in the deck: the rest of its cards, worth more or less beside it.
        if (Math.Abs(rest) >= 0.05)
        {
            HashSet<int> had = [.. without.Played];
            List<string> joined = [.. with.Played.Where(index => index < pick.Pool.Count && !had.Contains(index)).Select(index => pick.Pool[index].Name)];
            bool cheats = pick.Pool.Any(other => other.Cheats != 0);
            string what = card.Payload != 0 && cheats ? "gives its cheats something to cheat"
                : joined.Count > 0 && rest > 0 ? $"lets in {string.Join(", ", joined)}"
                : card.IsLand ? "casts its spells better"
                : "the rest of the deck";
            parts.Add($"{what} {Signed(rest)}");
        }

        if (parts.Count == 0)
        {
            parts.Add("a land that changes nothing");
        }

        string sum = after < before - 1e-9 ? $"worse with it ({Signed(after - before)}): left out" : Signed(pick.With[place][at] - pick.Without[at]);
        return $"in {ColorText(colors)}: {string.Join("; ", parts)} = {sum}";
    }


    /// <summary>
    /// Which colours are read as open, in worth a pick above or below the usual - and what that was read from, each
    /// reading with the weight it is given: what was passed to this seat this pack and two packs before (when packs came
    /// the same way), what it passed on, and cards of the colour seen taken over better ones.
    /// </summary>
    private string? OpenText(int round)
    {
        static string Colors(Func<int, double> of, double least) =>
            string.Join(" ", Enumerable.Range(0, 5).Where(color => Math.Abs(of(color)) >= least).OrderByDescending(of).Select(color => $"{"WUBRG"[color]} {Signed(of(color))}"));

        double Open(int color) => Memory.Open(color, round, Tuning.Signals, Tuning.Carry, Tuning.Cut, Tuning.Crowd);
        string open = Colors(Open, 0.02);
        if (open.Length == 0)
        {
            return Memory.Sights.Count > 1 ? "nothing read yet" : null;
        }

        // Each reading, where it moved anything: its numbers, then its weight.
        List<string> from = [];
        void From(string what, double weight, Func<int, double> reading)
        {
            if (weight != 0 && Enumerable.Range(0, 5).Any(color => Math.Abs(weight * reading(color)) >= 0.01))
            {
                from.Add($"{what} {Colors(reading, 0.05)} (x{weight.ToString("0.##", Invariant)})");
            }
        }

        From("passed me this pack", Tuning.Signals, color => Memory.Flow(color, round));
        From($"pack {round - 2}", Tuning.Carry, color => Memory.Flow(color, round - 2));
        From("I passed", -Tuning.Cut, color => Memory.Passed(color, round - 1));
        From("taken over better", -Tuning.Crowd, Memory.Contested);
        return $"{open} <- {string.Join("; ", from)}";
    }

    /// <summary>What did not come back of a pack seen before, and which of it was taken over better cards: somebody's colour.</summary>
    private static string? WheelText(DraftMemory.Sight sight)
    {
        if (sight.Before is not { } before)
        {
            return null;
        }

        HashSet<string> back = [.. sight.Cards.Select(card => card.Name)];
        List<DraftCard> passed = [.. before.Passed];
        List<DraftCard> gone = [.. passed.Where(card => !back.Contains(card.Name))];
        HashSet<DraftCard> best = [.. passed.OrderByDescending(card => card.Power).Take(gone.Count)];
        List<DraftCard> over = [.. gone.Where(card => !best.Contains(card) && !card.IsLand && card.Colors != 0)];
        string text = $"pick {before.Pick}'s pack came back without {string.Join(", ", gone.Select(card => card.Name))}";
        return over.Count == 0 ? text : $"{text}; taken over better: {string.Join(", ", over.Select(card => $"{card.Name} ({ColorText(card.Colors)})"))}";
    }

    /// <summary>The lines that tell a grid pick: every line's cards and what the pool gains by them, and the one taken.</summary>
    private List<string> ExplainLine(DraftMessage grid, List<DraftCard> pool, Weighing weighing, List<(int Line, double Gain)> weighed, int chosen, Func<int, List<DraftCard>> cardsOf, Func<int, double> left)
    {
        static string Named(int line) => line < 3 ? $"row {line + 1}" : $"column {line - 2}";
        List<string> lines =
        [
            $"Grid {grid.PackNumber} of {grid.Packs}, {(grid.PickNumber == 1 ? "first" : "second")} pick, pool {pool.Count}. Took {Named(chosen)}: {string.Join(", ", cardsOf(chosen).Select(card => card.Name))}.",
        ];

        if (pool.Count > 0)
        {
            lines.Add($"  deck so far: {BeliefText(Believed(weighing.Without, weighing.Sureness))}");
        }

        bool denying = Tuning.Deny != 0 && grid.PickNumber == 1;
        foreach ((int line, double gain) in weighed.OrderByDescending(pair => pair.Gain).ThenBy(pair => pair.Line))
        {
            string cards = string.Join(", ", cardsOf(line).Select(card => $"{card.Name} {One(card.Worth)}"));
            string deny = denying ? $" (for me {Signed(gain + (Tuning.Deny * left(line)))}, less {One(left(line))} left to them x {One(Tuning.Deny)})" : string.Empty;
            lines.Add($"  {(line == chosen ? "->" : "  ")} {Named(line),-8} {Signed(gain),6}  {cards}{deny}");
        }

        return lines;
    }

    /// <summary>The lines that tell a Winston pile: what it adds, what passing it comes to, and which was the more.</summary>
    /// <param name="nextCards">How many cards passing is weighed as, from the next pile; zero for a card from the stack.</param>
    private List<string> ExplainPile(DraftMessage winston, Weighing? weighing, double pile, double stack, int nextCards, bool take)
    {
        string cards = string.Join(", ", winston.Pile.Select(view => Card(view)).Select(card => $"{card.Name} {One(card.Worth)}"));
        string head = $"Winston pile {winston.Looking + 1} of {winston.Piles.Count} ({Cards(winston.Pile.Count)}, stack {winston.Stack}, pool {winston.Pool.Count}): {(take ? "took it" : "passed it")}.";
        if (weighing is null)
        {
            return [head, winston.Pile.Count == 0 ? "  an empty pile" : $"  {cards}: the last pile cannot be passed"];
        }

        List<string> lines = [head, $"  pile {Signed(pile)}: {cards}"];
        string passing = nextCards > 0 ? $"the next pile, {nextCards} unseen cards" : "a card from the stack";
        lines.Add($"  passing {Signed(stack)}: {passing}");
        if (Math.Abs(pile - stack) < 1e-9)
        {
            lines.Add($"  neither adds anything: a pile of two or more is taken, and this is {Cards(winston.Pile.Count)}");
        }
        else if (Tuning.WinstonMargin != 0)
        {
            lines.Add($"  a pile is taken when it beats passing by {One(Tuning.WinstonMargin)}");
        }

        if (winston.Pool.Count > 0)
        {
            lines.Add($"  deck so far: {BeliefText(Believed(weighing.Without, weighing.Sureness))}");
        }

        return lines;
    }

    /// <summary>
    /// The lines that tell a deck: its colours, splash and lands and why that many, the decks that came closest, its
    /// shape and curve, and the cards that only just made it or only just did not.
    /// </summary>
    private List<string> ExplainDeck(List<DraftCard> pool, int minimum, BuiltDeck deck, List<BuiltDeck> tried)
    {
        int colors = deck.Colors | deck.Splash;
        List<int> spells = [.. deck.Main.Where(place => !pool[place].IsLand)];
        List<int> lands = [.. deck.Main.Where(place => pool[place].IsLand)];
        int landCount = lands.Count + deck.Basics.Values.Sum();
        List<string> lines = [$"Deck: {DeckText(deck.Colors, deck.Splash)}, {spells.Count} spells and {landCount} lands, worth {One(deck.Score)} (pool {pool.Count})."];

        // The decks that came closest, and the splash decision: the same colours with the best splash and with none.
        List<BuiltDeck> others = [.. tried.Where(other => other != deck).OrderByDescending(other => other.Score)];
        if (others.Count > 0)
        {
            lines.Add($"  next best: {string.Join(", ", others.Take(4).Select(other => $"{DeckText(other.Colors, other.Splash)} {One(other.Score)}"))}");
        }
        BuiltDeck? plain = tried.FirstOrDefault(other => other.Colors == deck.Colors && other.Splash == 0);
        BuiltDeck? splashed = tried.Where(other => other.Colors == deck.Colors && other.Splash != 0).MaxBy(other => other.Score);
        if (deck.Splash != 0)
        {
            string cards = string.Join(", ", spells.Where(place => pool[place].OffColor(deck.Colors) > 0).Select(place => pool[place].Name));
            lines.Add($"  splash {ColorText(deck.Splash).ToLowerInvariant()} for {cards}: {(plain is null ? "no deck without it" : $"{Signed(deck.Score - plain.Score)} over none")}, {One(deck.Sources[int.TrailingZeroCount(deck.Splash)])} sources");
        }
        else if (splashed is not null)
        {
            lines.Add($"  no splash: the best, {ColorText(splashed.Splash).ToLowerInvariant()}, would be {Signed(splashed.Score - deck.Score)}");
        }

        // The land count: Karsten's for the curve, less for cheap mana and draw and for free mana (ManaBase.LandsFor), as
        // the builder last worked it out; in the deck's own size.
        (double average, int cheap, int free, int wanted) = deck.LandsAsked;
        double scale = minimum / 40.0;
        double karsten = 2.0 / 3.0 * (19.59 + (1.9 * average)) * scale;
        List<string> count = [$"mana value {average.ToString("0.00", Invariant)} asks {One(karsten)}"];
        if (cheap > 0)
        {
            count.Add($"{cheap} cheap mana or draw -{One(0.28 * cheap * scale)}");
        }

        if (free > 0)
        {
            count.Add($"{free} free mana -{One(free * scale)}");
        }

        double asked = karsten - (((0.28 * cheap) + free) * scale);
        string kept = asked < (15 * scale) - 0.5 ? $", raised to the least, {One(15 * scale)}" : asked > (18 * scale) + 0.5 ? $", cut to the most, {One(18 * scale)}" : string.Empty;
        string tuned = Deck.Lands != 0 ? $", tuned {Deck.Lands.ToString("+0;-0", Invariant)}" : string.Empty;
        string made = landCount == wanted ? $": {landCount}"
            : landCount > wanted ? $": {wanted}, but {landCount}, the pool having only {spells.Count} spells to play"
            : $": {wanted}, then {landCount} once its spells were settled";
        lines.Add($"  lands: {string.Join(", ", count)} = {One(asked)}{kept}{tuned}{made}");

        IEnumerable<string> nonbasic = lands.Select(place => pool[place].Name);
        IEnumerable<string> basic = deck.Basics.Where(pair => pair.Value > 0).Select(pair => $"{pair.Value} {pair.Key}");
        string sources = string.Join(" ", Enumerable.Range(0, 5).Where(color => ((colors >> color) & 1) != 0).Select(color => $"{"WUBRG"[color]} {One(deck.Sources[color])}"));
        lines.Add($"  mana: {string.Join(", ", nonbasic.Concat(basic))}; sources {sources}");

        string lean = deck.Lean > 0.25 ? "early" : deck.Lean < -0.25 ? "late" : "neither way";
        lines.Add($"  shape: {spells.Count(place => pool[place].IsCreature)} creatures, {spells.Count(place => pool[place].ManaValue <= 2)} at 2 or less, {spells.Count(place => pool[place].ManaValue >= 5)} at 5 or more; leans {lean} ({Signed(deck.Lean)})");
        foreach (IGrouping<int, int> cost in spells.GroupBy(place => Math.Min(pool[place].ManaValue, 7)).OrderBy(group => group.Key))
        {
            lines.Add($"  {(cost.Key == 7 ? "7+" : cost.Key.ToString(Invariant)),2}: {string.Join(", ", cost.Select(place => pool[place].Name))}");
        }

        // Each spell's worth in this deck, as the builder weighed it: the last in and the first out.
        Payloads payloads = Payloads.Of(spells.Select(place => pool[place]));
        double Value(int place) => (DeckBuilder.PowerIn(pool[place], colors, payloads, Deck) * ManaBase.Castable(pool[place], deck.Sources, colors, minimum))
            + (DeckBuilder.Synergy * pool[place].Speed * deck.Lean);
        int splashColor = deck.Splash == 0 ? -1 : int.TrailingZeroCount(deck.Splash);
        HashSet<int> splashable = splashColor < 0 ? [] : [.. Enumerable.Range(0, pool.Count)
            .Where(place => !pool[place].IsLand && pool[place].OffColor(colors) == 0 && pool[place].Pips(splashColor, colors) == 1)
            .OrderByDescending(place => pool[place].Power)
            .Take(Deck.SplashCards)];
        List<int> outs = [.. Enumerable.Range(0, pool.Count)
            .Where(place => !pool[place].IsLand && !deck.Main.Contains(place) && pool[place].OffColor(colors) == 0
                && (splashColor < 0 || pool[place].Pips(splashColor, colors) == 0 || splashable.Contains(place)))
            .OrderByDescending(Value)];
        List<int> lastIn = [.. spells.OrderBy(Value).Take(3)];
        if (lastIn.Count > 0)
        {
            double lowest = Value(lastIn[0]);
            string Out(int place) => $"{pool[place].Name} {One(Value(place))}{(Value(place) > lowest ? " (for the shape)" : string.Empty)}";
            lines.Add($"  last in: {string.Join(", ", lastIn.Select(place => $"{pool[place].Name} {One(Value(place))}"))}{(outs.Count > 0 ? $"; first out: {string.Join(", ", outs.Take(3).Select(Out))}" : string.Empty)}");
        }

        // The best of the rest: good cards the colours could not cast.
        List<int> unplayable = [.. Enumerable.Range(0, pool.Count)
            .Where(place => !pool[place].IsLand && !deck.Main.Contains(place) && !outs.Contains(place) && pool[place].Power > CardWorth.Middling)
            .OrderByDescending(place => pool[place].Power)
            .Take(4)];
        if (unplayable.Count > 0)
        {
            lines.Add($"  best of the rest: {string.Join(", ", unplayable.Select(place => $"{pool[place].Name} ({ColorText(pool[place].Colors)}) {One(pool[place].Worth)}"))}");
        }

        return lines;
    }
}
