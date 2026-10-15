using Cantrip.Cards.Model;
using Cantrip.Core.Random;

namespace Cantrip.Server.Events;

/// <summary>
/// A booster draft, as a cube is drafted at a table of eight: every player opens a pack, takes a card from it and passes
/// the rest to their neighbour, and takes from each pack passed to them in turn until the packs are empty; then everyone
/// opens their next pack and passes it the other way. Three packs of fifteen, classically.
/// <para>
/// Nobody waits for the whole table. A pack passed to a player who is still picking from another waits for them, in
/// the order it came (<see cref="Queues"/>), so a slow player has packs piling up while everyone else carries on - which
/// every seat sees. Only opening the next pack waits for everyone: it is opened when every pack of the last one is
/// empty.
/// </para>
/// <para>
/// Every pick is on a clock that gets shorter as the pack does (<see cref="SecondsFor"/>), started when the pack reaches
/// the front of its player's queue. A clock that runs out takes a card from the pack for its player - the best by the
/// cube's pick order (<see cref="PickOrder"/>), so a player who has gone does not pass the table the cards they would
/// have taken, or at random without one - unless whoever runs the draft holds it a moment longer, to ask the player for
/// the card they had chosen. The last card of a pack is nobody's choice, so its clock is short and nobody is asked for
/// it (<see cref="LastCardSeconds"/>): long enough to see what it is, and see it go into the pool.
/// </para>
/// <para>
/// A card drafted earlier can change a pick: Cogwork Librarian's draft ability (<see cref="AdditionalPicks"/>) lets
/// its drafter take a second card from the pack in front of them, and it goes into that pack in their place.
/// </para>
/// <para>
/// Every pick is kept (<see cref="Log"/>): who took what, from which pack, what else was in it, and whether the clock
/// took it - the draft log, in order for each player. The draft's own state, in one file, saved as it stands after every
/// pick, as <see cref="WinstonDraft"/> is.
/// </para>
/// </summary>
public sealed class BoosterDraft
{
    /// <summary>How many players draft, seated round the table from 1.</summary>
    public int Seats { get; set; } = 8;

    /// <summary>How many cards a pack starts with.</summary>
    public int PackSize { get; set; } = 15;

    /// <summary>How many packs each player opens.</summary>
    public int Packs { get; set; } = 3;

    /// <summary>The packs not opened yet: for each pack number from 1, each seat's pack, seat 1's first.</summary>
    public List<List<List<string>>> Unopened { get; set; } = [];

    /// <summary>The pack being drafted, from 1; zero once the draft is over.</summary>
    public int Pack { get; set; }

    /// <summary>For each seat, seat 1's first, the packs waiting for it to pick from, the one it is picking from first.</summary>
    public List<List<BoosterPack>> Queues { get; set; } = [];

    /// <summary>
    /// When each seat's clock on the pack in front of it runs out, seat 1's first; null for a seat with nothing to pick
    /// from.
    /// </summary>
    public List<DateTimeOffset?> Deadlines { get; set; } = [];

    /// <summary>
    /// How many picks each seat has made from the packs in front of it, seat 1's first. A pick names the one it is, so a click sent twice, or from a second window on a pack
    /// that has since gone, is refused rather than taken from the next pack.
    /// </summary>
    public List<int> Picks { get; set; } = [];

    /// <summary>What each seat has taken, in the order it came: seat 1's first.</summary>
    public List<List<string>> Pools { get; set; } = [];

    /// <summary>Every pick, in the order made.</summary>
    public List<BoosterPick> Log { get; set; } = [];

    /// <summary>Where the clock's random picks come from, with the seat and pick they are for, so each can be followed.</summary>
    public ulong Seed { get; set; }

    /// <summary>
    /// The cards dealt, best first by the cube's pick order: what the clock takes. A card it does not name ranks below
    /// every card it does, and the clock takes among equals at random - so with none, every pick of the clock's is.
    /// </summary>
    public List<string> PickOrder { get; set; } = [];

