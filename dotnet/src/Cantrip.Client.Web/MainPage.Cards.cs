using Cantrip.Client.Presentation;
using Cantrip.Protocol;
using ManaRow = (Cantrip.Protocol.ActionOptionMessage Option, Cantrip.Protocol.ColorView? Color, System.Collections.Generic.List<Cantrip.Protocol.ColorView> Combination, Cantrip.Protocol.ViewHandle? Sacrifice);

namespace Cantrip.Client.Web;

/// <summary>
/// What each card on the table does when clicked, and how it looks while the question it answers is open, over the
/// shared <see cref="PromptSelection"/> and <see cref="CardActions"/>. A card is clickable because the outstanding prompt named it, never because this client
/// worked out that it ought to be.
/// </summary>
public sealed partial class MainPage
{
    /// <summary>How a card is drawn and what a click on it does.</summary>
    private sealed record CardLook(
        bool Selected = false,
        bool Actionable = false,
        bool Dimmed = false,
        Action? Clicked = null,
        string? Note = null,
        string? Annotation = null);

    private static readonly CardLook Plain = new();

    private readonly PromptSelection _selection = new();
    private List<int> _picked => _selection.Cards;
    private List<int> _pickedModes => _selection.Modes;
    private List<int> _pickedColors => _selection.Colors;
    private List<TargetMessage> _pickedTargets => _selection.Targets;
    private List<int> _triggerOrder => _selection.TriggerOrder;
    private Dictionary<int, AttackChoice> _attacking => _selection.Attacking;
    private Dictionary<int, int> _blocking => _selection.Blocking;
    private List<int> _pending => _selection.Pending;
    private Dictionary<(int Source, int Creature), int> _shares => _selection.DamageShares;
    private Dictionary<int, int> _dividedShares => _selection.DividedShares;
    private ActionOptionMessage? _targeting { get => _selection.Targeting; set => _selection.Targeting = value; }

    // Rebuilt from the outstanding prompt on every redraw, so no stale offer can survive into a later decision.
    private CardActions _actions = new(null);
    private readonly Dictionary<int, TargetMessage> _targetableObjects = [];
    private readonly HashSet<int> _eligible = [];
    private Dictionary<int, List<ManaRow>> _costTakes = [];

    /// <summary>Each card's look for this redraw, by handle, so a click finds what its card offered when it was drawn.</summary>
    private readonly Dictionary<ViewHandle, CardLook> _looks = [];

    private readonly CardMenu _menu = new();

    /// <summary>Where the pointer last went down, in the page's coordinates, for a card's menu to open at.</summary>
    private Windows.Foundation.Point _pointer;

    /// <summary>Whether Shift was held as the pointer went down: the Auto-mana key, held (Hotkeys: AutoMana is Shift).</summary>
    private bool _autoManaHeld;

    /// <summary>
    /// Re-indexes whatever the server is currently asking, so the board can be drawn from it.
    /// </summary>
    private void ReadPrompt(GameClient client)
    {
        _actions = new CardActions(client.Prompt?.Prompt);
        _targetableObjects.Clear();
        _eligible.Clear();
        _looks.Clear();
        _costTakes = CostCandidates();

        if (AimCandidates(client) is { } aiming)
        {
            foreach (TargetMessage candidate in aiming)
            {
                if (!candidate.IsPlayer)
                {
                    _targetableObjects[candidate.Object.Value] = candidate;
                }
            }
        }

        switch (client.Prompt?.Prompt)
        {
            case DeclareAttackersPromptMessage attackers:
                _eligible.UnionWith(attackers.Eligible.Select(handle => handle.Value));
                break;

            case DeclareBlockersPromptMessage blockers:
                _eligible.UnionWith(blockers.Eligible.Select(handle => handle.Value));
                break;

            case ChooseCardsPromptMessage choose when CardChoicePresentation.ChoiceTable(client, choose) is not null:
                _eligible.UnionWith(choose.Eligible.Select(handle => handle.Value));
                break;

            case ChooseDividedTargetsPromptMessage divide:
                _eligible.UnionWith(divide.Candidates.Where(candidate => !candidate.IsPlayer).Select(candidate => candidate.Object.Value));
                break;

            case AssignCombatDamagePromptMessage assign when _shares.Count == 0:
                // Seeded from the engine's own division, which is already legal, so confirming without touching it is
                // a valid answer.
                foreach (DamageDivisionMessage division in assign.Divisions)
                {
                    for (int i = 0; i < division.Creatures.Count; i++)
                    {
                        _shares[(division.Source.Value, division.Creatures[i].Value)] = i < division.Suggested.Count ? division.Suggested[i] : 0;
                    }

                    if (division.MayAssignToDefender)
                    {
                        _shares[(division.Source.Value, 0)] = division.SuggestedToDefender;
                    }
                }

                break;

            default:
                break;
        }
    }

