using System.Text;
using Cantrip.Protocol;

namespace Cantrip.Client.Web;

/// <summary>
/// Where each permanent in one player's half goes: two rows, the creatures against the line the halves meet at and every other permanent against the half's outer edge.
/// The outer row is shelves in the order a player finds things by - enchantments, lands, mana-making artifacts, other
/// artifacts, planeswalkers, anything else - and on every row the permanents a player could not tell apart share a pile.
/// </summary>
public static class BoardRows
{
    public const float MarginX = 18f;
    public const float MarginY = 14f;

    /// <summary>Between the two rows, and the only space the rows share out.</summary>
    public const float RowGap = 10f;
    public const float CardGap = 8f;
    public const float ShelfGap = 28f;

    /// <summary>How far each card in a pile stands from the one under it.</summary>
    public const float PileStep = 18f;

    /// <summary>A battlefield card's width at the standard scale.</summary>
    public const float StandardCardWidth = 82f;

    public const float AspectRatio = 88f / 63f;

    private const int LandShelf = 1;

    public readonly record struct Placed(ObjectView Card, float Left, float Top);

    /// <summary>A pile's count, over the corner of its top card.</summary>
    public readonly record struct Badge(float X, float Y, bool GrowsLeft, int Count);

    /// <summary>
    /// The red ground under one combat group: where it lies, where the fighters stand inside it, what is written on it,
    /// and whether it opens downward to the line, as it does in the top half.
    /// </summary>
    public readonly record struct Band(float Left, float Top, float Right, float Bottom, float FightersLeft, float FightersTop, float FightersRight, float FightersBottom, string? Caption, bool OpensDownward);

    /// <param name="Height">How tall the half would have to be to show both rows whole, margins and the gap included.</param>
    public sealed record Laid(List<Placed> Cards, List<Badge> Badges, List<Band> Bands, float Width, float Height);

    /// <summary>
    /// One column of a combat group as this half shows it: an attacker (or a pile of unblocked twins), or the line of
    /// creatures blocking it, and the width both halves give the column so attacker and blockers share a centre line.
    /// </summary>
    public sealed record CombatColumn(float Width, List<ObjectView> Cards, bool Pile);

    /// <summary>A combat group in this half: its columns, and the caption its ground carries on the attacking side.</summary>
    public sealed record CombatGroup(string? Caption, List<CombatColumn> Columns);

    /// <summary>Between combat groups (the combat row's separation), and between the columns of one group.</summary>
    public const float GroupGap = 28f;
    public const float ColumnGap = 10f;

    /// <summary>Between blockers across from one attacker.</summary>
    public const float BlockerGap = 4f;

    /// <summary>Between the creatures out of combat, the combat groups and the balancing space (the creature row's separation).</summary>
    private const float CreatureRowGap = 14f;

    /// <summary>How far the ground reaches past the fighters, and the room its caption takes.</summary>
    private const float BandPad = 8f;
    private const float CaptionRoom = 20f;

    /// <summary>
    /// What is tucked under a permanent: the cards it has exiled until it leaves, then the Auras and Equipment attached
    /// to it, each after whatever is tucked under it in turn, furthest back first. Set by the table before it lays
    /// anything out; nothing is tucked under anything until it is.
    /// </summary>
    public static Func<ObjectView, IReadOnlyList<ObjectView>> Tucked { get; set; } = _ => [];

    /// <summary>
    /// How far each tucked card stands up and to the left of the next, as a fraction of a card's width: enough to show
    /// its name and the edge of its frame.
    /// </summary>
    private const float TuckAcross = 0.1f;
    private const float TuckDown = 0.18f;

    /// <summary>How wide a pile stands, as a combat column measures its attacker.</summary>
    public static float PileWidthOf(IReadOnlyList<ObjectView> pile, float cardWidth) =>
        pile.Count == 0 ? 0f : PileWidth([.. pile], cardWidth, CardHeight(cardWidth));

    /// <summary>
    /// How wide a half's creatures out of combat stand, for giving both halves the wider of the two while there is a
    /// fight, so the groups between them stand at the same place in each.
    /// </summary>
    public static float CreaturesWidth(IEnumerable<ObjectView> permanents, float cardWidth, Func<ObjectView, string> look) =>
        RowWidth([Alike(permanents.OrderBy(card => card.Handle.Value).Where(IsCreature), look)], cardWidth, CardHeight(cardWidth));