    /// <summary>
    /// The cards dealt with Cogwork Librarian's draft ability (<see cref="DraftAbility.AdditionalPickPutThisIntoPack"/>):
    /// one in a seat's pool may be put into the pack in front of it for an additional card from that pack.
    /// </summary>
    public List<string> AdditionalPicks { get; set; } = [];

    /// <summary>The seconds on the clock for each pick, by the cards left in the pack: five a card, but for the last card.</summary>
    public int SecondsPerCard { get; set; } = 5;

    /// <summary>The clock on the last card of a pack: nobody chooses it, but its player sees it before the clock takes it.</summary>
    public const int LastCardSeconds = 2;

    public bool IsOver => Pack == 0;

    /// <summary>A seat's pool.</summary>
    public IReadOnlyList<string> PoolOf(int seat) => seat >= 1 && seat <= Pools.Count ? Pools[seat - 1] : [];

    /// <summary>The packs waiting for a seat, the one it is picking from first.</summary>
    public IReadOnlyList<BoosterPack> QueueOf(int seat) => seat >= 1 && seat <= Queues.Count ? Queues[seat - 1] : [];

    /// <summary>The pack a seat is picking from now, or null while it waits for one.</summary>
    public BoosterPack? Current(int seat) => QueueOf(seat) is [var first, ..] ? first : null;

    /// <summary>Which way packs go in the pack being drafted: +1 to the next seat up (pack 1 and 3), -1 down (pack 2).</summary>
    public int Direction => Pack % 2 == 0 ? -1 : 1;

    /// <summary>The seat a pack goes to from <paramref name="seat"/>, the way packs are going now.</summary>
    public int PassesTo(int seat) => ((seat - 1 + Direction + Seats) % Seats) + 1;

    /// <summary>How long a pick from a pack of this many cards is given.</summary>
    public TimeSpan SecondsFor(int cards) => TimeSpan.FromSeconds(cards <= 1 ? LastCardSeconds : cards * SecondsPerCard);

    /// <summary>
    /// Deals the packs: the cards shuffled and dealt out, <paramref name="packSize"/> to a pack, one pack per seat per
    /// pack number, and the first packs opened.
    /// </summary>
    /// <param name="cards">At least seats * packs * packSize cards, in any order; the ones dealt are chosen at random.</param>
    /// <param name="pickOrder">The cube's cards best first (<see cref="CubeList.PickOrder"/>), or none.</param>
    /// <param name="additionalPicks">The cube's cards that may be spent for an additional pick (<see cref="AdditionalPicks"/>).</param>
    public static BoosterDraft Start(IReadOnlyList<string> cards, int seats, int packs, int packSize, IRandom random, ulong seed, DateTimeOffset now, int secondsPerCard = 5, IReadOnlyList<string>? pickOrder = null, IReadOnlyCollection<string>? additionalPicks = null)
    {
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(random);
        if (seats is < 2 or > 16) throw new ArgumentOutOfRangeException(nameof(seats), "A booster draft needs two to sixteen players.");
        if (packs is < 1 or > 6) throw new ArgumentOutOfRangeException(nameof(packs), "A booster draft has one to six packs.");
        if (packSize is < 2 or > 30) throw new ArgumentOutOfRangeException(nameof(packSize), "A pack has two to thirty cards.");
        if (secondsPerCard is < 1 or > 60) throw new ArgumentOutOfRangeException(nameof(secondsPerCard));
        int needed = seats * packs * packSize;
        if (cards.Count < needed) throw new ArgumentException($"{seats} players opening {packs} packs of {packSize} need {needed} cards; there are {cards.Count}.", nameof(cards));

        List<string> deck = [.. cards];
        random.Shuffle(deck);

        var draft = new BoosterDraft
        {
            Seats = seats,
            Packs = packs,
            PackSize = packSize,
            Seed = seed,
            SecondsPerCard = secondsPerCard,
            Queues = [.. Enumerable.Range(0, seats).Select(_ => new List<BoosterPack>())],
            Deadlines = [.. Enumerable.Range(0, seats).Select(_ => (DateTimeOffset?)null)],
            Picks = [.. Enumerable.Repeat(0, seats)],
            Pools = [.. Enumerable.Range(0, seats).Select(_ => new List<string>())],
        };

        int next = 0;
        for (int pack = 0; pack < packs; pack++)
        {
            List<List<string>> round = [];
            for (int seat = 0; seat < seats; seat++)
            {
                round.Add(deck.GetRange(next, packSize));
                next += packSize;
            }

            draft.Unopened.Add(round);
        }

        HashSet<string> dealt = [.. draft.Unopened.SelectMany(round => round.SelectMany(pack => pack))];
        draft.AdditionalPicks = [.. (additionalPicks ?? []).Where(dealt.Contains).Distinct(StringComparer.Ordinal)];
        draft.PickOrder = [.. (pickOrder ?? []).Where(dealt.Remove)];
        draft.Open(now);
        return draft;
    }

