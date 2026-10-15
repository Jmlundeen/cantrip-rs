using System.Text.RegularExpressions;
using Cantrip.Protocol;

namespace Cantrip.Client.Drafting;

/// <summary>
/// A card as a drafter reads it: what it costs in each colour, what mana it makes, what it is for, and how good it is.
/// <para>
/// Read from what a player is shown - the cost, the type line, the numbers and the rules text - and from nothing else,
/// so a card this build has never met is read as well as one it has: nothing here knows a card by its name. The
/// reading is a skim, as <c>CardReading</c> is for the seat that plays: the common shapes, and a card whose text says
/// none of them is a body of its size at its cost.
/// </para>
/// </summary>
public sealed partial class DraftCard
{
    /// <summary>Every colour, as a mask: bit 0 white, 1 blue, 2 black, 3 red, 4 green - <see cref="ColorView"/>'s order.</summary>
    public const int AllColors = 0b11111;

    private DraftCard(ObjectView view)
    {
        View = view;
    }

    public ObjectView View { get; }

    public string Name => View.Name;

    public int ManaValue => View.ManaValue;

    public bool IsLand { get; private init; }

    public bool IsCreature { get; private init; }

    /// <summary>The card's colours, as a mask. A land has none, whatever it taps for.</summary>
    public int Colors { get; private init; }

    /// <summary>
    /// The coloured symbols of each way of casting it - one, but for a card with two halves or two faces - each a mask
    /// of the colours that pay for it: one bit for {R}, two for {R/W}. Phyrexian symbols and {2/W} are left out, as
    /// life or generic mana pays for them. A coloured card with no cost to read - one cast only some other way - asks
    /// for a symbol of each of its colours.
    /// </summary>
    public IReadOnlyList<IReadOnlyList<int>> Costs { get; private init; } = [];

    /// <summary>Whether it taps for mana, turn after turn: a land that does, a Mox, a Signet, an Elf. Not a Lotus, spent once.</summary>
    public bool MakesMana { get; private init; }

    /// <summary>The colours of mana it makes or finds, as a mask: a dual land's two, a fetch land's two, a Mox's one.</summary>
    public int Produces { get; private init; }

    /// <summary>How much mana it makes at a tap, where that is more than one: Sol Ring's two.</summary>
    public int ManaMade { get; private init; }

    public bool Removal { get; private init; }

    public bool Sweeper { get; private init; }

    public bool Counterspell { get; private init; }

    /// <summary>Cards beyond itself: draws two or more, or draws again and again.</summary>
    public bool CardAdvantage { get; private init; }

    public bool Evasive { get; private init; }

    public bool Planeswalker { get; private init; }

    /// <summary>
    /// What it puts onto the battlefield for less than it costs, as a mask of <see cref="CheatsCreature"/> and
    /// <see cref="CheatsArtifact"/>: a creature card from a hand or a graveyard, or one searched for or revealed, or an
    /// artifact searched for. Such a card is only as good as what the deck has for it to put there
    /// (<see cref="Payload"/>). The wording that caps what it finds ("creature card with mana value 3 or less") is
    /// not read as one: it does not say "creature card from" or "creature card, put".
    /// </summary>
    public int Cheats { get; private init; }

    /// <summary>What it is to a card that <see cref="Cheats"/>: a mask of the same bits, for a creature or an artifact costing six or more.</summary>
    public int Payload { get; private init; }

    /// <summary>Whether what it <see cref="Cheats"/> into play comes from a graveyard: what an <see cref="Outlet"/> puts there.</summary>
    public bool FromGraveyard { get; private init; }

    /// <summary>
    /// Whether it puts its controller's own cards into the graveyard by choice: draws and then discards, discards as a
    /// cost, mills itself, or searches for a card to put there. What a <see cref="FromGraveyard"/> cheat is fed by.
    /// </summary>
    public bool Outlet { get; private init; }