    /// <summary>
    /// The card width at which two rows of permanents fill a half's height, so the size the board is drawn at does not
    /// depend on what is on it.
    /// </summary>
    public static float CardWidth(float halfHeight)
    {
        float rows = halfHeight - (2f * MarginY) - (2f * RowGap);
        float scale = Math.Clamp(rows / 2f / AspectRatio / StandardCardWidth, 0.5f, 4f);
        return MathF.Round(StandardCardWidth * scale);
    }

    public static float CardHeight(float width) => MathF.Round(width * AspectRatio);

    /// <param name="top">The opponent's half, whose outer edge is the top of the screen.</param>
    /// <param name="look">What else about a card shows right now - what can be done with it - so twins that differ in it stand apart.</param>
    /// <param name="spent">A tapped land nothing is asked about, which joins the one pile of spent lands.</param>
    public static Laid Lay(
        IEnumerable<ObjectView> permanents,
        bool top,
        float viewportWidth,
        float halfHeight,
        float cardWidth,
        Func<ObjectView, string> look,
        Func<ObjectView, bool> spent,
        IReadOnlyList<CombatGroup>? combat = null,
        float alignedCreatures = 0f)
    {
        combat ??= [];
        float cardHeight = CardHeight(cardWidth);
        List<ObjectView> all = [.. permanents.OrderBy(card => card.Handle.Value)];

        List<List<List<ObjectView>>> creatureRow = [Alike(all.Where(IsCreature), look)];
        List<List<List<ObjectView>>> shelfRow = [];

        foreach (IGrouping<int, ObjectView> shelf in all.Where(card => !IsCreature(card)).GroupBy(Shelf).OrderBy(group => group.Key))
        {
            if (shelf.Key == LandShelf)
            {
                // Every spent land in one pile at the end of the shelf, whatever they are named.
                List<List<ObjectView>> lands = Alike(shelf.Where(land => !spent(land)), look);
                List<ObjectView> spentLands = [.. shelf.Where(spent)];
                if (spentLands.Count > 0)
                {
                    lands.Add(spentLands);
                }

                shelfRow.Add(lands);
            }
            else
            {
                shelfRow.Add(Alike(shelf, look));
            }
        }

        float creatureWidth = RowWidth(creatureRow, cardWidth, cardHeight);
        float shelfWidth = RowWidth(shelfRow, cardWidth, cardHeight);

        // While there is a fight the creature row is three parts: the creatures out of
        // combat, right-aligned in a box; the combat groups; and a balancing space as wide as that box, so the groups
        // stand in the middle. Both boxes are at least the wider of the two halves' creatures.
        bool fighting = combat.Count > 0;
        float combatWidth = fighting
            ? combat.Sum(group => group.Columns.Sum(column => column.Width) + (ColumnGap * (group.Columns.Count - 1))) + (GroupGap * (combat.Count - 1))
            : 0f;
        float needed = fighting
            ? (2f * MathF.Max(creatureWidth, alignedCreatures)) + (2f * CreatureRowGap) + combatWidth
            : creatureWidth;

        float width = MathF.Max(viewportWidth, MathF.Max(needed, shelfWidth) + (2f * MarginX));
        float inner = width - (2f * MarginX);
        float side = fighting ? MathF.Max(MathF.Max(creatureWidth, alignedCreatures), (inner - combatWidth - (2f * CreatureRowGap)) / 2f) : 0f;

        float creatureHeight = MathF.Max(RowHeight(creatureRow, cardWidth, cardHeight), fighting ? cardHeight : 0f);
        float shelfHeight = RowHeight(shelfRow, cardWidth, cardHeight);

        // The only space is between the rows: the row against the outer edge is always the margin from it, and the
        // creature rows stay against the line they fight across. When the rows need all the height, the space closes;
        // when they need more than there is, the outer row goes past the half's outer edge, which the half clips, and
        // the creatures still stand against the line.
        float innerTop = top ? halfHeight - MarginY - creatureHeight : MarginY;
        float outerTop = top
            ? MathF.Min(MarginY, innerTop - RowGap - shelfHeight)
            : MathF.Max(innerTop + creatureHeight + RowGap, halfHeight - MarginY - shelfHeight);

        var laid = new Laid([], [], [], width, (2f * MarginY) + creatureHeight + RowGap + shelfHeight);
        PlaceRow(laid, creatureRow, fighting ? MarginX + side - creatureWidth : (width - creatureWidth) / 2f, innerTop, creatureHeight, cardWidth, cardHeight);

        if (fighting)
        {
            PlaceCombat(laid, combat, MarginX + side + CreatureRowGap, innerTop, creatureHeight, cardWidth, cardHeight, top, halfHeight);
        }

        PlaceRow(laid, shelfRow, (width - shelfWidth) / 2f, outerTop, shelfHeight, cardWidth, cardHeight);
        return laid;
    }