    /// <summary>A card's look for this redraw, worked out once and remembered for its click.</summary>
    private CardLook LookOf(GameClient client, ObjectView card)
    {
        if (!_looks.TryGetValue(card.Handle, out CardLook? look))
        {
            look = Build(client, card);
            if (look.Note is null && TuckedNote(client, card) is { } note)
            {
                look = look with { Note = note };
            }

            if (look.Annotation is null && card.Zone == ZoneKindView.Battlefield && OnAnnotation(client, card) is { } on)
            {
                look = look with { Annotation = on };
            }

            if (OwnedAnnotation(client, card) is { } owned)
            {
                // Under whatever else is written over the art, a single chosen quality included, which it would hide.
                string? above = look.Annotation ?? (card.Choices.Count == 1 ? card.Choices[0].Value : null);
                look = look with { Annotation = above is null ? owned : $"{above}\n{owned}" };
            }

            // A card in exile or a graveyard that may be played says until when, and a flashback that it is exiled after.
            if (PermissionAnnotation(card) is { } window)
            {
                look = look with { Annotation = look.Annotation is null ? window : $"{look.Annotation}\n{window}" };
            }

            _looks[card.Handle] = look;
        }

        return look;
    }

    /// <summary>
    /// What a card in exile or a graveyard may be played for, and until when (CR 601.3, 305.1): one line for each permission
    /// it stands under. Null for a card nobody may play from where it is.
    /// </summary>
    private static string? PermissionAnnotation(ObjectView card) =>
        card.PlayPermissions.Count > 0 && card.Zone is ZoneKindView.Exile or ZoneKindView.Graveyard
            ? string.Join("\n", card.PlayPermissions.Select(permission => permission.Text))
            : null;

    /// <summary>A card placed on one of the rows, drawn as its look says.</summary>
    private CardBoardElement.BoardCell Cell(GameClient client, ObjectView card, float left, float top, bool tapped)
    {
        CardLook look = LookOf(client, card);
        bool dividing = _wornShares.TryGetValue(card.Handle, out WornShare worn);
        return new CardBoardElement.BoardCell(
            card,
            left,
            top,
            tapped,
            look.Actionable,
            look.Dimmed,
            look.Selected,
            Clickable: look.Clicked is not null,
            look.Annotation,
            DamageShare: dividing ? worn.Share : null,
            LowerLive: dividing && worn.Lower,
            RaiseLive: dividing && worn.Raise,
            ShareCounters: dividing && worn.Counters);
    }

    private void OnCardClicked(ViewHandle handle)
    {
        if (_gameClient is null || !_looks.TryGetValue(handle, out CardLook? look) || look.Clicked is not { } clicked)
        {
            return;
        }

        clicked();
        RedrawNow();
    }