    public const int CheatsCreature = 1;

    public const int CheatsArtifact = 2;

    /// <summary>
    /// Which kind of deck it is for, from -1 (a deck that means to win late: sweepers, counterspells, card advantage)
    /// to +1 (a deck that means to win early: cheap creatures that hit hard, haste, burn).
    /// </summary>
    public double Speed { get; private init; }

    /// <summary>
    /// How good the card is taken by itself, on the scale a pick order's ranks are read onto (<see cref="CardWorth"/>):
    /// about 3 for a cube's middling card, 5 for one of its best twenty, 7 for the Power Nine.
    /// </summary>
    public double Worth { get; private set; }

    /// <summary>Whether <see cref="Worth"/> came from a pick order rather than from reading the card.</summary>
    public bool Ranked { get; private set; }

    /// <summary>
    /// How good the card is in a deck that can cast it, which is not quite what it is worth as a pick
    /// (<see cref="CardWorth.InDeck"/>): a pick order already marks a gold card down for how few decks can play it, and
    /// a colourless one up for how many can.
    /// </summary>
    public double Power => Worth + CardWorth.InDeck(this);

    /// <summary>The card with a worth read from where it stands in a pick order.</summary>
    public DraftCard WithWorth(double worth)
    {
        var copy = (DraftCard)MemberwiseClone();
        copy.Worth = worth;
        copy.Ranked = true;
        return copy;
    }

    /// <summary>For each of the thirty-two sets of colours a deck could be: the symbols it cannot pay for, and those only each colour pays for.</summary>
    private readonly byte[] _off = new byte[32];
    private readonly byte[] _pips = new byte[32 * 5];

    /// <summary>How many coloured symbols of its cheapest casting a deck of these colours cannot pay for.</summary>
    public int OffColor(int deck) => _off[deck & AllColors];

    /// <summary>
    /// How many symbols of its cost only this colour pays for among a deck's colours - the two of {1}{W}{W}, and the one
    /// of {R/W} in a white-blue deck - taking the casting that asks least of colours the deck does not have.
    /// </summary>
    public int Pips(int color, int deck) => _pips[((deck & AllColors) * 5) + color];

    /// <summary>Works out, once, what every deck would have to pay: these are asked thousands of times a pick.</summary>
    private void Count()
    {
        for (int deck = 0; deck <= AllColors; deck++)
        {
            IReadOnlyList<int>? best = null;
            int bestOff = int.MaxValue;
            foreach (IReadOnlyList<int> cost in Costs)
            {
                int off = 0;
                foreach (int symbol in cost)
                {
                    if ((symbol & deck) == 0)
                    {
                        off++;
                    }
                }

                if (off < bestOff)
                {
                    best = cost;
                    bestOff = off;
                }
            }

            _off[deck] = (byte)(best is null ? 0 : bestOff);
            for (int color = 0; color < 5 && best is not null; color++)
            {
                int bit = 1 << color;
                int pips = 0;
                foreach (int symbol in best)
                {
                    if ((symbol & deck) == bit)
                    {
                        pips++;
                    }
                }

                _pips[(deck * 5) + color] = (byte)pips;
            }
        }
    }

    /// <summary>The colours of a deck's that it has any symbol of, as a mask.</summary>
    public int Needs(int deck)
    {
        int needs = 0;
        for (int color = 0; color < 5; color++)
        {
            if (((deck >> color) & 1) != 0 && Pips(color, deck) > 0)
            {
                needs |= 1 << color;
            }
        }

        return needs;
    }

    private static readonly Dictionary<string, int> LandTypes = new(StringComparer.OrdinalIgnoreCase)
    {
        ["Plains"] = 1, ["Island"] = 2, ["Swamp"] = 4, ["Mountain"] = 8, ["Forest"] = 16,
    };

    public static int Mask(IEnumerable<ColorView> colors) => colors.Aggregate(0, (mask, color) => mask | (1 << (int)color));