    /// <summary>
    /// Everything about a card that shows on the board, as one string: two cards with the same string look the same to
    /// the player and can share a pile.
    /// </summary>
    public static string Likeness(ObjectView card, string alsoShown) => string.Concat(CardLikenesses.GetValue(card, CardLikeness), alsoShown);

    /// <summary>
    /// The part of a card's likeness the card itself says, worked out once for each object the client is sent: a view is
    /// never changed once received, and every permanent is asked for more than once a redraw.
    /// </summary>
    private static readonly System.Runtime.CompilerServices.ConditionalWeakTable<ObjectView, string> CardLikenesses = new();

    private static string CardLikeness(ObjectView card)
    {
        var look = new StringBuilder();

        look.Append(card.Name).Append('|')
            .Append(card.Controller).Append('|')
            .Append(card.Owner).Append('|')
            .Append(card.Kind).Append('|')
            .Append(card.IsCopy ? 'C' : 'c')
            // Which card it is, so a Metamorph and a Clone copying one Solitude are not piled as one.
            .Append(card.CardId).Append('|')
            .Append(card.Tapped ? 'T' : 'u')
            .Append(card.SummoningSick ? 'S' : 'r')
            .Append(card.FaceDown ? 'F' : 'f')
            .Append(card.Monstrous ? 'M' : 'm')
            .Append(card.Solved ? 'V' : 'v')
            .Append(card.Suspected ? 'Q' : 'q')
            .Append(card.PhasedOut ? 'P' : 'p')
            .Append(card.ClassLevel)
            .Append(card.Harnessed ? 'H' : 'h')
            .Append(card.Exerted ? 'E' : 'e')
            .Append(card.Saddled ? 'D' : 'd')
            .Append(card.Goaded ? 'G' : 'g')
            .Append(card.Known ? 'K' : 'k').Append('|')
            .Append(card.Power).Append('/').Append(card.Toughness).Append('|')
            .Append(card.DamageMarked).Append('|')
            .AppendJoin(',', card.Counters.OrderBy(c => c.Name, StringComparer.Ordinal).Select(c => $"{c.Name}:{c.Count}")).Append('|')
            .AppendJoin(',', card.Colors).Append('|')
            .AppendJoin(',', card.Supertypes).Append('|')
            .AppendJoin(',', card.CardTypes).Append('|')
            .AppendJoin(',', card.Subtypes).Append('|')
            .Append(card.ManaCost).Append('|')
            .Append(card.OracleText).Append('|')
            .AppendJoin(',', card.GrantedAbilities.Select(a => a.ToString())).Append('|')
            .AppendJoin(',', card.LostAbilities).Append('|')
            .Append(card.CantBeCountered ? 'U' : 'u').Append(card.EveryCreatureType ? 'Y' : 'y').Append('|')
            .AppendJoin(',', card.TypeLineSubtypes).Append('|')
            .AppendJoin(',', card.SpentLines).Append('|')
            .AppendJoin('|', card.ChosenModes).Append('|');

        return look.ToString();
    }