    /// <summary>What the outstanding prompt and the answer being built make of one card.</summary>
    private CardLook Build(GameClient client, ObjectView card)
    {
        int handle = card.Handle.Value;

        // Aiming comes first: while something is being aimed, nothing else on the board is a control.
        if (AimCandidates(client) is not null)
        {
            bool aimable = _targetableObjects.TryGetValue(handle, out TargetMessage? target);

            return new CardLook(
                Selected: aimable && IsPickedTarget(target!),
                Actionable: aimable,
                Dimmed: !aimable,
                Clicked: aimable ? () => Aim(client, target!) : null);
        }

        // Choosing the permanent a cost takes is aiming by another name: the candidates are lit and nothing else answers.
        if (_selection.CostChoice is not null)
        {
            bool takes = _costTakes.TryGetValue(handle, out List<ManaRow>? rows);
            return new CardLook(Actionable: takes, Dimmed: !takes, Clicked: takes ? () => PickCostPermanent(client, rows!) : null);
        }

        return client.Prompt?.Prompt switch
        {
            // A triggered ability put on the stack by a click that has not been sent yet is clicked again to take it back
            // off, and the ones put on after it stay in their order.
            OrderTriggersPromptMessage ordering when card.Zone == ZoneKindView.Stack && handle < 0 => new CardLook(
                Actionable: true,
                Note: "Click to take it back off the stack.",
                Clicked: () =>
                {
                    _triggerOrder.RemoveAll(index => TriggerStandIn(client, ordering, index).Handle.Value == handle);
                    _needsRedraw = true;
                }),

            // Discarding to hand size asks for a number of cards: once that many are picked the rest of the hand stops
            // being clickable. With one to discard, clicking another card trades rather than adding.
            DiscardPromptMessage discard when card.Zone == ZoneKindView.Hand && card.ZoneOwner == client.Seat =>
                new CardLook(
                    Selected: _picked.Contains(handle),
                    Actionable: _picked.Contains(handle) || _selection.CanPick(discard.Count),
                    Clicked: _picked.Contains(handle) || _selection.CanPick(discard.Count) ? () => Pick(handle, discard.Count) : null),

            // A choice among cards already in view is made where they are; the rest of that zone is dimmed.
            ChooseCardsPromptMessage choose when CardChoicePresentation.ChoiceTable(client, choose) is { } table && InTable(client, card, table) =>
                new CardLook(
                    Selected: _picked.Contains(handle),
                    Dimmed: !_eligible.Contains(handle),
                    Actionable: _eligible.Contains(handle) && !_picked.Contains(handle) && _selection.CanPick(choose.Count),
                    Clicked: _eligible.Contains(handle) && (_picked.Contains(handle) || _selection.CanPick(choose.Count))
                        ? () => PickOnBoard(client, choose, handle)
                        : null,
                    Annotation: PutBackAnnotation(choose, handle)),

            DeclareAttackersPromptMessage attackers when _eligible.Contains(handle) =>
                new CardLook(
                    Selected: _attacking.ContainsKey(handle) || _pending.Contains(handle),
                    Actionable: true,
                    Clicked: () => ClickAttacker(attackers, handle)),

            // With creatures picked up, the planeswalkers and battles one of them may attack are what to click.
            DeclareAttackersPromptMessage attackers
                when _pending.Count > 0 && _pending.Any(creature => MayAttack(attackers, creature, new AttackChoice(0, card.Handle))) =>
                new CardLook(Actionable: true, Clicked: () => AssignAttack(client, attackers, new AttackChoice(0, card.Handle))),

            // A blocker already against an attacker comes back off it when clicked; one that is not is picked up, and
            // the attackers become the things to click.
            DeclareBlockersPromptMessage when _eligible.Contains(handle) && _blocking.ContainsKey(handle) =>
                new CardLook(Selected: true, Actionable: true, Clicked: () => Unassign(handle)),

            DeclareBlockersPromptMessage when _eligible.Contains(handle) =>
                new CardLook(Selected: _pending.Contains(handle), Actionable: true, Clicked: () => PickUp(handle)),

            DeclareBlockersPromptMessage blockers
                when _pending.Count > 0
                    && blockers.Attackers.IndexOf(card.Handle) is >= 0 and var target
                    && _pending.Any(blocker => MayBlock(blockers, blocker, card.Handle)) =>
                new CardLook(Actionable: true, Clicked: () => AssignBlocks(client, blockers, target)),

            // An attacker none of the held blockers may block, such as a flier against creatures without flying or reach.
            DeclareBlockersPromptMessage blockers
                when _pending.Count > 0 && blockers.Attackers.Contains(card.Handle) =>
                new CardLook(Dimmed: true, Note: "Can't be blocked by " + (_pending.Count == 1 ? "that creature." : "those creatures.")),

            // The spell being paid for. Not a control, but it says what it is waiting on - unless it can help pay.
            PayCostsPromptMessage pay when pay.Spell == card.Handle && _actions.For(card.Handle).Count == 0 =>
                new CardLook(Note: card.Zone switch
                {
                    ZoneKindView.Battlefield => $"Being activated. {pay.Remaining} still to pay.",
                    ZoneKindView.Stack => $"{pay.Remaining} still to pay.",
                    _ => $"Being cast. {pay.Remaining} still to pay.",
                }),

            // A land offered while paying is clicked exactly as it is from a priority prompt.
            PriorityPromptMessage or PayCostsPromptMessage or UnlessPaysPromptMessage
                when _actions.For(card.Handle) is { Count: > 0 } offered =>
                new CardLook(Actionable: true, Clicked: () => Choose(client, offered)),

            DeclareAttackersPromptMessage or DeclareBlockersPromptMessage =>
                new CardLook(Dimmed: card.Controller == client.Seat && card.CardTypes.Contains(CardTypeView.Creature)),

            // A divided slot's candidates wear their shares (CR 601.2d); a click on one adds to its share, as its + does.
            ChooseDividedTargetsPromptMessage divide when divide.Candidates.Any(candidate => !candidate.IsPlayer && candidate.Object == card.Handle) =>
                new CardLook(
                    Selected: _dividedShares.GetValueOrDefault(handle) > 0,
                    Actionable: true,
                    Clicked: () => OnShareStepped(card.Handle, raise: true)),

            ChooseDividedTargetsPromptMessage when card.Zone == ZoneKindView.Battlefield => new CardLook(Dimmed: true),

            _ => Plain,
        };
    }

