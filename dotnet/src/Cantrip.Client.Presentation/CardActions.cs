using Cantrip.Protocol;

namespace Cantrip.Client.Presentation;

/// <summary>
/// What the server is currently offering, indexed by the card offering it.
/// <para>
/// A re-indexing of the priority prompt and nothing more. Every entry came from an
/// <see cref="ActionOptionMessage"/> the server sent, so a card with no entry has no control on it — the client
/// never works out for itself that something ought to be playable.
/// </para>
/// <para>
/// A pay-costs prompt is indexed the same way: the lands it offers to tap are clicked exactly as they are from a
/// priority prompt, which is what makes paying for a spell feel like the rest of the board rather than a dialog.
/// </para>
/// </summary>
public sealed class CardActions
{
    private static readonly IReadOnlyList<ActionOptionMessage> None = [];

    private readonly Dictionary<int, List<ActionOptionMessage>> _bySource = [];

    public CardActions(PromptPayload? prompt)
    {
        List<ActionOptionMessage>? offered = prompt switch
        {
            PriorityPromptMessage priority => priority.Options,
            PayCostsPromptMessage pay => pay.Options,
            UnlessPaysPromptMessage unless => unless.Options,
            _ => null,
        };

        if (offered is null)
        {
            return;
        }

        // Kept whole as well as indexed: choosing which of a card's mana abilities to use means counting what every
        // other card could have produced, which is a question about the prompt rather than about any one card.
        Offered = offered;

        foreach (ActionOptionMessage option in offered)
        {
            // Belongs to no card; the undo button offers it.
            if (option.Kind == ActionKindView.UndoManaAbility)
            {
                Undo = option;
                continue;
            }

            // Belongs to no card either; the dock's cancel button offers it.
            if (option.Kind == ActionKindView.CancelCasting)
            {
                Cancel = option;
                continue;
            }

            // Paying or declining an optional cost: dock buttons too.
            if (option.Kind == ActionKindView.PayCost)
            {
                Pay = option;
                continue;
            }

            if (option.Kind == ActionKindView.DeclineCost)
            {
                Decline = option;
                continue;
            }

            // Choosing X: pool mana to put towards the spell belongs to the pool chips, and the rest to the dock.
            switch (option.Kind)
            {
                case ActionKindView.AssignMana:
                    Assignable.Add(option);
                    continue;

                case ActionKindView.IncreaseX:
                    IncreaseX = option;
                    continue;

                case ActionKindView.DecreaseX:
                    DecreaseX = option;
                    continue;

                case ActionKindView.ConfirmCasting:
                    Confirm = option;
                    continue;

                case ActionKindView.PayLifeForSymbol:
                    PayLife.Add(option);
                    continue;

                default:
                    break;
            }

            if (!_bySource.TryGetValue(option.Source.Value, out List<ActionOptionMessage>? options))
            {
                options = [];
                _bySource[option.Source.Value] = options;
            }

            options.Add(option);
        }
    }

    public bool Any => _bySource.Count > 0;

    /// <summary>Everything the prompt offered, in the order it offered it. Empty when the prompt offers nothing.</summary>
    public IReadOnlyList<ActionOptionMessage> Offered { get; } = None;

    /// <summary>The offer to take back the last mana ability, when there is one.</summary>
    public ActionOptionMessage? Undo { get; }

    /// <summary>The offer to stop casting the spell being paid for, while one is.</summary>
    public ActionOptionMessage? Cancel { get; }

    /// <summary>The offer to pay an optional cost, once the pool covers it.</summary>
    public ActionOptionMessage? Pay { get; }

    /// <summary>The offer to decline an optional cost.</summary>
    public ActionOptionMessage? Decline { get; }

    /// <summary>Offers to put one mana from the pool towards the spell being paid for, one per kind of free mana.</summary>
    public List<ActionOptionMessage> Assignable { get; } = [];

    /// <summary>The offer to raise X by one, while paying for a spell with {X}.</summary>
    public ActionOptionMessage? IncreaseX { get; }