    /// <summary>
    /// Piles of cards that look alike, each where its first card was, and a card's untapped pile before its tapped one.
    /// </summary>
    private static List<List<ObjectView>> Alike(IEnumerable<ObjectView> cards, Func<ObjectView, string> look)
    {
        List<List<List<ObjectView>>> kins = [];
        Dictionary<string, int> byKin = [];
        Dictionary<string, List<ObjectView>> byLook = [];

        foreach (ObjectView card in cards)
        {
            string likeness = look(card);
            if (byLook.TryGetValue(likeness, out List<ObjectView>? group))
            {
                group.Add(card);
                continue;
            }

            string kin = $"{card.Name}|{card.Controller}";
            if (!byKin.TryGetValue(kin, out int index))
            {
                index = kins.Count;
                byKin[kin] = index;
                kins.Add([]);
            }

            group = [card];
            byLook[likeness] = group;
            kins[index].Add(group);
        }

        return [.. kins.SelectMany(kin => kin.OrderBy(group => group[0].Tapped))];
    }

    private static int Shelf(ObjectView card) =>
        card.CardTypes.Contains(CardTypeView.Land) ? LandShelf
        : card.CardTypes.Contains(CardTypeView.Planeswalker) ? 4
        : card.CardTypes.Contains(CardTypeView.Artifact) ? card.ProducesMana ? 2 : 3
        : card.CardTypes.Contains(CardTypeView.Enchantment) ? 0
        : 5;

    private static bool IsCreature(ObjectView card) => card.CardTypes.Contains(CardTypeView.Creature);

    /// <summary>
    /// How wide a pile stands: one card, turned if tapped, and a step for each card under it; or, for one card with
    /// things tucked under it, the whole of that stack.
    /// </summary>
    private static float PileWidth(List<ObjectView> pile, float cardWidth, float cardHeight) =>
        pile.Count == 1 && Tucked(pile[0]) is { Count: > 0 } under
            ? Stack(pile[0], under, cardWidth, cardHeight).Width
            : (pile[0].Tapped ? cardHeight : cardWidth) + ((pile.Count - 1) * PileStep);

    private static float PileHeight(List<ObjectView> pile, float cardWidth, float cardHeight) =>
        pile.Count == 1 && Tucked(pile[0]) is { Count: > 0 } under
            ? Stack(pile[0], under, cardWidth, cardHeight).Height
            : pile[0].Tapped ? cardWidth : cardHeight;

    /// <summary>
    /// A permanent with things tucked under it, as boxes from its top left: each tucked card a step further down and
    /// right than the one before, the permanent itself last, on top. Each box is its card as it lies, turned if tapped.
    /// </summary>
    private static (float Width, float Height, List<(ObjectView Card, float X, float Y, float W, float H)> Boxes) Stack(
        ObjectView holder, IReadOnlyList<ObjectView> under, float cardWidth, float cardHeight)
    {
        float across = MathF.Round(cardWidth * TuckAcross);
        float down = MathF.Round(cardWidth * TuckDown);
        List<(ObjectView, float, float, float, float)> boxes = [];
        float width = 0f;
        float height = 0f;

        for (int i = 0; i <= under.Count; i++)
        {
            ObjectView card = i < under.Count ? under[i] : holder;
            float w = card.Tapped ? cardHeight : cardWidth;
            float h = card.Tapped ? cardWidth : cardHeight;
            boxes.Add((card, across * i, down * i, w, h));
            width = MathF.Max(width, (across * i) + w);
            height = MathF.Max(height, (down * i) + h);
        }

        return (width, height, boxes);
    }

    private static float RowWidth(List<List<List<ObjectView>>> groups, float cardWidth, float cardHeight)
    {
        List<float> widths = [.. groups.Where(g => g.Count > 0)
            .Select(g => g.Sum(pile => PileWidth(pile, cardWidth, cardHeight)) + (CardGap * (g.Count - 1)))];

        return widths.Count == 0 ? 0f : widths.Sum() + (ShelfGap * (widths.Count - 1));
    }

    /// <summary>A row as tall as its tallest pile: a row of nothing but tapped cards lies lower.</summary>
    private static float RowHeight(List<List<List<ObjectView>>> groups, float cardWidth, float cardHeight)
    {
        List<float> heights = [.. groups.SelectMany(g => g).Select(pile => PileHeight(pile, cardWidth, cardHeight))];
        return heights.Count == 0 ? 0f : heights.Max();
    }