    private static bool InTable(GameClient client, ObjectView card, ZoneKindView table) =>
        card.Zone == table && (table != ZoneKindView.Hand || card.ZoneOwner == client.Seat);

    private string? PutBackAnnotation(ChooseCardsPromptMessage prompt, int handle) =>
        prompt.Purpose is CardChoicePurposeView.PutOnTop or CardChoicePurposeView.Splice
            && _picked.IndexOf(handle) is >= 0 and var at
            ? prompt.Purpose == CardChoicePurposeView.Splice ? $"Effect {at + 1}"
                : at == prompt.Count - 1 ? "On top" : $"{at + 1}"
            : null;

    /// <summary>
    /// Acts on a card. One thing to offer does it; more than one opens the menu. An ability that asks
    /// nothing opens the menu even alone, since nothing else stands between the click and the stack.
    /// </summary>
    private void Choose(GameClient client, IReadOnlyList<ActionOptionMessage> offered)
    {
        if (client.Prompt is null)
        {
            return;
        }

        List<ManaRow> rows = CardActions.ManaRows(offered);
        var entries = CardActions.MenuEntries(rows);

        // One thing to do, and it wants a permanent: straight to picking it, whatever auto-mana would say.
        if (entries.Count == 1 && CardActions.CostPermanent(entries[0][0]) is not null)
        {
            ChooseCostPermanent(entries[0]);
            return;
        }

        if (rows.Count == 1
            && (!CardActions.IsIrreversibleActivation(rows[0].Option) || CardActions.AsksBeforeActivating(rows[0].Option)))
        {
            Take(client, rows[0].Option, rows[0].Color, rows[0].Combination, rows[0].Sacrifice);
            return;
        }

        if (AutoManaPick(client, rows) is { } picked)
        {
            if (CardActions.CostPermanent(picked) is not null)
            {
                ChooseCostPermanent(entries.First(entry => entry.Any(row => CardActions.SameAbility(row.Option, picked.Option))));
            }
            else
            {
                Take(client, picked.Option, picked.Color, picked.Combination, picked.Sacrifice);
            }

            return;
        }

        List<(string, Action)> items = [];
        foreach (var entry in entries)
        {
            (ActionOptionMessage option, ColorView? color, List<ColorView> combination, ViewHandle? sacrifice) = entry[0];
            items.Add(CardActions.CostPermanent(entry[0]) is not null
                ? (CostChoiceLabel(client, entry), () => { ChooseCostPermanent(entry); RedrawNow(); })
                : (CardActions.Describe(client, option, color, combination, Sacrificed(client, sacrifice)),
                    () => { Take(client, option, color, combination, sacrifice); RedrawNow(); }));
        }

        _menu.Open(Root, _pointer, items, abilities: [.. entries.Select(entry => entry[0].Option.AbilityIndex)]);
    }

    /// <summary>Whether auto-mana answers a card's menu: on by default or not, and Shift held turns it the other way.</summary>
    private bool AutoManaWanted() => PlayerSettings.AutoManaByDefault != _autoManaHeld;

    /// <summary>What auto-mana would take from these rows, or null when it is not wanted, nothing is owed, or it has no pick.</summary>
    private ManaRow? AutoManaPick(GameClient client, List<ManaRow> rows)
    {
        if (!AutoManaWanted() || ManaOwed.Of(client.Prompt?.Prompt) is not { Any: true } owed)
        {
            return null;
        }

        var uniform = rows.Where(row => row.Combination.Distinct().Count() <= 1).ToList();
        if (AutoMana.Pick(
                [.. uniform.Select(row => (row.Option, row.Color ?? (row.Combination.Count > 0 ? row.Combination[0] : null)))],
                _actions.Offered,
                owed) is not { } picked)
        {
            return null;
        }

        ColorView? color = picked.Option.ProducesOfAnyOneColor > 0 ? picked.Color : null;
        List<ColorView> combination = picked.Option.ProducesOfAnyColorEach > 0 && picked.Color is { } each
            ? [.. Enumerable.Repeat(each, picked.Option.ProducesOfAnyColorEach)]
            : [];

        foreach (var row in uniform)
        {
            if (ReferenceEquals(row.Option, picked.Option) && row.Color == color && row.Combination.SequenceEqual(combination))
            {
                return row;
            }
        }

        return (picked.Option, color, combination, null);
    }