    /// <summary>
    /// A seat takes a card from the pack in front of it - by its place in the pack - and the rest of the pack goes on to
    /// its neighbour.
    /// </summary>
    /// <param name="pick">How many picks the seat has made before this one (<see cref="Picks"/>).</param>
    /// <returns>The card taken.</returns>
    public string Take(int seat, int pick, int place, DateTimeOffset now) => Take(seat, pick, [place], now)[0];

    /// <summary>
    /// A seat takes cards from the pack in front of it - the first its pick, and each after it an additional card, for
    /// which a card of its pool with Cogwork Librarian's draft ability (<see cref="Spendable"/>) goes into the pack - and
    /// the rest of the pack goes on to its neighbour.
    /// </summary>
    /// <param name="pick">How many picks the seat has made before this one (<see cref="Picks"/>).</param>
    /// <param name="places">The cards' places in the pack, the pick first.</param>
    /// <returns>The cards taken, in the order of <paramref name="places"/>.</returns>
    public IReadOnlyList<string> Take(int seat, int pick, IReadOnlyList<int> places, DateTimeOffset now)
    {
        ArgumentNullException.ThrowIfNull(places);
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        if (seat < 1 || seat > Seats) throw new InvalidOperationException("There is no such seat.");
        if (pick != Picks[seat - 1]) throw new InvalidOperationException("That pick was from a pack that has since gone on.");
        if (Current(seat) is not { } pack) throw new InvalidOperationException("There is no pack to pick from yet.");
        if (places.Count == 0) throw new InvalidOperationException("Choose a card to take.");
        if (places.Any(place => place < 0 || place >= pack.Cards.Count)) throw new InvalidOperationException("That card is not in the pack.");
        if (places.Distinct().Count() != places.Count) throw new InvalidOperationException("A card can be taken only once.");
        if (places.Count - 1 > Spendable(seat).Count) throw new InvalidOperationException("There is nothing in your pool to draft an additional card with.");
        return TakeFrom(seat, places, auto: false, now);
    }

    /// <summary>
    /// The places in a seat's pool of the cards it may put into the pack in front of it for an additional card from it
    /// (<see cref="AdditionalPicks"/>), in the order they would go: the last drafted first. None while it has no pack, or
    /// a pack of one card, which has nothing to take a second card of.
    /// </summary>
    public IReadOnlyList<int> Spendable(int seat)
    {
        if (Current(seat) is not { Cards.Count: > 1 } || AdditionalPicks.Count == 0) return [];
        List<string> pool = Pools[seat - 1];
        return [.. Enumerable.Range(0, pool.Count).Reverse().Where(place => AdditionalPicks.Contains(pool[place]))];
    }