    /// <summary>Reads a card as it is shown.</summary>
    public static DraftCard Read(ObjectView view)
    {
        ArgumentNullException.ThrowIfNull(view);
        string text = view.OracleText ?? string.Empty;
        string lower = text.ToLowerInvariant();
        bool land = view.CardTypes.Contains(CardTypeView.Land);
        bool creature = view.CardTypes.Contains(CardTypeView.Creature);

        // A split card is shown with its halves' costs added together (CR 709.4), and each half's own cost at the head
        // of its text: "Life {G} - Sorcery". Either half is a way of casting it.
        List<IReadOnlyList<int>> costs = [];
        if (view.Name.Contains(" // ", StringComparison.Ordinal))
        {
            foreach (string half in view.Name.Split(" // "))
            {
                if (Regex.Match(text, $@"^{Regex.Escape(half)} (?<cost>(\{{[^}}]+\}})+) ", RegexOptions.Multiline | RegexOptions.CultureInvariant) is { Success: true } found)
                {
                    costs.Add(Symbols(found.Groups["cost"].Value));
                }
            }
        }

        if (costs.Count == 0)
        {
            costs.Add(Symbols(view.ManaCost));
        }

        if (view.OtherFace is { ManaCost.Length: > 0 } other && !other.CardTypes.Contains(CardTypeView.Land))
        {
            costs.Add(Symbols(other.ManaCost));
        }

        if (!land && view.Colors.Count > 0 && view.ManaCost.Length == 0 && costs.TrueForAll(cost => cost.Count == 0))
        {
            costs = [[.. view.Colors.Select(color => 1 << (int)color)]];
        }

        // What it taps for: the "Add" of a mana ability it keeps, a land's basic types, what a fetch land finds.
        int produces = 0;
        int made = 0;
        bool makes = false;
        foreach (Match ability in ManaAbility().Matches(text))
        {
            string cost = ability.Groups["cost"].Value;
            string adds = ability.Groups["adds"].Value;
            if (!cost.Contains("{T}", StringComparison.Ordinal) || cost.Contains("Sacrifice", StringComparison.OrdinalIgnoreCase))
            {
                continue;
            }

            makes = true;
            int colors = Added(adds);
            produces |= colors;
            // "{C}{C}" is two mana; "{W} or {U}" and "{G}, {W}, or {U}" are one, of a choice.
            bool choice = adds.Contains("any", StringComparison.OrdinalIgnoreCase) || adds.Contains(" or ", StringComparison.Ordinal);
            made = Math.Max(made, choice ? 1 : ManaSymbol().Matches(adds).Count);
        }

        if (land)
        {
            foreach (string subtype in view.Subtypes)
            {
                if (LandTypes.TryGetValue(subtype, out int bit))
                {
                    produces |= bit;
                    makes = true;
                }
            }

            if (Fetch().Match(text) is { Success: true } fetch)
            {
                string found = fetch.Groups["what"].Value;
                int types = LandTypes.Where(type => found.Contains(type.Key, StringComparison.OrdinalIgnoreCase)).Aggregate(0, (mask, type) => mask | type.Value);
                produces |= types != 0 ? types : found.Contains("basic", StringComparison.OrdinalIgnoreCase) ? AllColors : 0;
                makes |= produces != 0;
            }
        }

        makes |= view.ProducesMana && !lower.Contains("sacrifice", StringComparison.Ordinal);

        bool removal = RemovalText().IsMatch(lower);
        bool sweeper = SweeperText().IsMatch(lower);
        bool counter = lower.Contains("counter target", StringComparison.Ordinal);
        bool advantage = DrawsCards().IsMatch(lower) || (lower.Contains("draw a card", StringComparison.Ordinal) && Repeats().IsMatch(lower));
        bool evasive = creature && EvasionText().IsMatch(lower);
        bool haste = HasKeyword(lower, "haste");
        bool burn = BurnText().IsMatch(lower);
        int power = view.Power ?? 0;

        // Which deck wants it. Cheap creatures that hit, haste and burn for the deck that wins early; sweepers,
        // counterspells and card advantage for the one that wins late. What a card costs is not part of it: a deck
        // full of five-drops is not a late deck, it is a deck short of two-drops, and that is counted elsewhere.
        double speed = 0;
        if (!land)
        {
            if (creature && view.ManaValue <= 2 && power >= 2) speed += 0.6;
            if (creature && view.ManaValue <= 1 && power >= 1) speed += 0.3;
            if (creature && view.ManaValue == 3 && power >= 3) speed += 0.3;
            if (haste) speed += 0.4;
            if (burn) speed += 0.4;
            if (sweeper) speed -= 1.0;
            if (counter) speed -= 0.5;
            if (advantage && !creature) speed -= 0.3;
            if (creature && power == 0 && !view.ProducesMana) speed -= 0.3;
        }

        // A card that cheats something big into play, and what is big enough to be worth cheating. Not a creature or
        // anything big itself, which is worth its own body whatever it finds, and not one capped at a small mana value.
        int cheats = 0;
        if (!land && !creature && view.ManaValue < Big)
        {
            cheats |= CheatCreatureText().IsMatch(lower) ? CheatsCreature : 0;
            cheats |= CheatArtifactText().IsMatch(lower) ? CheatsArtifact : 0;
        }

        bool fromGraveyard = cheats != 0 && GraveyardCheatText().IsMatch(lower);
        bool outlet = !land && OutletText().IsMatch(lower);

        int payload = land || view.ManaValue < Big ? 0
            : (creature ? CheatsCreature : 0) | (view.CardTypes.Contains(CardTypeView.Artifact) ? CheatsArtifact : 0);

        var card = new DraftCard(view)
        {
            IsLand = land,
            IsCreature = creature,
            Colors = Mask(view.Colors),
            Costs = costs,
            MakesMana = makes,
            Produces = produces,
            ManaMade = made,
            Removal = removal,
            Sweeper = sweeper,
            Counterspell = counter,
            CardAdvantage = advantage,
            Evasive = evasive,
            Planeswalker = view.CardTypes.Contains(CardTypeView.Planeswalker),
            Cheats = cheats,
            Payload = payload,
            FromGraveyard = fromGraveyard,
            Outlet = outlet,
            Speed = Math.Clamp(speed, -1, 1),
        };

        card.Count();
        card.Worth = CardWorth.Estimate(card, lower);
        return card;
    }

