using Cantrip.Core.Random;

namespace Cantrip.Server.Events;

/// <summary>
/// A grid draft, for two: nine cards laid out face up in a three-by-three grid - a pack. The player who picks first
/// takes a row or a column of it, three cards; the other takes a row or a column of what is left, which may be only
/// two cards now; the rest of the grid is discarded. Then the next grid is laid out, and the other player picks first.
/// The draft is over when the last pack's grid has been picked from twice - eighteen packs, by default.
/// <para>
/// The draft's own state: the cards still to be laid out, the grid, whose pick it is, and what each player has taken
/// (<see cref="Pools"/>) - all of it in one file, saved as it stands after every pick, as a Winston draft's is.
/// Everything is face up: both players see the grid and what each takes from it.
/// </para>
/// </summary>
public sealed class GridDraft : IPoolDraft
{
    /// <summary>How many cards a side of the grid has.</summary>
    public const int Side = 3;

    /// <summary>How many cards a grid - a pack - has.</summary>
    public const int PackSize = Side * Side;

    /// <summary>How many lines there are to take: the rows, from the top, then the columns, from the left.</summary>
    public const int Lines = Side * 2;

    /// <summary>A grid draft is for two.</summary>
    public int Seats { get; set; } = 2;

    /// <summary>How many packs the draft lays out, one grid each.</summary>
    public int Packs { get; set; }

    /// <summary>The grid being picked from, from 1; zero once the draft is over.</summary>
    public int Pack { get; set; }

    /// <summary>The cards of the grids still to be laid out, the next grid's first, nine a grid.</summary>
    public List<string> Stack { get; set; } = [];

    /// <summary>The grid's nine places, row by row from the top left; a place whose card has been taken is null.</summary>
    public List<string?> Grid { get; set; } = [];

    /// <summary>The seat that picks first from this grid.</summary>
    public int First { get; set; }

    /// <summary>Whose pick it is, by seat; zero once the draft is over.</summary>
    public int Active { get; set; }

    /// <summary>
    /// How many picks have been made. A player's choice names the decision it answers, so a click sent twice is refused
    /// rather than taken from the next grid.
    /// </summary>
    public int Decision { get; set; }

    /// <summary>
    /// The cube's cards best first, as far as this draft's cards go (<see cref="CubeList.PickOrder"/>): what a seat with
    /// no ideas of its own (a bot) is told the cards of the grid stand at. Empty for a cube without one.
    /// </summary>
    public List<string> PickOrder { get; set; } = [];

    /// <summary>Every pick, in order: public, as the grid is.</summary>
    public List<GridPick> History { get; set; } = [];

    /// <summary>The cards left in each grid once both players had picked from it, which nobody drafts.</summary>
    public List<string> Discarded { get; set; } = [];

    /// <summary>What each seat has taken, in the order it came: seat 1's first.</summary>
    public List<List<string>> Pools { get; set; } = [];

    /// <summary>A seat's pool.</summary>
    public IReadOnlyList<string> PoolOf(int seat) => seat >= 1 && seat <= Pools.Count ? Pools[seat - 1] : [];

    public bool IsOver => Active == 0;

    /// <summary>Which pick of this grid is being made: 1, or 2 once the first player has taken their line.</summary>
    public int PickOfPack => Active == First ? 1 : 2;

    /// <summary>Lays out a new draft: the cards shuffled, and the first grid laid out from them.</summary>
    /// <param name="cards">The cards to draft, nine a pack, in any order; they are shuffled here.</param>
    /// <param name="first">The seat that picks first from the first grid; the other picks first from the second.</param>
    /// <param name="pickOrder">The cube's cards best first (<see cref="CubeList.PickOrder"/>), or none.</param>
    public static GridDraft Start(IReadOnlyList<string> cards, int packs, int first, IRandom random, IReadOnlyList<string>? pickOrder = null)
    {
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(random);
        if (packs < 1) throw new ArgumentOutOfRangeException(nameof(packs), "A grid draft lays out at least one grid.");
        if (first is < 1 or > 2) throw new ArgumentOutOfRangeException(nameof(first));
        if (cards.Count < packs * PackSize) throw new ArgumentException($"A grid draft of {packs} packs needs {packs * PackSize} cards.", nameof(cards));

        List<string> stack = [.. cards];
        for (int i = stack.Count - 1; i > 0; i--)
        {
            int j = random.NextInt(i + 1);
            (stack[i], stack[j]) = (stack[j], stack[i]);
        }

        stack.RemoveRange(packs * PackSize, stack.Count - (packs * PackSize));
        HashSet<string> dealt = [.. stack];
        var draft = new GridDraft
        {
            Packs = packs,
            Stack = stack,
            First = first,
            PickOrder = [.. (pickOrder ?? []).Where(dealt.Remove)],
            Pools = [[], []],
        };
        draft.LayOut();
        return draft;
    }