    /// <summary>
    /// The combat groups: each column centred in its width, an attacker pile or a line of blockers, and under each group
    /// the ground that runs from the fighters to the line between the halves.
    /// </summary>
    private static void PlaceCombat(Laid laid, IReadOnlyList<CombatGroup> combat, float left, float top, float rowHeight, float cardWidth, float cardHeight, bool topHalf, float halfHeight)
    {
        float x = left;
        float slotTop = top + ((rowHeight - cardHeight) / 2f);

        foreach (CombatGroup group in combat)
        {
            float groupLeft = x;

            foreach (CombatColumn column in group.Columns)
            {
                if (column.Pile && column.Cards.Count > 0)
                {
                    PlaceRow(laid, [[column.Cards]], x + ((column.Width - PileWidthOf(column.Cards, cardWidth)) / 2f), top, rowHeight, cardWidth, cardHeight);
                }
                else if (column.Cards.Count > 0)
                {
                    List<List<ObjectView>> line = [.. column.Cards.Select(card => new List<ObjectView> { card })];
                    float lineWidth = line.Sum(one => PileWidth(one, cardWidth, cardHeight)) + (BlockerGap * (line.Count - 1));
                    float at = x + ((column.Width - lineWidth) / 2f);

                    foreach (List<ObjectView> one in line)
                    {
                        PlaceRow(laid, [[one]], at, top, rowHeight, cardWidth, cardHeight);
                        at += PileWidth(one, cardWidth, cardHeight) + BlockerGap;
                    }
                }

                x += column.Width + ColumnGap;
            }

            float groupRight = x - ColumnGap;
            float room = group.Caption is null ? 0f : CaptionRoom;

            laid.Bands.Add(topHalf
                ? new Band(groupLeft - BandPad, slotTop - BandPad - room, groupRight + BandPad, halfHeight, groupLeft, slotTop, groupRight, slotTop + cardHeight, group.Caption, OpensDownward: true)
                : new Band(groupLeft - BandPad, 0f, groupRight + BandPad, slotTop + cardHeight + BandPad + room, groupLeft, slotTop, groupRight, slotTop + cardHeight, group.Caption, OpensDownward: false));

            x = groupRight + GroupGap;
        }
    }

    private static void PlaceRow(Laid laid, List<List<List<ObjectView>>> groups, float left, float top, float rowHeight, float cardWidth, float cardHeight)
    {
        float x = left;

        foreach (List<List<ObjectView>> group in groups.Where(g => g.Count > 0))
        {
            foreach (List<ObjectView> pile in group)
            {
                // One permanent with things tucked under it: the tucked cards first, so the permanent is drawn over them.
                if (pile.Count == 1 && Tucked(pile[0]) is { Count: > 0 } under)
                {
                    (float stackWidth, float stackHeight, var boxes) = Stack(pile[0], under, cardWidth, cardHeight);
                    float stackTop = top + ((rowHeight - stackHeight) / 2f);

                    foreach ((ObjectView card, float bx, float by, float bw, float bh) in boxes)
                    {
                        float centreX = x + bx + (bw / 2f);
                        float centreY = stackTop + by + (bh / 2f);
                        laid.Cards.Add(new Placed(card, centreX - (cardWidth / 2f), centreY - (cardHeight / 2f)));
                    }

                    x += stackWidth + CardGap;
                    continue;
                }

                bool turned = pile[0].Tapped;
                float slotWidth = turned ? cardHeight : cardWidth;
                float slotHeight = turned ? cardWidth : cardHeight;
                float slotTop = top + ((rowHeight - slotHeight) / 2f);

                for (int index = 0; index < pile.Count; index++)
                {
                    // Upright or turned, the pile steps right from its top card, which is at the left.
                    float step = (pile.Count - 1 - index) * PileStep;
                    float centreX = x + step + (slotWidth / 2f);
                    float centreY = slotTop + (slotHeight / 2f);
                    laid.Cards.Add(new Placed(pile[index], centreX - (cardWidth / 2f), centreY - (cardHeight / 2f)));
                }

                if (pile.Count > 1)
                {
                    // The count sits on the top card, at the left of the pile.
                    laid.Badges.Add(turned
                        ? new Badge(x + 5f, slotTop - 6f, GrowsLeft: false, pile.Count)
                        : new Badge(x + slotWidth - 5f, slotTop - 6f, GrowsLeft: true, pile.Count));
                }

                x += PileWidth(pile, cardWidth, cardHeight) + CardGap;
            }

            x += ShelfGap - CardGap;
        }
    }
}