    /// <summary>The coloured symbols of a cost, each a mask of the colours that pay for it.</summary>
    private static List<int> Symbols(string cost)
    {
        List<int> symbols = [];
        foreach (Match symbol in ManaSymbol().Matches(cost))
        {
            string inside = symbol.Groups[1].Value;
            if (inside.Contains('P', StringComparison.Ordinal) || inside.Any(char.IsDigit))
            {
                continue;
            }

            int mask = ColorsOf(inside);
            if (mask != 0)
            {
                symbols.Add(mask);
            }
        }

        return symbols;
    }

    private static int ColorsOf(string letters)
    {
        int mask = 0;
        foreach (char letter in letters)
        {
            mask |= letter switch { 'W' => 1, 'U' => 2, 'B' => 4, 'R' => 8, 'G' => 16, _ => 0 };
        }

        return mask;
    }

    /// <summary>The colours an "Add ..." makes: its symbols', or every colour for "any color".</summary>
    private static int Added(string adds)
    {
        if (adds.Contains("any", StringComparison.OrdinalIgnoreCase))
        {
            return AllColors;
        }

        int mask = 0;
        foreach (Match symbol in ManaSymbol().Matches(adds))
        {
            mask |= ColorsOf(symbol.Groups[1].Value);
        }

        return mask;
    }