    private void ChooseCostPermanent(List<ManaRow> entry)
    {
        _selection.CostChoice = [.. entry.Select(row => row.Option).Distinct()];
        _needsRedraw = true;
    }

    private Dictionary<int, List<ManaRow>> CostCandidates()
    {
        Dictionary<int, List<ManaRow>> candidates = [];
        if (_selection.CostChoice is not { } options)
        {
            return candidates;
        }

        foreach (var row in CardActions.ManaRows(options))
        {
            if (CardActions.CostPermanent(row) is { } permanent)
            {
                if (!candidates.TryGetValue(permanent.Value, out var taking))
                {
                    candidates[permanent.Value] = taking = [];
                }

                taking.Add(row);
            }
        }

        return candidates;
    }

    private void PickCostPermanent(GameClient client, List<ManaRow> rows)
    {
        _selection.CostChoice = null;
        _needsRedraw = true;

        if (rows.Count == 1)
        {
            Take(client, rows[0].Option, rows[0].Color, rows[0].Combination, rows[0].Sacrifice);
            return;
        }

        if (AutoManaPick(client, rows) is { } picked)
        {
            Take(client, picked.Option, picked.Color, picked.Combination, picked.Sacrifice);
            return;
        }

        List<(string, Action)> items = [];
        foreach ((ActionOptionMessage option, ColorView? color, List<ColorView> combination, ViewHandle? sacrifice) in rows)
        {
            items.Add((
                CardActions.Describe(client, option, color, combination, Sacrificed(client, sacrifice)),
                () => { Take(client, option, color, combination, sacrifice); RedrawNow(); }));
        }

        _menu.Open(Root, _pointer, items, abilities: [.. rows.Select(row => row.Option.AbilityIndex)]);
    }

    private static string CostChoiceLabel(GameClient client, List<ManaRow> entry)
    {
        List<string> readings =
            [.. entry.Select(row => CardActions.Describe(client, row.Option, row.Color, row.Combination, anyPermanent: true)).Distinct()];
        return readings.Count == 1 ? readings[0] : $"{entry[0].Option.CostText}: Add mana...";
    }

    /// <summary>Takes an action, first asking which target when the server offered a choice of them.</summary>
    private void Take(
        GameClient client,
        ActionOptionMessage option,
        ColorView? color = null,
        IReadOnlyList<ColorView>? combination = null,
        ViewHandle? sacrifice = null)
    {
        if (CardActions.NeedsATarget(option))
        {
            _targeting = option;
            PeekAtTargets(client, option.TargetCandidates);
            _needsRedraw = true;
            return;
        }

        TakeActionAnswer answer = AnswerFor(option);
        answer.Color = color;
        answer.Colors = [.. combination ?? []];
        answer.Sacrifice = sacrifice;

        // Colorless is not a colour (CR 105.1), so a choice of colorless alone is answered as the mana type instead.
        if (CardActions.ChoosesColorless(option))
        {
            answer.Mana = ManaTypeView.Colorless;
        }

        Send(client, answer);
    }

    private static string Sacrificed(GameClient client, ViewHandle? sacrifice) =>
        sacrifice is { } handle ? client.Object(handle)?.Name ?? string.Empty : string.Empty;

    /// <summary>What can be aimed at right now: a spell being announced, or a triggered ability's target prompt.</summary>
    private IReadOnlyList<TargetMessage>? AimCandidates(GameClient client) =>
        _targeting?.TargetCandidates
        ?? (client.Prompt?.Prompt as ChooseTargetPromptMessage)?.Candidates;

    private void PickOnBoard(GameClient client, ChooseCardsPromptMessage choose, int handle)
    {
        Pick(handle, choose.Count);

        CardChoicePresentation model = CardChoicePresentation.Describe(client, choose, _picked);
        if (model.AnswersOnLastPick && model.CanSubmit)
        {
            Send(client, new CardsChosenAnswer { Cards = [.. _picked.Select(h => new ViewHandle(h))] });
        }
    }

    /// <summary>
    /// Aims at a target. An exact number answers itself on the click that reaches it; "up to" a number
    /// collects clicks until the dock's button. Clicking a picked one puts it back; with one to choose, another trades.
    /// </summary>
    private void Aim(GameClient client, TargetMessage target)
    {
        (int count, bool upTo) = _targeting is { } announced
            ? (1, announced.TargetsOptional)
            : client.Prompt?.Prompt is ChooseTargetPromptMessage asked ? (asked.Count, asked.Optional || asked.Minimum > 0) : (1, false);

        if (_pickedTargets.RemoveAll(picked => Same(picked, target)) == 0)
        {
            if (count == 1)
            {
                _pickedTargets.Clear();
            }

            if (_pickedTargets.Count < count)
            {
                _pickedTargets.Add(target);
            }
        }

        if (upTo || _pickedTargets.Count < count)
        {
            _needsRedraw = true;
            return;
        }

        SendTargets(client);
    }

