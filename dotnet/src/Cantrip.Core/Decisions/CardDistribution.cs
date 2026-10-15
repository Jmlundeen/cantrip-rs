namespace Cantrip.Core.Decisions;

/// <summary>
/// How many cards one destination of a distribution takes.
/// <para>
/// <see cref="Max"/> of <see cref="Any"/> means "and the rest": no upper bound beyond the cards there are.
/// </para>
/// </summary>
public readonly record struct DestinationLimit(int Min, int Max)
{
    /// <summary>No upper bound: this destination takes as many as are left.</summary>
    public const int Any = int.MaxValue;

    /// <summary>A destination with no requirement and no limit, as scrying's top and bottom both are.</summary>
    public static DestinationLimit Unlimited => new(0, Any);

    /// <summary>A destination that takes exactly <paramref name="count"/> cards, as Stock Up's hand takes two.</summary>
    public static DestinationLimit Exactly(int count) => new(count, count);
}

/// <summary>
/// The rule behind every "look at some cards and say where each one goes" question: scry (CR 701.22), surveil
/// (CR 701.25), Ponder's "put them back in any order", Stock Up's "put two of them into your hand and the rest on the
/// bottom".
/// <para>
/// It lives in <c>Cantrip.Core</c> because three places have to agree about it and disagreeing is a bug in each
/// direction. The engine's prompt refuses an answer that breaks it, so a bad answer is re-asked rather than fatal; the
/// client greys its Done button by it, so a player is never offered a placement that will be refused; and the wire
/// message answers itself by it, so a client that draws no dock at all still returns something legal. One function,
/// three callers, no drift.
/// </para>
/// <para>
/// Everything here is pure arithmetic over card identities, which is what keeps it shareable: a destination arrives
/// carrying the cards it will accept rather than a filter to evaluate, so no side needs to know what a card <em>is</em>.
/// The engine works that list out from the real <c>CardFilter</c> once, as the question is asked.
/// </para>
/// </summary>
public static class CardDistribution
{
    /// <summary>
    /// Why this placement is not a legal answer, or null when it is.
    /// <para>
    /// The three lists are index-parallel: <paramref name="limits"/>[i], <paramref name="eligible"/>[i] and
    /// <paramref name="placements"/>[i] are all about destination i. A legal placement gives every card in
    /// <paramref name="cards"/> exactly one destination that will accept it, and leaves each destination holding a
    /// number of cards within its limit. <paramref name="groups"/>, when given, is index-parallel too: a destination
    /// with groups takes at most one card of each (see <see cref="Fits"/>).
    /// </para>
    /// </summary>
    public static string? Problem<T>(
        IReadOnlyList<T> cards,
        IReadOnlyList<DestinationLimit> limits,
        IReadOnlyList<IReadOnlyList<T>> eligible,
        IReadOnlyList<IReadOnlyList<T>> placements,
        IReadOnlyList<IReadOnlyList<IReadOnlyList<T>>>? groups = null)
        where T : notnull
    {
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(limits);
        ArgumentNullException.ThrowIfNull(eligible);
        ArgumentNullException.ThrowIfNull(placements);

        if (limits.Count != eligible.Count)
        {
            throw new ArgumentException("Every destination needs both a limit and a list of what it accepts.", nameof(eligible));
        }

        if (placements.Count != limits.Count)
        {
            return $"The answer places cards into {placements.Count} destination(s), but there are {limits.Count}.";
        }

        var placed = new HashSet<T>();

        for (int i = 0; i < placements.Count; i++)
        {
            IReadOnlyList<T> here = placements[i];
            DestinationLimit limit = limits[i];

            if (here.Count < limit.Min || here.Count > limit.Max)
            {
                return $"Destination {i} takes {Describe(limit)}, not {here.Count}.";
            }

            var accepted = eligible[i] as ISet<T> ?? new HashSet<T>(eligible[i]);

            foreach (T card in here)
            {
                if (!placed.Add(card))
                {
                    return "A card was placed in more than one destination.";
                }

                if (!accepted.Contains(card))
                {
                    return $"A card was placed in destination {i}, which does not accept it.";
                }
            }

            if (GroupsOf(groups, i) is { } kinds && !Fits(here, kinds))
            {
                return $"Destination {i} takes one card of each kind, and these cannot each be a different one.";
            }
        }

        // Every card is accounted for: the count settles it, since each placed card was distinct and had to be
        // eligible somewhere, and anything not among the cards looked at is not eligible anywhere.
        return placed.Count == cards.Count
            ? null
            : $"Every one of the {cards.Count} card(s) needs a destination; {placed.Count} were placed.";
    }

