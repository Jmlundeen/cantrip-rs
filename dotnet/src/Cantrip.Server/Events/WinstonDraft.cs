using Cantrip.Core.Random;

namespace Cantrip.Server.Events;

/// <summary>
/// A Winston draft: the cards shuffled face down into a stack, and a few piles - three, classically - each started
/// with one card from it. On a turn a player looks at the first pile and takes it or passes it. Taking ends the turn,
/// and the pile is started again with a card from the stack. Passing puts a card from the stack on that pile and moves
/// on to the next; passing the last pile takes the top card of the stack instead, unseen by anyone else. The draft is
/// over when every card has been taken.
/// <para>
/// Once the stack is empty a passed pile gets nothing, and an empty pile is never looked at. A pile can be passed only
/// while something is left after it - a later pile with cards in it, or the stack to draw from - so every turn takes
/// at least one card and the draft always ends. Passing the last pile with one card left in the stack takes that card
/// rather than adding it to the pile: there is nothing else to take.
/// </para>
/// <para>
/// The draft's own state: the stack, the piles, whose turn it is, and what each player has taken
/// (<see cref="Pools"/>) - all of it in one file, saved as it stands after every pick, so a pick is never half saved.
/// The event copies the pools to its participants (<see cref="EventParticipant.Pool"/>), where a pool belongs however
/// it was drafted, taking the draft's as the authority while the draft runs.
/// </para>
/// </summary>
public sealed class WinstonDraft : IPoolDraft
{
    /// <summary>How many players draft, taking turns from seat 1 upwards and round again.</summary>
    public int Seats { get; set; } = 2;

    /// <summary>The cards still face down, top first.</summary>
    public List<string> Stack { get; set; } = [];

    /// <summary>The piles in the order they are looked at, each a list of cards.</summary>
    public List<List<string>> Piles { get; set; } = [];

    /// <summary>Whose turn it is, by seat; zero once the draft is over.</summary>
    public int Active { get; set; }

    /// <summary>The pile the active player is looking at, from zero; -1 once the draft is over.</summary>
    public int Looking { get; set; } = -1;

    /// <summary>
    /// How many decisions have been made: one per take or pass. A player's choice names the decision it answers, so a
    /// click sent twice, or sent from a second window on a position that has moved on, is refused rather than applied
    /// to the next pile.
    /// </summary>
    public int Decision { get; set; }

    /// <summary>Every take and pass, in order: public, since every player sees which pile was taken and how big it was.</summary>
    public List<WinstonAction> History { get; set; } = [];

    /// <summary>What each seat has taken, in the order it came: seat 1's first.</summary>
    public List<List<string>> Pools { get; set; } = [];

    /// <summary>
    /// The cube's cards best first, as far as this draft's cards go (<see cref="CubeList.PickOrder"/>): what a seat with
    /// no ideas of its own (a bot) is told the cards of the pile it is looking at stand at. Empty for a cube without one.
    /// </summary>
    public List<string> PickOrder { get; set; } = [];

    /// <summary>Where a card stands in <see cref="PickOrder"/>, from 0; past the end for a card it does not name.</summary>
    public int Rank(string card)
    {
        int rank = PickOrder.IndexOf(card);
        return rank < 0 ? PickOrder.Count : rank;
    }

    /// <summary>A seat's pool.</summary>
    public IReadOnlyList<string> PoolOf(int seat) => seat >= 1 && seat <= Pools.Count ? Pools[seat - 1] : [];

    public bool IsOver => Active == 0;

    /// <summary>Lays out a new draft: the cards shuffled into a stack and a card from it on each pile.</summary>
    /// <param name="cards">The cards to draft, in any order; they are shuffled here.</param>
    /// <param name="first">The seat that looks first.</param>
    /// <param name="pickOrder">The cube's cards best first (<see cref="CubeList.PickOrder"/>), or none.</param>
    public static WinstonDraft Start(IReadOnlyList<string> cards, int piles, int seats, int first, IRandom random, IReadOnlyList<string>? pickOrder = null)
    {
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(random);
        if (piles is < 1 or > 8) throw new ArgumentOutOfRangeException(nameof(piles), "A Winston draft has one to eight piles.");
        if (seats is < 2 or > 8) throw new ArgumentOutOfRangeException(nameof(seats), "A Winston draft needs two to eight players.");
        if (first < 1 || first > seats) throw new ArgumentOutOfRangeException(nameof(first));
        if (cards.Count < piles) throw new ArgumentException($"A draft with {piles} piles needs at least {piles} cards.", nameof(cards));

        List<string> stack = [.. cards];
        for (int i = stack.Count - 1; i > 0; i--)
        {
            int j = random.NextInt(i + 1);
            (stack[i], stack[j]) = (stack[j], stack[i]);
        }

        HashSet<string> dealt = [.. stack];
        var draft = new WinstonDraft
        {
            Seats = seats,
            Stack = stack,
            Active = first,
            PickOrder = [.. (pickOrder ?? []).Where(dealt.Remove)],
            Pools = [.. Enumerable.Range(0, seats).Select(_ => new List<string>())],
        };
        for (int i = 0; i < piles; i++)
        {
            draft.Piles.Add([draft.Draw()!]);
        }

        draft.Looking = draft.NextPile(0);
        return draft;
    }