    /// <summary>The offer to lower X by one, while paying for a spell with {X}.</summary>
    public ActionOptionMessage? DecreaseX { get; }

    /// <summary>The offer to cast the spell with X as it stands, once nothing is owed.</summary>
    public ActionOptionMessage? Confirm { get; }

    /// <summary>Offers to pay a Phyrexian mana symbol still owed with life instead of mana (CR 107.4f), one per symbol.</summary>
    public List<ActionOptionMessage> PayLife { get; } = [];

    /// <summary>The offer to put one mana of this kind towards the spell, or null when none is free.</summary>
    public ActionOptionMessage? AssignFor(ManaTypeView type, ManaSpendFilterView filter, ManaSpendEffectView effect) =>
        Assignable.FirstOrDefault(option =>
            option.Produces.Count == 1 && option.Produces[0] == type
            && option.Filter == filter && option.Effect == effect);

    public IReadOnlyList<ActionOptionMessage> For(ViewHandle handle) =>
        _bySource.TryGetValue(handle.Value, out List<ActionOptionMessage>? options) ? options : None;

    /// <summary>
    /// The mana abilities the player has of their own (CR 611.2a, 605.1a), such as the one Channel gives: no card
    /// offers them, so they come with no source and the name of what gave them, and the player's own area shows them.
    /// </summary>
    public IReadOnlyList<ActionOptionMessage> Own =>
        [.. For(ViewHandle.None).Where(option => option.Kind == ActionKindView.ActivateAbility && option.GrantedBy.Length > 0)];