    private void SendTargets(GameClient client)
    {
        Send(client, _targeting is { } announced
            ? AnswerFor(announced, _pickedTargets.Count > 0 ? _pickedTargets[0] : null)
            : new TargetsChosenAnswer { Targets = [.. _pickedTargets] });
    }

    private static bool Same(TargetMessage one, TargetMessage other) =>
        one.Player == other.Player && one.Object == other.Object;

    private bool IsPickedTarget(TargetMessage candidate) => _pickedTargets.Any(picked => Same(picked, candidate));

    /// <summary>
    /// Whether the spell being paid for is cast by web-slinging (CR 702.188a): its tapped creature is chosen once the mana is
    /// paid, so the payment's caption says to tap a creature for mana while none is tapped, or the cast is taken back.
    /// </summary>
    private bool _webSlinging;

    private static TakeActionAnswer AnswerFor(ActionOptionMessage option, TargetMessage? target = null) => new()
    {
        Kind = option.Kind,
        Source = option.Source,
        AbilityIndex = option.AbilityIndex,
        Tap = option.Tap,
        Targets = target is null ? [] : [target],
        Mana = option.Kind is ActionKindView.AssignMana or ActionKindView.PayLifeForSymbol ? option.Produces[0] : null,
        Filter = option.Filter,
        Effect = option.Effect,
    };

    private void Send(GameClient client, AnswerPayload answer)
    {
        if (client.Prompt is null)
        {
            return;
        }

        // Whether the cast just chosen is by web-slinging (CR 702.188a), for the payment's caption to say what it needs.
        if (client.Prompt.Prompt is PriorityPromptMessage priority)
        {
            _webSlinging = answer is TakeActionAnswer taken && priority.Options.Exists(option => AutoPlay.IsWebSlinging(option)
                && option.Source == taken.Source && option.AbilityIndex == taken.AbilityIndex && option.Kind == taken.Kind);
        }

        client.Answer(answer);
        _selection.Clear();
        _needsRedraw = true;
    }

    private void Pick(int handle, int limit)
    {
        _selection.Pick(handle, limit);
        _needsRedraw = true;
    }

    /// <summary>
    /// Cycles an attacker through not attacking, then each player it could attack in turn, then each planeswalker they
    /// control (CR 508.1b).
    /// </summary>
    private void CycleAttack(int handle, List<AttackChoice> defenders)
    {
        if (!_attacking.TryGetValue(handle, out AttackChoice defender))
        {
            _attacking[handle] = defenders[0];
        }
        else
        {
            int next = defenders.IndexOf(defender) + 1;

            if (next >= defenders.Count)
            {
                _attacking.Remove(handle);
            }
            else
            {
                _attacking[handle] = defenders[next];
            }
        }

        _needsRedraw = true;
    }

    /// <summary>What one creature may be sent at: every offered seat and planeswalker but those closed to it (CR 508.1c).</summary>
    private static List<AttackChoice> AttackChoices(DeclareAttackersPromptMessage prompt, ViewHandle attacker) =>
        [.. prompt.AttacksFor(attacker).Select(attack => new AttackChoice(attack.TargetPlayer, attack.TargetPermanent))];

    /// <summary>Picks a creature up, or puts it back down, to be sent somewhere along with the others picked up.</summary>
    private void PickUp(int creature)
    {
        if (!_pending.Remove(creature))
        {
            _pending.Add(creature);
        }

        _needsRedraw = true;
    }

    /// <summary>Whether a creature may be declared making an attack (CR 508.1c).</summary>
    private static bool MayAttack(DeclareAttackersPromptMessage prompt, int attacker, AttackChoice choice) =>
        prompt.AttacksFor(new ViewHandle(attacker)).Any(attack => attack.TargetPlayer == choice.Player && attack.TargetPermanent == choice.Permanent);

    private static bool MayBlock(DeclareBlockersPromptMessage prompt, int blocker, ViewHandle attacker) =>
        prompt.Allowed.Any(pair => pair.Blocker.Value == blocker && pair.Attacker == attacker);

    /// <summary>
    /// Whether the attackers have a choice of what to attack: more than one seat, planeswalker or battle that some
    /// creature may be sent at. With none, a click on a creature is all it takes (<see cref="CycleAttack"/>).
    /// </summary>
    private static bool ChoosesWhatToAttack(DeclareAttackersPromptMessage prompt) =>
        prompt.Eligible.Any(creature => prompt.AttacksFor(creature).Count > 1);