    /// <summary>Whether the active player may pass the pile they are looking at.</summary>
    public bool CanPass => !IsOver && (Stack.Count > 0 || NextPile(Looking + 1) >= 0);

    /// <summary>
    /// The active player takes the pile they are looking at. It is started again with a card from the stack, and the
    /// turn passes on.
    /// </summary>
    /// <returns>The cards taken.</returns>
    public IReadOnlyList<string> Take(int seat, int decision)
    {
        Check(seat, decision);
        int pile = Looking;
        List<string> taken = Piles[pile];
        Piles[pile] = Draw() is { } fresh ? [fresh] : [];
        History.Add(new WinstonAction { Seat = seat, Decision = Decision, Pile = pile, Took = taken.Count });
        Pools[seat - 1].AddRange(taken);
        EndTurn();
        return taken;
    }

    /// <summary>
    /// The active player passes the pile they are looking at: a card from the stack goes onto it, and they look at the
    /// next. Passing the last pile there is to look at takes the top card of the stack instead, and ends the turn.
    /// </summary>
    /// <returns>The card drawn from the stack when the last pile was passed; null when the turn goes on.</returns>
    public string? Pass(int seat, int decision)
    {
        Check(seat, decision);
        if (!CanPass) throw new InvalidOperationException("This is the last of the cards; it has to be taken.");

        int pile = Looking;
        int next = NextPile(pile + 1);

        // The last pile passed with one card left in the stack: that card is what is taken, not added to the pile.
        if ((next >= 0 || Stack.Count >= 2) && Draw() is { } added)
        {
            Piles[pile].Add(added);
        }

        if (next >= 0)
        {
            History.Add(new WinstonAction { Seat = seat, Decision = Decision, Pile = pile });
            Decision++;
            Looking = next;
            return null;
        }

        // Passed the last pile: the top of the stack, which CanPass promised is there.
        string blind = Draw() ?? throw new InvalidOperationException("Nothing is left in the stack.");
        History.Add(new WinstonAction { Seat = seat, Decision = Decision, Pile = pile, Took = 1, Blind = true });
        Pools[seat - 1].Add(blind);
        EndTurn();
        return blind;
    }

    /// <summary>
    /// How many turns have ended: what the pick clock counts, since a turn - every pile looked at, to the one taken -
    /// is on one clock.
    /// </summary>
    public int Turns => History.Count(action => action.Took > 0);

    /// <summary>
    /// The pick clock has run out on the active player: the rest of their turn is played for them - a pile of two or more
    /// cards taken, a pile of one passed for the next, and the last taken when there is no passing it.
    /// </summary>
    public void TakeForClock()
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        int seat = Active;
        while (Active == seat && !IsOver)
        {
            int before = History.Count;
            if (Piles[Looking].Count >= 2 || !CanPass) Take(seat, Decision);
            else Pass(seat, Decision);
            History[before].Auto = true;
        }
    }

    private void Check(int seat, int decision)
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        if (seat != Active) throw new InvalidOperationException("It is not this player's turn to pick.");
        if (decision != Decision) throw new InvalidOperationException("That pick was for a pile that has since changed.");
    }

    private void EndTurn()
    {
        Decision++;
        if (Stack.Count == 0 && Piles.All(pile => pile.Count == 0))
        {
            Active = 0;
            Looking = -1;
            return;
        }

        Active = (Active % Seats) + 1;
        Looking = NextPile(0);
    }

    private string? Draw()
    {
        if (Stack.Count == 0)
        {
            return null;
        }

        string top = Stack[0];
        Stack.RemoveAt(0);
        return top;
    }

    /// <summary>The first pile from <paramref name="from"/> on that has cards in it, or -1.</summary>
    private int NextPile(int from)
    {
        for (int i = Math.Max(0, from); i < Piles.Count; i++)
        {
            if (Piles[i].Count > 0)
            {
                return i;
            }
        }

        return -1;
    }
}

/// <summary>
/// One decision in a Winston draft. <see cref="Took"/> is zero for a pass that moved on to the next pile; for a take it
/// is how many cards the pile had, and for the last pile passed it is the one card taken blind from the stack.
/// </summary>
public sealed class WinstonAction
{
    public int Seat { get; set; }
    public int Decision { get; set; }
    public int Pile { get; set; }
    public int Took { get; set; }
    public bool Blind { get; set; }

    /// <summary>Whether the pick clock made this decision, the player having run out of time.</summary>
    public bool Auto { get; set; }
}