    /// <summary>
    /// How an action reads on a menu.
    /// <para>
    /// The trailing ellipsis is not decoration: it says the click will ask for a target rather than do the thing,
    /// which is the difference between a menu item that commits and one that opens a second decision.
    /// </para>
    /// </summary>
    /// <param name="color">
    /// For a mana ability that adds mana of any one color: the color this row adds. Each color is its own row, so
    /// Talisman of Indulgence reads as three abilities rather than two with a second menu behind one of them.
    /// </param>
    public static string Describe(
        GameClient client,
        ActionOptionMessage option,
        ColorView? color = null,
        IReadOnlyList<ColorView>? combination = null,
        string sacrificed = "",
        bool anyPermanent = false)
    {
        ArgumentNullException.ThrowIfNull(client);
        ArgumentNullException.ThrowIfNull(option);

        string name = option.GrantedBy.Length > 0 ? option.GrantedBy : client.Object(option.Source)?.Name ?? "this";

        // The ellipsis means "taking this will ask something more", so a row that has already narrowed one of the
        // ability's own choices drops that reason for it while keeping the others: this row is one combination, not
        // "and then it will want to know which". A row standing for every permanent the cost could take asks which.
        bool asks = option.HasX
            || (option.ProducesOfAnyColorEach > 0 && combination is not { Count: > 0 })
            || anyPermanent;

        string ellipsis = NeedsATarget(option) || asks ? "..." : string.Empty;

        // A mana ability reads as its cost and the mana it adds, in symbols, since those are the whole of what
        // distinguishes one from another.
        if (option.Kind == ActionKindView.ActivateAbility)
        {
            string adds = string.Concat(option.Produces.Select(type => $"{{{Symbol(type)}}}"));

            if (color is { } chosen)
            {
                adds += string.Concat(Enumerable.Repeat($"{{{Symbol(chosen)}}}", option.ProducesOfAnyOneColor));
            }
            else if (ChoosesColorless(option))
            {
                // A choice among the types a sacrificed land could produce (CR 106.7) narrowed to colorless alone.
                adds += string.Concat(Enumerable.Repeat($"{{{Symbol(ManaTypeView.Colorless)}}}", option.ProducesOfAnyOneColor));
            }
            else if (option.ProducesOfAnyOneColor > 0)
            {
                adds += (adds.Length > 0 ? " and " : string.Empty) +
                    $"{option.ProducesOfAnyOneColor} of {string.Join("/", option.ColorChoices.Select(c => $"{{{Symbol(c)}}}"))}";
            }

            // A row for one particular combination reads as the mana it would actually add, the way a chosen
            // any-one-colour row does.
            if (combination is { Count: > 0 })
            {
                adds += string.Concat(combination.Select(each => $"{{{Symbol(each)}}}"));
            }
            else if (option.ProducesOfAnyColorEach > 0)
            {
                // "In any combination of" is one choice per mana (CR 106.1b), so an unchosen row says how many and out
                // of what rather than naming a single colour.
                adds += (adds.Length > 0 ? " and " : string.Empty)
                    + $"{option.ProducesOfAnyColorEach} in any combination of "
                    + string.Join("/", option.ColorChoices.Select(c => $"{{{Symbol(c)}}}"));
            }

            if (adds.Length > 0)
            {
                // With more than one permanent able to pay the sacrifice, the cost line names the one this row would
                // take: "Sacrifice a Forest" is the same sentence on every row and says nothing about which.
                string cost = sacrificed.Length > 0 ? $"{option.CostText} ({sacrificed})" : option.CostText;
                if (option.Tap is { } tapped && !anyPermanent)
                {
                    cost += $" ({client.Object(tapped)?.Name ?? "permanent"})";
                }
                string text = cost.Length > 0 ? $"{cost}: Add {adds}" : $"{name}: add {adds}";

                List<string> notes = [];
                if (option.Filter != ManaSpendFilterView.None)
                {
                    notes.Add(option.Filter == ManaSpendFilterView.NotNonartifactSpells
                        ? "can't be spent on nonartifact spells"
                        : $"only for {Restricted(option.Filter)}");
                }

                if (option.Effect == ManaSpendEffectView.GainsHaste)
                {
                    notes.Add("a creature spell it pays for gains haste");
                }
                else if (option.Effect == ManaSpendEffectView.CantBeCountered)
                {
                    notes.Add("that spell can't be countered");
                }

                if (notes.Count > 0)
                {
                    text += $" ({string.Join(", and ", notes)})";
                }

                if (option.DamageToController > 0)
                {
                    text += $". Deals {option.DamageToController} damage to you";
                }

                if (option.LifeGained > 0)
                {
                    text += $". You gain {option.LifeGained} life";
                }

                return text + ellipsis;
            }

            // An ability that uses the stack reads as printed, since its cost and effect are what tell two apart.
            return option.Description.Length > 0 ? $"{option.Description}{ellipsis}" : $"Activate {name}{ellipsis}";
        }

        // Waterbend (CR 701.67a) and improvise (CR 702.126a): the permanent pays {1} of the cost by being tapped,
        // rather than making any mana. Which permanents are offered is the keyword's filter, not this line's.
        if (option.Kind == ActionKindView.TapToPay)
        {
            // Convoke (CR 702.51a) says what the creature could pay, which may be a colored mana as well as {1}.
            return option.Description.Length > 0 ? option.Description : $"Tap {name} to pay {{1}}";
        }

        // Delve (CR 702.66a): the graveyard card pays {1} of the cost by being exiled, rather than paying mana.
        if (option.Kind == ActionKindView.ExileToPay)
        {
            return $"Exile {name} to pay {{1}}";
        }

        // A face-down permanent has no name to give (CR 708.2a); the server says what turning it up costs.
        if (option.Kind == ActionKindView.TurnFaceUp)
        {
            return option.Description.Length > 0 ? option.Description : "Turn face up";
        }

        // Unlocking a Room's door (CR 709.5e): which door and what it costs are the whole of what tells this apart from
        // the Room's other abilities, and the server says both.
        if (option.Kind == ActionKindView.UnlockDoor)
        {
            return option.Description.Length > 0 ? option.Description : "Unlock a door";
        }

        // Taking your companion from outside the game (CR 116.2g): the server says which card and what it costs.
        if (option.Kind == ActionKindView.TakeCompanion)
        {
            return option.Description.Length > 0 ? option.Description : $"Put {name} into your hand";
        }

        // Plot (CR 702.170a): what it costs is the whole of what tells this apart from casting the card, and it says
        // where the card is going, since exiling it is half of what the action does.
        if (option.Kind == ActionKindView.Plot)
        {
            return option.Description.Length > 0
                ? $"Exile {name} to plot it {option.Description}"
                : $"Exile {name} to plot it";
        }

        // Suspend (CR 702.62a): exiling the card with its time counters, for its suspend cost, which the server says.
        if (option.Kind == ActionKindView.Suspend)
        {
            return option.Description.Length > 0
                ? $"Suspend {name}: exile it {option.Description}"
                : $"Suspend {name}";
        }

        // Foretell (CR 702.143a): the {2} the server says, and that the card goes away face down, which is the half of it
        // a player is deciding about.
        if (option.Kind == ActionKindView.Foretell)
        {
            return option.Description.Length > 0
                ? $"Foretell {name}: exile it face down {option.Description}"
                : $"Foretell {name}";
        }

        // A Phyrexian mana symbol (CR 107.4f) paid with life instead of the mana it names.
        if (option.Kind == ActionKindView.PayLifeForSymbol)
        {
            return $"Pay {option.LifeCost} life instead of {{{Symbol(option.Produces[0])}}}";
        }

        string verb = option.Kind switch
        {
            ActionKindView.PlayLand => "Play",
            ActionKindView.CastSpell => "Cast",
            _ => option.Kind.ToString(),
        };

        // A cast that puts a differently named spell on the stack reads as that spell: an adventurer card cast as its
        // Adventure (CR 715.3) casts Battle Display, not Embereth Shieldbreaker some other way, so the Adventure's name
        // is the whole of what there is to say. The server names it; the client never reads it out of the description.
        if (option.Kind == ActionKindView.CastSpell && option.SpellName.Length > 0)
        {
            return $"{verb} {option.SpellName}{ellipsis}";
        }

        // An alternative cost (CR 118.9) says what it costs, since that is all that tells it from casting for mana; so
        // does a card cast from outside a hand, as an airbended card is from exile for {2}.
        // A land played from outside a hand says where from, as the top of a library under Bolas's Citadel.
        if (option.Kind is ActionKindView.CastSpell or ActionKindView.PlayLand && option.Description.Length > 0)
        {
            return $"{verb} {name} — {option.Description}{ellipsis}";
        }

        return $"{verb} {name}{ellipsis}";
    }