    /// <summary>
    /// Lets time pass: every seat whose clock has run out has a card taken for it from the pack in front of it
    /// (<see cref="TakeForClock(int, int, DateTimeOffset)"/>), unless <paramref name="holding"/> says to wait for it. A
    /// seat that is passed a pack with a clock that has already run out - which happens only to a seat whose own clock ran
    /// out at the same moment - is given its full time on it.
    /// </summary>
    /// <param name="holding">Asked of each seat whose clock has run out, before its card is taken: true waits for it.</param>
    /// <returns>Whether anything was taken.</returns>
    public bool Tick(DateTimeOffset now, Func<int, bool>? holding = null)
    {
        bool took = false;
        for (int seat = 1; seat <= Seats && !IsOver; seat++)
        {
            if (RanOut(seat, now) && holding?.Invoke(seat) != true)
            {
                TakeForClock(seat, now);
                took = true;
            }
        }

        return took;
    }

    /// <summary>Whether the clock on the pack in front of a seat has run out.</summary>
    public bool RanOut(int seat, DateTimeOffset now) =>
        seat >= 1 && seat <= Seats && Deadlines[seat - 1] is { } deadline && now >= deadline && Current(seat) is not null;

    /// <summary>
    /// Takes a card for a seat whose clock has run out, as its clock would - the pack's best by <see cref="PickOrder"/>,
    /// at random among equals: the player was asked for the card they had chosen and said they had none.
    /// </summary>
    /// <param name="pick">How many picks the seat has made before this one (<see cref="Picks"/>).</param>
    /// <returns>The card taken.</returns>
    public string TakeForClock(int seat, int pick, DateTimeOffset now)
    {
        if (IsOver) throw new InvalidOperationException("The draft is over.");
        if (seat < 1 || seat > Seats) throw new InvalidOperationException("There is no such seat.");
        if (pick != Picks[seat - 1]) throw new InvalidOperationException("That pick was from a pack that has since gone on.");
        if (!RanOut(seat, now)) throw new InvalidOperationException("Choose a card to take.");
        return TakeForClock(seat, now);
    }

    /// <summary>
    /// Starts every clock again from now, as the draft is taken up after a restart: time the table was down for is
    /// nobody's, and a clock that ran out while it was would take a card nobody had the chance to choose.
    /// </summary>
    public void Resume(DateTimeOffset now)
    {
        for (int seat = 1; seat <= Seats; seat++)
        {
            Deadlines[seat - 1] = Current(seat) is { } pack ? now + SecondsFor(pack.Cards.Count) : null;
        }
    }

    /// <summary>How long a seat has left on the pack in front of it, or null while it waits for one.</summary>
    public TimeSpan? Left(int seat, DateTimeOffset now) =>
        seat >= 1 && seat <= Seats && Deadlines[seat - 1] is { } deadline ? (deadline - now < TimeSpan.Zero ? TimeSpan.Zero : deadline - now) : null;

    /// <summary>Where a card stands in <see cref="PickOrder"/>, from 0; past the end for a card it does not name.</summary>
    public int Rank(string card)
    {
        int rank = PickOrder.IndexOf(card);
        return rank < 0 ? PickOrder.Count : rank;
    }

    private string TakeForClock(int seat, DateTimeOffset now)
    {
        List<string> cards = Current(seat)!.Cards;
        int best = cards.Min(Rank);
        List<int> places = [.. Enumerable.Range(0, cards.Count).Where(place => Rank(cards[place]) == best)];
        var random = new Pcg32Random(unchecked(Seed ^ ((ulong)seat * 0x9E3779B97F4A7C15UL) ^ ((ulong)Picks[seat - 1] * 0xC2B2AE3D27D4EB4FUL)));
        return TakeFrom(seat, [places[random.NextInt(places.Count)]], auto: true, now)[0];
    }