    /// <summary>The places in the grid of a line: a row (0 to 2, from the top) or a column (3 to 5, from the left).</summary>
    public static IReadOnlyList<int> Places(int line)
    {
        if (line is < 0 or >= Lines) throw new ArgumentOutOfRangeException(nameof(line), "A line is a row or a column of the grid.");
        return line < Side
            ? [.. Enumerable.Range(0, Side).Select(column => (line * Side) + column)]
            : [.. Enumerable.Range(0, Side).Select(row => (row * Side) + (line - Side))];
    }

    /// <summary>A line in words: "row 2", "column 3".</summary>
    public static string Named(int line) => line < Side ? $"row {line + 1}" : $"column {line - Side + 1}";

    /// <summary>The cards a line has left in it.</summary>
    public IReadOnlyList<string> CardsIn(int line) => [.. Places(line).Select(place => Grid[place]).OfType<string>()];

    /// <summary>Whether a line can be taken now: one with a card left in it, while the draft is on.</summary>
    public bool CanTake(int line) => !IsOver && line is >= 0 and < Lines && CardsIn(line).Count > 0;

    /// <summary>
    /// The active player takes a row or a column of the grid. The first pick of a grid passes the turn to the other
    /// player; the second discards what is left and lays out the next grid, for the other player to pick from first.
    /// </summary>
    /// <returns>The cards taken, in the line's order.</returns>
    public IReadOnlyList<string> Take(int seat, int decision, int line) => Take(seat, decision, line, auto: false);

    private IReadOnlyList<string> Take(int seat, int decision, int line, bool auto)
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        if (seat != Active) throw new InvalidOperationException("It is not this player's turn to pick.");
        if (decision != Decision) throw new InvalidOperationException("That pick was for a grid that has since changed.");
        if (line is < 0 or >= Lines) throw new InvalidOperationException("That is not a row or a column of the grid.");
        if (CardsIn(line).Count == 0) throw new InvalidOperationException($"There is nothing left in {Named(line)}.");

        List<string> taken = [];
        foreach (int place in Places(line))
        {
            if (Grid[place] is { } card)
            {
                taken.Add(card);
                Grid[place] = null;
            }
        }

        History.Add(new GridPick { Seat = seat, Decision = Decision, Pack = Pack, Line = line, Cards = [.. taken], Auto = auto });
        Pools[seat - 1].AddRange(taken);
        Decision++;

        if (Active == First)
        {
            Active = Other(First);
            return taken;
        }

        Discarded.AddRange(Grid.OfType<string>());
        First = Other(First);
        LayOut();
        return taken;
    }

    /// <summary>
    /// The pick clock has run out on the active player: the line <see cref="ClockLine"/> chooses is taken for them.
    /// </summary>
    /// <returns>The cards taken.</returns>
    public IReadOnlyList<string> TakeForClock()
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        return Take(Active, Decision, ClockLine(), auto: true);
    }

    /// <summary>
    /// The line the pick clock takes: the one whose cards' squared pick-order ranks (<see cref="Rank"/>, from 1) are
    /// lowest on average, so a poor card costs a line more than a good one earns it. A place whose card is gone counts as
    /// the worst card left on the grid, so a short line is not taken for being short. The first among equals.
    /// </summary>
    public int ClockLine()
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        long worst = Grid.OfType<string>().Max(card => (long)Rank(card) + 1);
        double Average(int line) => Places(line).Average(place => Grid[place] is { } card ? Squared(Rank(card) + 1) : Squared(worst));
        return Enumerable.Range(0, Lines).Where(CanTake).OrderBy(Average).ThenBy(line => line).First();

        static double Squared(long rank) => (double)rank * rank;
    }

    /// <summary>Lays out the next grid from the stack, for <see cref="First"/> to pick from; or ends the draft with none left.</summary>
    private void LayOut()
    {
        if (Stack.Count < PackSize)
        {
            Discarded.AddRange(Stack);
            Stack.Clear();
            Grid = [];
            Pack = 0;
            Active = 0;
            return;
        }

        Grid = [.. Stack.Take(PackSize)];
        Stack.RemoveRange(0, PackSize);
        Pack++;
        Active = First;
    }

    private static int Other(int seat) => seat == 1 ? 2 : 1;

    /// <summary>Where a card stands in <see cref="PickOrder"/>, from 0; past the end for a card it does not name.</summary>
    public int Rank(string card)
    {
        int rank = PickOrder.IndexOf(card);
        return rank < 0 ? PickOrder.Count : rank;
    }
}

/// <summary>One pick of a grid draft: who took which line of which grid, and the cards that were in it.</summary>
public sealed class GridPick
{
    public int Seat { get; set; }
    public int Decision { get; set; }
    public int Pack { get; set; }

    /// <summary>The line taken: a row from 0, or a column from 3 (<see cref="GridDraft.Places"/>).</summary>
    public int Line { get; set; }

    public List<string> Cards { get; set; } = [];

    /// <summary>Whether the pick clock took this line, the player having run out of time.</summary>
    public bool Auto { get; set; }
}