    /// <summary>
    /// A click on a creature able to attack. Where there is one thing to attack the creature is sent at it, and a
    /// second click takes it back; where there are several it is picked up with the others and the click on what to
    /// attack declares them all.
    /// </summary>
    private void ClickAttacker(DeclareAttackersPromptMessage prompt, int handle)
    {
        if (!ChoosesWhatToAttack(prompt))
        {
            CycleAttack(handle, AttackChoices(prompt, new ViewHandle(handle)));
        }
        else if (_attacking.Remove(handle))
        {
            _needsRedraw = true;
        }
        else
        {
            PickUp(handle);
        }
    }

    /// <summary>Sends every creature picked up that may at the chosen seat, planeswalker or battle, and the arrows to it go in.</summary>
    private void AssignAttack(GameClient client, DeclareAttackersPromptMessage prompt, AttackChoice choice)
    {
        List<int> sent = [.. _pending.Where(creature => MayAttack(prompt, creature, choice))];
        RetireArrows(sent, AttackBounds(client, choice));

        foreach (int creature in sent)
        {
            _attacking[creature] = choice;
            _pending.Remove(creature);
        }

        _needsRedraw = true;
    }

    /// <summary>Has every blocker picked up that may block the attacker do so, and the arrows to it go in.</summary>
    private void AssignBlocks(GameClient client, DeclareBlockersPromptMessage prompt, int attackerIndex)
    {
        ViewHandle attacker = prompt.Attackers[attackerIndex];
        List<int> sent = [.. _pending.Where(blocker => MayBlock(prompt, blocker, attacker))];
        RetireArrows(sent, CardBounds(attacker));

        foreach (int blocker in sent)
        {
            _blocking[blocker] = attackerIndex;
            _pending.Remove(blocker);
        }

        _needsRedraw = true;
    }

    /// <summary>Puts down whatever was picked up.</summary>
    private void PutDown()
    {
        _pending.Clear();
        _needsRedraw = true;
    }

    private void Unassign(int blocker)
    {
        _blocking.Remove(blocker);
        _needsRedraw = true;
    }

    /// <summary>
    /// Whether a land has been spent: tapped, and nothing this client is offering or asking about it,
    /// so it joins the one pile of spent lands.
    /// </summary>
    private bool IsSpent(ObjectView land)
    {
        int handle = land.Handle.Value;

        return land.Tapped
            && TuckedLikeness(land).Length == 0
            && _actions.For(land.Handle).Count == 0
            && !_eligible.Contains(handle)
            && !_targetableObjects.ContainsKey(handle)
            && !_picked.Contains(handle);
    }

    /// <summary>The cards in exile that a permanent is holding "until it leaves the battlefield" (CR 610.3).</summary>
    private static IEnumerable<ObjectView> HeldBy(GameClient client, ViewHandle holder) => Holdings(client).Held[holder];

    /// <summary>The permanents attached to this one: the Auras enchanting it and the Equipment attached to it (CR 301.5c, 303.4).</summary>
    private static IEnumerable<ObjectView> WornBy(GameClient client, ViewHandle host) => Holdings(client).Worn[host];

    private static (PlayerView? View, ILookup<ViewHandle, ObjectView> Held, ILookup<ViewHandle, ObjectView> Worn) _holdings;

    /// <summary>
    /// What each permanent holds in exile and has attached to it, gathered once per view: every permanent on a redraw
    /// asks, and a pass over the zone per ask made a board of two hundred permanents cost forty thousand looks a redraw.
    /// </summary>
    private static (PlayerView? View, ILookup<ViewHandle, ObjectView> Held, ILookup<ViewHandle, ObjectView> Worn) Holdings(GameClient client)
    {
        if (!ReferenceEquals(_holdings.View, client.View) || _holdings.Held is null)
        {
            _holdings = (
                client.View,
                client.Zone(ZoneKindView.Exile, 0).Where(card => card.ExiledWith != ViewHandle.None).ToLookup(card => card.ExiledWith),
                client.Battlefield().Where(card => card.AttachedTo != ViewHandle.None).ToLookup(card => card.AttachedTo));
        }

        return _holdings;
    }

    /// <summary>
    /// Whether this permanent is drawn under the one it is attached to rather than standing in a row of its own. An Aura on a card in a graveyard has no permanent to be drawn under, so it keeps its place.
    /// </summary>
    private static bool Nested(GameClient client, ObjectView card) =>
        card.AttachedTo != ViewHandle.None
        && client.Object(card.AttachedTo) is { Zone: ZoneKindView.Battlefield };