    private List<string> TakeFrom(int seat, IReadOnlyList<int> places, bool auto, DateTimeOffset now)
    {
        List<BoosterPack> queue = Queues[seat - 1];
        BoosterPack pack = queue[0];
        List<string> pool = Pools[seat - 1];
        List<string> offered = [.. pack.Cards];
        List<string> cards = [.. places.Select(place => offered[place])];

        // What pays for each additional card leaves the pool before anything joins it: a Librarian taken in this pick
        // was not drafted yet when the pick was made.
        List<int> spending = [.. Spendable(seat).Take(places.Count - 1)];
        List<string> spent = [.. spending.Select(place => pool[place])];
        foreach (int place in spending.Order().Reverse())
        {
            pool.RemoveAt(place);
        }

        for (int i = 0; i < cards.Count; i++)
        {
            Log.Add(new BoosterPick
            {
                Seat = seat,
                Pack = Pack,
                Pick = PackSize - offered.Count + 1,
                Opener = pack.Opener,
                Card = cards[i],
                Offered = offered,
                Auto = auto,
                Spent = i == 0 ? string.Empty : spent[i - 1],
                At = now,
            });
        }

        foreach (int place in places.Order().Reverse())
        {
            pack.Cards.RemoveAt(place);
        }

        pack.Cards.AddRange(spent);
        pool.AddRange(cards);
        Picks[seat - 1]++;
        queue.RemoveAt(0);
        Deadlines[seat - 1] = null;

        if (pack.Cards.Count > 0)
        {
            Pass(pack, PassesTo(seat), now);
        }

        // The next pack this seat has waiting, on its own clock from now.
        Arrived(seat, now);

        if (Queues.All(waiting => waiting.Count == 0))
        {
            Open(now);
        }

        return cards;
    }

    /// <summary>A pack reaches a seat, at the back of its queue - the last card of it too, on its own short clock.</summary>
    private void Pass(BoosterPack pack, int to, DateTimeOffset now)
    {
        Queues[to - 1].Add(pack);
        if (Queues[to - 1].Count == 1)
        {
            Arrived(to, now);
        }
    }

    /// <summary>Starts a seat's clock on the pack now in front of it, if there is one.</summary>
    private void Arrived(int seat, DateTimeOffset now)
    {
        Deadlines[seat - 1] = Current(seat) is { } pack ? now + SecondsFor(pack.Cards.Count) : null;
    }

    /// <summary>Opens the next pack for every seat, or ends the draft when every pack has been drafted.</summary>
    private void Open(DateTimeOffset now)
    {
        if (Pack >= Packs)
        {
            Pack = 0;
            return;
        }

        Pack++;
        List<List<string>> round = Unopened[0];
        Unopened.RemoveAt(0);
        for (int seat = 1; seat <= Seats; seat++)
        {
            Queues[seat - 1].Add(new BoosterPack { Opener = seat, Cards = round[seat - 1] });
            Arrived(seat, now);
        }
    }
}

/// <summary>A pack going round the table: who opened it, and the cards still in it.</summary>
public sealed class BoosterPack
{
    /// <summary>The seat that opened it, which with <see cref="BoosterDraft.Pack"/> names it.</summary>
    public int Opener { get; set; }

    public List<string> Cards { get; set; } = [];
}

/// <summary>One pick of a booster draft, as the draft log keeps it.</summary>
public sealed class BoosterPick
{
    public int Seat { get; set; }

    /// <summary>Which pack, from 1.</summary>
    public int Pack { get; set; }

    /// <summary>Which pick from that pack, from 1: the first card taken from a fresh pack is pick 1.</summary>
    public int Pick { get; set; }

    /// <summary>The seat that opened the pack it came from.</summary>
    public int Opener { get; set; }

    public string Card { get; set; } = string.Empty;

    /// <summary>Everything in the pack when it was taken, the card taken among them.</summary>
    public List<string> Offered { get; set; } = [];

    /// <summary>Whether nobody chose it: the clock ran out, or it was the last card of the pack.</summary>
    public bool Auto { get; set; }

    /// <summary>
    /// For an additional card from the pack, the card of the drafter's pool put into the pack for it (Cogwork
    /// Librarian); empty for a pick.
    /// </summary>
    public string Spent { get; set; } = string.Empty;

    public DateTimeOffset At { get; set; }
}