    /// <summary>
    /// The limits as they can actually be met, given how many cards there are to place.
    /// <para>
    /// A requirement is only a requirement while there are cards to meet it with (CR 609.3 does as much as it can and
    /// no more). Stock Up asks for two cards in hand, but a player with one card left in their library puts that one
    /// card into their hand and nothing on the bottom. Requirements are met in the order the card lists them, so the
    /// earlier destination is the one that keeps what it can — which is why a card's destinations are written in the
    /// order its rules text names them.
    /// </para>
    /// <para>
    /// Maximums are left alone: a destination that could take more cards than exist simply does not get them.
    /// </para>
    /// </summary>
    public static DestinationLimit[] Achievable(int cards, IReadOnlyList<DestinationLimit> limits)
    {
        ArgumentNullException.ThrowIfNull(limits);

        var achievable = new DestinationLimit[limits.Count];
        int left = Math.Max(0, cards);

        for (int i = 0; i < limits.Count; i++)
        {
            int min = Math.Min(limits[i].Min, left);
            achievable[i] = new DestinationLimit(min, limits[i].Max);
            left -= min;
        }

        return achievable;
    }

    /// <summary>
    /// A legal placement that asks nothing of anybody: the answer an automatic player gives, and the one a client with
    /// no dock for this question falls back to.
    /// <para>
    /// Requirements are met first, in order, from the cards each destination will accept; whatever is left over goes
    /// to the first destination that will still take it. That is the least interesting legal answer rather than a good
    /// one — scrying keeps the library as it was, and Stock Up takes the top two cards — which is exactly what a
    /// default is for.
    /// </para>
    /// </summary>
    public static List<List<T>> Greedy<T>(
        IReadOnlyList<T> cards,
        IReadOnlyList<DestinationLimit> limits,
        IReadOnlyList<IReadOnlyList<T>> eligible,
        IReadOnlyList<IReadOnlyList<IReadOnlyList<T>>>? groups = null)
        where T : notnull
    {
        ArgumentNullException.ThrowIfNull(cards);
        ArgumentNullException.ThrowIfNull(limits);
        ArgumentNullException.ThrowIfNull(eligible);

        List<List<T>> placements = [.. limits.Select(_ => new List<T>())];
        var accepted = eligible.Select(list => list as ISet<T> ?? new HashSet<T>(list)).ToArray();
        var left = new List<T>(cards);

        // Requirements first, in order, so the destinations a card names first are the ones that fill up.
        for (int i = 0; i < limits.Count; i++)
        {
            Fill(i, limits[i].Min);
        }

        // Then everything still unplaced, wherever it will go.
        for (int i = 0; i < limits.Count && left.Count > 0; i++)
        {
            Fill(i, limits[i].Max);
        }

        return placements;

        void Fill(int destination, int upTo)
        {
            for (int c = 0; c < left.Count && placements[destination].Count < upTo;)
            {
                if (accepted[destination].Contains(left[c])
                    && (GroupsOf(groups, destination) is not { } kinds || Fits([.. placements[destination], left[c]], kinds)))
                {
                    placements[destination].Add(left[c]);
                    left.RemoveAt(c);
                }
                else
                {
                    c++;
                }
            }
        }
    }

    /// <summary>
    /// Whether the cards put in one destination can each be counted as a different one of its groups, as Atraxa, Grand
    /// Unifier's "for each card type, you may put a card of that type into your hand": an artifact creature may be the
    /// artifact or the creature, but not both, and a second creature needs a group of its own.
    /// <para>
    /// A matching question (Hall's theorem), answered by augmenting paths: each card in turn takes a group it belongs
    /// to, moving an earlier card to another of its groups where that frees one. Tiny sizes — ten cards, nine card
    /// types — so the plain algorithm is the right one.
    /// </para>
    /// </summary>
    public static bool Fits<T>(IReadOnlyList<T> placed, IReadOnlyList<IReadOnlyList<T>> groups)
        where T : notnull
    {
        ArgumentNullException.ThrowIfNull(placed);
        ArgumentNullException.ThrowIfNull(groups);

        if (placed.Count > groups.Count)
        {
            return false;
        }

        var holder = new T[groups.Count];
        var held = new bool[groups.Count];
        var taken = new bool[groups.Count];

        foreach (T card in placed)
        {
            Array.Clear(taken);
            if (!Claim(card))
            {
                return false;
            }
        }

        return true;

        bool Claim(T card)
        {
            for (int g = 0; g < groups.Count; g++)
            {
                if (taken[g] || !groups[g].Contains(card))
                {
                    continue;
                }

                taken[g] = true;
                if (!held[g] || Claim(holder[g]))
                {
                    holder[g] = card;
                    held[g] = true;
                    return true;
                }
            }

            return false;
        }
    }

    /// <summary>A destination's groups, or null where it has none and so takes cards without regard to kind.</summary>
    private static IReadOnlyList<IReadOnlyList<T>>? GroupsOf<T>(IReadOnlyList<IReadOnlyList<IReadOnlyList<T>>>? groups, int destination) =>
        groups is not null && destination < groups.Count && groups[destination].Count > 0 ? groups[destination] : null;

    private static string Describe(DestinationLimit limit) =>
        limit.Max == DestinationLimit.Any
            ? limit.Min == 0 ? "any number of cards" : $"at least {limit.Min} card(s)"
            : limit.Min == limit.Max ? $"exactly {limit.Min} card(s)"
            : $"{limit.Min} to {limit.Max} card(s)";
}