    /// <summary>
    /// Everything tucked under a permanent, furthest back first: the cards it holds in exile, then
    /// each thing attached to it after whatever that holds in turn.
    /// </summary>
    private static List<ObjectView> Tucked(GameClient client, ObjectView holder)
    {
        List<ObjectView> under = [.. HeldBy(client, holder.Handle)];
        foreach (ObjectView worn in WornBy(client, holder.Handle))
        {
            under.AddRange(Tucked(client, worn));
            under.Add(worn);
        }

        return under;
    }

    /// <summary>What a tucked card says about where it is, when its prompt has nothing to say.</summary>
    private static string? TuckedNote(GameClient client, ObjectView card)
    {
        if (Nested(client, card) && client.Object(card.AttachedTo) is { } host)
        {
            return $"Attached to {host.Name}.";
        }

        if (card.Zone == ZoneKindView.Exile && card.ExiledWith != ViewHandle.None
            && client.Object(card.ExiledWith) is { Zone: ZoneKindView.Battlefield } holder)
        {
            return card.Kind == ObjectKindView.CardCopy
                ? $"Prepared by {holder.Name}. Cast it from here for as long as {holder.Name} stays prepared on the battlefield."
                : $"Exiled by {holder.Name} until it leaves the battlefield.";
        }

        return null;
    }

    /// <summary>
    /// What an attachment standing in a row of its own says it is on: the card it enchants when that
    /// is not a permanent, or the player.
    /// </summary>
    private static string? OnAnnotation(GameClient client, ObjectView card)
    {
        if (card.AttachedTo != ViewHandle.None && !Nested(client, card) && client.Object(card.AttachedTo) is { } host)
        {
            return $"On {host.Name}";
        }

        if (card.AttachedToPlayer != 0)
        {
            PlayerSummaryView? player = client.View?.Players.FirstOrDefault(seat => seat.Seat == card.AttachedToPlayer);
            return $"On {player?.Name ?? $"Seat {card.AttachedToPlayer}"}";
        }

        // CR 310.9: a battle's protector, who defends it and alone may not attack it.
        if (card.Protector != 0)
        {
            return card.Protector == client.Seat ? "You protect it" : $"Protected by {NameOf(client, card.Protector)}";
        }

        return null;
    }

    /// <summary>Who owns a permanent or spell its controller does not (CR 108.3), or null for one controlled by its owner.</summary>
    private static string? OwnedAnnotation(GameClient client, ObjectView card)
    {
        if (card.Zone is not (ZoneKindView.Battlefield or ZoneKindView.Stack) || card.Owner == 0 || card.Owner == card.Controller)
        {
            return null;
        }

        if (card.Owner == client.Seat)
        {
            return "Owned by you";
        }

        PlayerSummaryView? owner = client.View?.Players.FirstOrDefault(seat => seat.Seat == card.Owner);
        return $"Owned by {owner?.Name ?? $"Seat {card.Owner}"}";
    }

    /// <summary>
    /// The handles of what is tucked under a permanent, so one holding something never piles with a twin that holds
    /// nothing, or something else. Empty for one holding nothing.
    /// </summary>
    private string TuckedLikeness(ObjectView card) =>
        _gameClient is null || card.Zone != ZoneKindView.Battlefield
            ? string.Empty
            : string.Join(",", Tucked(_gameClient, card).Select(under => under.Handle.Value));

    /// <summary>What the outstanding prompt and the half-made answer say about a card, for telling twins apart.</summary>
    private string PromptLikeness(ObjectView card)
    {
        int handle = card.Handle.Value;
        return string.Concat(
            TuckedLikeness(card),
            "|",
            _eligible.Contains(handle) ? "e" : "-",
            _targetableObjects.TryGetValue(handle, out TargetMessage? aimed) ? IsPickedTarget(aimed) ? "T" : "t" : "-",
            _selection.CostChoice is null ? "-" : _costTakes.ContainsKey(handle) ? "c" : "x",
            _picked.Contains(handle) ? "p" : "-",
            _pending.Contains(handle) ? "a" : "-",
            _blocking.TryGetValue(handle, out int blocking) ? blocking.ToString(System.Globalization.CultureInfo.InvariantCulture) : "-1",
            "|",
            (_shares.Where(pair => pair.Key.Creature == handle).Sum(pair => pair.Value) + _dividedShares.GetValueOrDefault(handle)).ToString(System.Globalization.CultureInfo.InvariantCulture),
            "|",
            _actions.For(card.Handle).Count.ToString(System.Globalization.CultureInfo.InvariantCulture));
    }
}
