using Cantrip.Client.Drafting;
using Cantrip.Protocol;

namespace Cantrip.Client;

/// <summary>
/// Drafts and builds a deck for a seat nobody is sitting in, so a bot can be put in an event's empty seat and an event
/// can be played through end to end without a person.
/// <para>
/// A booster draft it drafts in earnest (<see cref="CubeDrafter"/>): the card that most improves the deck its pool is
/// becoming, going by the cube's pick order, what its colours can cast and what the packs passed to it say is open -
/// and a second card with a Cogwork Librarian from any pack still half full. The deck is built in earnest too
/// (<see cref="DeckBuilder"/>), whatever the draft was: the colours, splash and lands that cast the most of the pool's
/// best cards, and never a card more than the minimum.
/// </para>
/// <para>
/// A grid draft's row or column, and whether to take a Winston pile, are weighed the same way
/// (<see cref="CubeDrafter.ChooseLine"/>, <see cref="CubeDrafter.TakePile"/>): the line the pool is worth most with,
/// and a pile taken when the pool is worth more with it than with a card from the stack. The plain ways they replaced -
/// the line worth most by the pick order (<see cref="ChooseLine"/>), any pile of two or more - are kept to be measured
/// against.
/// </para>
/// <para>
/// With a <see cref="Narrator"/> it also says what it was thinking, after each choice is made: a booster pick's closest
/// cards and what each would have added, a grid's lines, a Winston pile against passing it, and the deck and why.
/// </para>
/// </summary>
public sealed class DraftPlay
{
    private int _decided = -1;

    /// <summary>What this seat has learnt of the draft: where its cards stand in the pick order, and what it was passed.</summary>
    private readonly CubeDrafter _drafter = new();

    /// <summary>
    /// How long to look at a Winston pile or a grid before picking from it, so a person across the table sees the pick
    /// being made rather than only its result. Zero picks at once; a booster draft, on its clock, always does.
    /// </summary>
    public TimeSpan Think { get; init; }

    public TimeProvider Time { get; init; } = TimeProvider.System;

    /// <summary>
    /// Where this seat tells each pick and deck it makes and why, or null (the default) for silence. Written after the
    /// choice is sent, from the numbers it was made by: a narrated seat drafts exactly as a silent one. A writer that
    /// fails is let go, and the seat drafts on without it.
    /// </summary>
    public TextWriter? Narrator { get; set; }

    /// <summary>Whether this seat has said what draft it is in, which it does once, before its first pick.</summary>
    private bool _toldDraft;

    /// <summary>The pick this seat was first shown, and when: what <see cref="Think"/> is counted from.</summary>
    private (int Decision, long At) _seen = (-1, 0);

    /// <summary>Whether this seat has looked at the pick long enough to make it.</summary>
    private bool Thought(int decision)
    {
        if (Think <= TimeSpan.Zero)
        {
            return true;
        }

        if (_seen.Decision != decision)
        {
            _seen = (decision, Time.GetTimestamp());
            return false;
        }

        return Time.GetElapsedTime(_seen.At) >= Think;
    }

    /// <summary>Whether this seat has said ready in the ready check going on now.</summary>
    private bool _readied;

    /// <summary>Picks or builds if the seat has something to do. Returns true if it did.</summary>
    public bool Act(GameClient client)
    {
        ArgumentNullException.ThrowIfNull(client);

        // A forming table that has filled: ready at once, once a check.
        if (client.Table is { Forming: true } table)
        {
            bool checking = table.ReadyLeftMs > 0;
            if (!checking)
            {
                _readied = false;
            }
            else if (!_readied && table.Players.FirstOrDefault(player => player.Seat == client.Seat) is { Ready: false })
            {
                _readied = true;
                client.SetReady(true);
                return true;
            }
        }

        // Whatever the format, where the cards shown stand in the pick order is what the deck will be built knowing.
        if (client.Draft is { PileRanks.Count: > 0 } ranked)
        {
            _drafter.Learn(ranked.Pile, ranked.PileRanks, ranked.Total);
        }

        if (client.Draft is { Format: "booster" } booster)
        {
            if (booster.Pile.Count == 0 || booster.Decision == _decided)
            {
                return false;
            }

            _decided = booster.Decision;

            // A Cogwork Librarian goes into the pack while the pack is still deep enough that its second card beats it.
            bool second = booster.AdditionalPicks.Count > 0 && booster.Pile.Count * 2 >= booster.PackSize;
            DraftChoice choice = _drafter.Choose(booster, second ? 2 : 1, explain: Narrator is not null);
            client.PickCard(choice.Places[0], [.. choice.Places.Skip(1)]);
            Tell(booster, choice.Why);
            return true;
        }

        if (client.Draft is { Format: "grid" } grid)
        {
            if (grid.Active != client.Seat || grid.Decision == _decided || !Thought(grid.Decision))
            {
                return false;
            }

            _decided = grid.Decision;
            List<string>? why = Narrator is null ? null : [];
            client.PickLine(_drafter.ChooseLine(grid, why));
            Tell(grid, why);
            return true;
        }

        if (client.Draft is { } draft && draft.Active == client.Seat && draft.Decision != _decided && Thought(draft.Decision))
        {
            _decided = draft.Decision;
            List<string>? why = Narrator is null ? null : [];
            client.Pick(take: _drafter.TakePile(draft, why));
            Tell(draft, why);
            return true;
        }

        // Once, and again only if the server came back without it submitted: refused, or taken back.
        if (client.DeckBuilding is { Submitted: false } building && client.DeckCaughtUp)
        {
            // Between the games of a match the deck is the one already built: played again as it stands. A deck that
            // is not the minimum is not one this seat built - the whole pool, submitted for a seat that was not there
            // when the time ran out - and is built now.
            if (building.Title.Length > 0 && building.Main.Count + building.Basics.Sum(basic => basic.Count) == building.Minimum)
            {
                client.BuildDeck(building.Main, building.Basics.Where(basic => basic.Count > 0).ToDictionary(basic => basic.Name, basic => basic.Count, StringComparer.Ordinal), submit: true);
                Tell(null, [$"Deck: the one already built, played again ({building.Title})."]);
                return true;
            }

            List<string>? why = Narrator is null ? null : [];
            BuiltDeck deck = _drafter.Build(building, why);
            client.BuildDeck(deck.Main, deck.Basics, submit: true);
            Tell(null, why);
            return true;
        }

        return false;
    }