    /// <summary>The mana value from which a creature or an artifact is worth cheating into play.</summary>
    private const int Big = 6;

    internal static bool HasKeyword(string lower, string keyword) =>
        Regex.IsMatch(lower, $@"(^|\n|, ){keyword}(\b|$)", RegexOptions.CultureInvariant);

    [GeneratedRegex(@"creature cards? (from|of) (your|a|their) (hand|graveyard)[^.]*(onto|to) the battlefield|put target creature card from|enchant creature card in a graveyard|top creature card of your graveyard to the battlefield|search your library for an? (\w+ )?creature card,? (and )?put (it|that card) onto the battlefield|until they reveal a creature card[^.]*\.[^.]*puts? that card onto the battlefield", RegexOptions.CultureInvariant)]
    private static partial Regex CheatCreatureText();

    [GeneratedRegex(@"graveyard[^.]*(onto|to) the battlefield|enchant creature card in a graveyard", RegexOptions.CultureInvariant)]
    private static partial Regex GraveyardCheatText();

    [GeneratedRegex(@"draws? (a|two|three|x) cards?, then discards?|discards? (a|two|three|x) cards?, then draws?|draw a card\. if you do, discard|discard a card:|additional cost to cast this spell, (sacrifice a creature or )?discard|you may discard (a|up to \w+) cards?|discard up to \w+ cards|search your library for [^.]*put (it|that card|them) into your graveyard|put the rest into your graveyard|(^|\n|\. |, )mill (a|two|three|four|five) cards?", RegexOptions.CultureInvariant)]
    private static partial Regex OutletText();

    [GeneratedRegex(@"search your library for an artifact card(,? (and )?| *\. *if [^.]*, )put (it|that card) onto the battlefield", RegexOptions.CultureInvariant)]
    private static partial Regex CheatArtifactText();

    [GeneratedRegex(@"\{([^}]+)\}", RegexOptions.CultureInvariant)]
    private static partial Regex ManaSymbol();

    [GeneratedRegex(@"(?<cost>[^\n.:]*):\s*Add (?<adds>[^.\n]*)", RegexOptions.CultureInvariant)]
    private static partial Regex ManaAbility();

    [GeneratedRegex(@"[Ss]earch your library for an? (?<what>[^.]*?) card", RegexOptions.CultureInvariant)]
    private static partial Regex Fetch();

    [GeneratedRegex(@"(destroy|exile) (up to \w+ )?(another )?target (attacking |tapped |nonland |nontoken )?(creature|permanent|artifact, creature|creature or|planeswalker)|damage to (any target|target creature|up to \w+ target|each of up to)|damage divided as you choose|target creature gets -\d+/-\d+|target creature gets -x/-x|fights? (up to one )?(another )?target|target (player|opponent) sacrifices a creature|each opponent sacrifices a creature", RegexOptions.CultureInvariant)]
    private static partial Regex RemovalText();

    [GeneratedRegex(@"destroy all|exile all|all creatures get -|each creature gets -|damage to each creature|each player sacrifices", RegexOptions.CultureInvariant)]
    private static partial Regex SweeperText();

    [GeneratedRegex(@"draws? (two|three|four|seven|x|that many) cards", RegexOptions.CultureInvariant)]
    private static partial Regex DrawsCards();

    [GeneratedRegex(@"whenever|at the beginning of|\{t\}[^\n]*: draw|: draw a card", RegexOptions.CultureInvariant)]
    private static partial Regex Repeats();

    [GeneratedRegex(@"(^|\n|, )(flying|menace|shadow|fear|intimidate|trample|horsemanship|skulk)\b|can't be blocked|protection from", RegexOptions.CultureInvariant)]
    private static partial Regex EvasionText();

    [GeneratedRegex(@"damage to (any target|target player|each opponent|target opponent|target player or planeswalker)", RegexOptions.CultureInvariant)]
    private static partial Regex BurnText();
}