    /// <summary>
    /// Whether taking this action will ask the player what to aim it at.
    /// <para>
    /// An action that requires a target but carries no candidates is sent as it is: the server only sends
    /// candidates it found, so an empty list means there was nothing to choose between. Which of those cases it
    /// is, is the server's to know and not the client's to reconstruct.
    /// </para>
    /// </summary>
    public static bool NeedsATarget(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);
        return option.TargetsRequired > 0 && option.TargetCandidates.Count > 0;
    }

    /// <summary>
    /// Whether an activation's one-mana choice is of colorless alone: a choice, with no colours listed. The server
    /// always lists the colours a choice may name, so this is only ever an ability choosing among the types a
    /// sacrificed permanent could produce (CR 106.7, Squandered Resources) for a candidate that could only add {C} —
    /// answered with <see cref="TakeActionAnswer.Mana"/>, since colorless is not a colour (CR 105.1).
    /// </summary>
    public static bool ChoosesColorless(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);
        return option.Kind == ActionKindView.ActivateAbility
            && option.ProducesOfAnyOneColor > 0
            && option.ColorChoices.Count == 0;
    }

    /// <summary>
    /// The rows a card's menu offers for these options: one per colour for an ability of any one colour (one {C} row
    /// for a choice of colorless alone), one per combination for "in any combination of" (CR 106.1b), each crossed
    /// with the permanent a cost sacrifices when there are several (CR 118.3). The colour, combination and sacrifice
    /// are part of the activation, so they are picked from the same menu rather than a second one opened behind it.
    /// </summary>
    public static List<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)> ManaRows(
        IEnumerable<ActionOptionMessage> offered)
    {
        ArgumentNullException.ThrowIfNull(offered);

        List<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)> rows = [];
        foreach (ActionOptionMessage option in offered)
        {
            List<(ColorView? Color, List<ColorView> Combination)> mana = option.ProducesOfAnyOneColor > 0
                ? ChoosesColorless(option)
                    ? [(null, [])]
                    : [.. option.ColorChoices.Select(color => ((ColorView?)color, new List<ColorView>()))]
                : option.ProducesOfAnyColorEach > 0
                    ? [.. ColorCombinations(option.ColorChoices, option.ProducesOfAnyColorEach)
                        .Select(combination => ((ColorView?)null, combination))]
                    : [(null, [])];

            List<ViewHandle?> sacrifices = option.SacrificeCandidates.Count > 0
                ? [.. option.SacrificeCandidates.Select(handle => (ViewHandle?)handle)]
                : [null];

            foreach ((ColorView? color, List<ColorView> combination) in mana)
            {
                rows.AddRange(sacrifices.Select(sacrifice => (option, color, combination, sacrifice)));
            }
        }

        return rows;
    }

    /// <summary>
    /// The permanent a row's cost takes, when the cost is one of the player's choosing: the artifact Urza, Lord High
    /// Artificer taps, the land Squandered Resources sacrifices. Null for a row whose cost names no such choice.
    /// </summary>
    public static ViewHandle? CostPermanent(
        (ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice) row) =>
        row.Option.Tap ?? row.Sacrifice;

    /// <summary>
    /// A card's menu rows gathered into what the menu offers: a row whose cost chooses no permanent stands alone, and
    /// the rows of one ability whose cost does are a single entry, since which permanent is then picked on the board
    /// the way a target is — never read off a list of names. The colour comes after, from the rows left for that one.
    /// </summary>
    public static List<List<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)>> MenuEntries(
        IEnumerable<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)> rows)
    {
        ArgumentNullException.ThrowIfNull(rows);

        List<List<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)>> entries = [];
        foreach (var row in rows)
        {
            List<(ActionOptionMessage Option, ColorView? Color, List<ColorView> Combination, ViewHandle? Sacrifice)>? same =
                CostPermanent(row) is null
                    ? null
                    : entries.FirstOrDefault(entry => CostPermanent(entry[0]) is not null && SameAbility(entry[0].Option, row.Option));

            if (same is null)
            {
                entries.Add([row]);
            }
            else
            {
                same.Add(row);
            }
        }

        return entries;
    }

    /// <summary>Whether two options are the same ability of the same source, differing only in what their choices are.</summary>
    public static bool SameAbility(ActionOptionMessage one, ActionOptionMessage other)
    {
        ArgumentNullException.ThrowIfNull(one);
        ArgumentNullException.ThrowIfNull(other);
        return one.Kind == other.Kind && one.Source == other.Source && one.AbilityIndex == other.AbilityIndex;
    }

    /// <summary>
    /// How a cost that takes a permanent of the player's choosing reads while it is being picked: what the permanent
    /// is for, as the verb of the dock's title. Tapping (Urza) or sacrificing (Squandered Resources).
    /// </summary>
    public static string CostVerb(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);
        return option.Tap is not null ? "tap" : "sacrifice";
    }

    /// <summary>
    /// Whether this is an activated ability the undo button cannot take back: one that uses the stack, as opposed to a
    /// mana ability, which the server records so it can be undone.
    /// </summary>
    public static bool IsIrreversibleActivation(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);
        return option.Kind == ActionKindView.ActivateAbility
            && option.Produces.Count == 0
            && option.ProducesOfAnyOneColor == 0
            && option.ProducesOfAnyColorEach == 0;
    }

    /// <summary>
    /// Whether taking this option still asks the player something that can be backed out of before anything happens:
    /// its targets, its X, or mana to pay for it (CR 602.2b), each of which comes with Cancel. An ability that asks
    /// none of these — Griselbrand's "Pay 7 life", a bare {T} — goes on the stack the moment it is taken.
    /// </summary>
    public static bool AsksBeforeActivating(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);
        return NeedsATarget(option) || option.HasX || CostsMana(option.CostText);
    }

    /// <summary>
    /// Whether a cost in oracle notation includes mana: any symbol but tap, untap and energy, so "{2}, {T}" does and
    /// "{T}, Pay 7 life" does not.
    /// </summary>
    public static bool CostsMana(string costText)
    {
        ArgumentNullException.ThrowIfNull(costText);
        return System.Text.RegularExpressions.Regex.Matches(costText, @"\{([^}]*)\}")
            .Any(symbol => symbol.Groups[1].Value is not ("T" or "Q" or "E"));
    }

    /// <summary>
    /// Every combination of <paramref name="count"/> colours out of <paramref name="colors"/>, for a mana ability that
    /// adds mana "in any combination of" them (CR 106.1b): one per multiset, since a pool is a bag and {R}{G} is the
    /// same mana as {G}{R}. Four for Orcish Lumberjack's three of two colours, six for Relic of Sauron's two of three.
    /// <para>
    /// Sorted so the uniform ones come first, which is what a player reaching for "all of one colour" wants at the top
    /// of the menu, and so the order is the same every time the card is clicked.
    /// </para>
    /// </summary>
    public static List<List<ColorView>> ColorCombinations(IReadOnlyList<ColorView> colors, int count)
    {
        ArgumentNullException.ThrowIfNull(colors);

        List<List<ColorView>> combinations = [[]];

        for (int i = 0; i < count; i++)
        {
            // Each colour may only be added at or after the last one taken, so every multiset is built exactly once.
            combinations = [.. combinations.SelectMany(taken => Enumerable.Range(0, colors.Count)
                .Where(at => taken.Count == 0 || at >= IndexOf(colors, taken[^1]))
                .Select(at => (List<ColorView>)[.. taken, colors[at]]))];
        }

        return [.. combinations.OrderBy(combination => combination.Distinct().Count())];
    }

    private static int IndexOf(IReadOnlyList<ColorView> colors, ColorView color)
    {
        for (int at = 0; at < colors.Count; at++)
        {
            if (colors[at] == color)
            {
                return at;
            }
        }

        return 0;
    }

    /// <summary>What restricted mana may be spent on, as a phrase.</summary>
    public static string Restricted(ManaSpendFilterView filter) => filter switch
    {
        ManaSpendFilterView.CreatureSpells => "creature spells",
        ManaSpendFilterView.InstantOrSorcerySpells => "instant and sorcery spells",
        ManaSpendFilterView.ActivatedAbilities => "activated abilities",
        ManaSpendFilterView.ArtifactSpells => "artifact spells",
        ManaSpendFilterView.LegendarySpells => "legendary spells",
        ManaSpendFilterView.NotNonartifactSpells => "anything but nonartifact spells",
        ManaSpendFilterView.PlaneswalkerSpells => "planeswalker spells",
        ManaSpendFilterView.LandAbilities => "abilities of land sources",
        ManaSpendFilterView.ArtifactAbilities => "abilities of artifact sources",
        _ => "anything",
    };

    /// <summary>The oracle symbol for a type of mana: "W", "C".</summary>
    public static string Symbol(ManaTypeView type) => type switch
    {
        ManaTypeView.White => "W",
        ManaTypeView.Blue => "U",
        ManaTypeView.Black => "B",
        ManaTypeView.Red => "R",
        ManaTypeView.Green => "G",
        _ => "C",
    };

    /// <summary>The oracle symbol for a color.</summary>
    public static string Symbol(ColorView color) => Symbol((ManaTypeView)(int)color);

    /// <summary>
    /// Whether taking this action asks something else first: a value for X. The color a mana ability adds is not
    /// asked afterwards; each color is a row of its own.
    /// </summary>
    public static bool NeedsAChoice(ActionOptionMessage option)
    {
        ArgumentNullException.ThrowIfNull(option);

        // A mana ability whose colours are chosen one per mana (CR 106.1b) asks a question of its own before it can be
        // taken, and so does a cost that taps or sacrifices a permanent of the player's choosing (CR 118.3).
        return option.HasX || option.ProducesOfAnyColorEach > 0 || option.SacrificeCandidates.Count > 0 || option.Tap is not null;
    }
}