    /// <summary>
    /// Writes what a choice weighed to <see cref="Narrator"/>, once it is made - and before a draft's first pick, what
    /// draft it is and how to read what follows.
    /// </summary>
    private void Tell(DraftMessage? draft, IReadOnlyList<string>? lines)
    {
        if (Narrator is not { } narrator || lines is null)
        {
            return;
        }

        try
        {
            if (draft is not null && !_toldDraft)
            {
                _toldDraft = true;
                foreach (string line in CubeDrafter.Opening(draft))
                {
                    narrator.WriteLine(line);
                }
            }

            foreach (string line in lines)
            {
                narrator.WriteLine(line);
            }

            narrator.Flush();
        }
        catch (Exception error) when (error is IOException or ObjectDisposedException or UnauthorizedAccessException)
        {
            // A record that cannot be written is given up; the draft is not.
            Narrator = null;
        }
    }

    /// <summary>
    /// A card from a pack the plain way, which is how the grid's lines are weighed when there is no pick order
    /// (<see cref="ChooseLine"/>) and what a booster draft's cards were taken by before <see cref="CubeDrafter"/>: the best by the cube's pick order (<paramref name="ranks"/>), and among cards it ranks
    /// equal, or with none - one of the pool's two main colours, or colourless, if there is one - once the pool has a
    /// few cards to say what they are - then a creature, then the cheapest; the first in the pack among equals.
    /// </summary>
    /// <param name="ranks">Each card's place in the pick order, best from 0 (<see cref="DraftMessage.PileRanks"/>), or none.</param>
    public static int Choose(IReadOnlyList<ObjectView> pack, IReadOnlyList<ObjectView> pool, IReadOnlyList<int>? ranks = null)
    {
        ArgumentNullException.ThrowIfNull(pack);
        ArgumentNullException.ThrowIfNull(pool);
        int Rank(int place) => ranks is not null && ranks.Count == pack.Count ? ranks[place] : 0;
        List<ColorView> colors = [.. pool
            .SelectMany(card => card.Colors)
            .GroupBy(color => color)
            .OrderByDescending(group => group.Count())
            .ThenBy(group => group.Key)
            .Take(2)
            .Select(group => group.Key)];

        int Score(ObjectView card) =>
            (pool.Count < 4 || card.Colors.All(colors.Contains) ? 100 : 0)
            + (card.CardTypes.Contains(CardTypeView.Creature) ? 20 : 0)
            - card.ManaValue;

        return pack
            .Select((card, place) => (card, place))
            .OrderBy(pair => Rank(pair.place))
            .ThenByDescending(pair => Score(pair.card))
            .ThenBy(pair => pair.place)
            .First().place;
    }

    /// <summary>
    /// A row (0 to 2) or column (3 to 5) of a grid draft's grid (<paramref name="grid"/>, nine places row by row, a taken
    /// place an unknown card): the one whose cards are worth most - by the cube's pick order when there is one, each card
    /// worth more the better it ranks, and otherwise by what <see cref="Choose"/> looks for - the first among equals.
    /// </summary>
    public static int ChooseLine(IReadOnlyList<ObjectView> grid, IReadOnlyList<ObjectView> pool, IReadOnlyList<int>? ranks = null)
    {
        ArgumentNullException.ThrowIfNull(grid);
        ArgumentNullException.ThrowIfNull(pool);
        bool ranked = ranks is not null && ranks.Count == grid.Count && grid.Count > 0;
        int worst = ranked ? ranks!.Max() + 1 : 0;

        // Without an order, each card is worth where Choose would put it among the grid's: the best the most.
        List<int> known = [.. Enumerable.Range(0, grid.Count).Where(place => grid[place].Known)];
        Dictionary<int, int> worth = [];
        List<int> left = [.. known];
        while (left.Count > 0)
        {
            int best = left[Choose([.. left.Select(place => grid[place])], pool, ranked ? [.. left.Select(place => ranks![place])] : null)];
            worth[best] = ranked ? worst - ranks![best] : left.Count;
            left.Remove(best);
        }

        int Value(int line) => Places(line).Sum(place => worth.GetValueOrDefault(place));
        return Enumerable.Range(0, 6)
            .Where(line => Places(line).Any(place => place < grid.Count && grid[place].Known))
            .OrderByDescending(Value)
            .ThenBy(line => line)
            .DefaultIfEmpty(0)
            .First();
    }

    /// <summary>The grid's places in a line: a row, 0 to 2, or a column, 3 to 5.</summary>
    private static IEnumerable<int> Places(int line) => line < 3
        ? Enumerable.Range(0, 3).Select(column => (line * 3) + column)
        : Enumerable.Range(0, 3).Select(row => (row * 3) + (line - 3));
}
