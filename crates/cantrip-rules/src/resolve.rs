//! Resolving the top of the stack (CR 608), and countering (CR 701.6).

use cantrip_api::*;

use crate::layers::{self, live};
use crate::{Cx, Flow, R, ops};

/// The card whose characteristics a spell on the stack has: its Adventure's while it is cast as one (CR 715.3b).
fn face(state: &GameState, spell: ObjectId) -> CardId {
    let obj = state.obj(spell);
    obj.copy.map_or(obj.card, |c| c.card)
}

fn target_specs(cards: &CardTable, state: &GameState, item: &StackItem) -> &'static [TargetSpec] {
    match item.kind {
        StackKind::Spell => cards.get(face(state, item.object)).spell_targets(),
        _ => match cards.ability(item.ability) {
            AbilityDef::Activated(a) => a.targets,
            AbilityDef::Triggered(t) => t.targets,
        },
    }
}

/// The modes an item on the stack was put there with (CR 700.2), if it is modal.
pub fn modal_of(cards: &CardTable, state: &GameState, item: &StackItem) -> Option<&'static ModalDef> {
    match item.kind {
        StackKind::Spell => cards.get(face(state, item.object)).modal,
        StackKind::Activated => match cards.ability(item.ability) {
            AbilityDef::Activated(a) => a.modal,
            AbilityDef::Triggered(_) => None,
        },
        StackKind::Triggered => match cards.ability(item.ability) {
            AbilityDef::Triggered(t) => t.modal,
            AbilityDef::Activated(_) => None,
        },
    }
}

/// How many targets each of a modal item's target slots holds (CR 700.2c, 601.2c): one for a slot of at most one target, and
/// for a slot of a range ("one or two target creatures") what is left of the item's targets once the single ones are
/// counted, the first such slot taking all of it.
fn slot_widths(specs: &[TargetSpec], target_count: usize) -> Vec<usize> {
    let single = specs.iter().filter(|s| s.max <= 1).count();
    let mut left = target_count.saturating_sub(single);
    specs
        .iter()
        .map(|s| {
            if s.max <= 1 {
                1
            } else {
                std::mem::take(&mut left)
            }
        })
        .collect()
}

/// The slots a list of target specs stands for, for a spell or ability controlled by `controller` (CR 601.2c): one each,
/// except that a `TargetSpec::EACH_OPPONENT` is a slot of at most one target for each seat but the controller's, in seat
/// order, each with the opponent it is for. A player who has left keeps their seat, so the others keep their places.
pub(crate) fn expand_specs(state: &GameState, controller: PlayerId, specs: &[TargetSpec]) -> Vec<(TargetSpec, Option<PlayerId>)> {
    let mut out = Vec::with_capacity(specs.len());
    for spec in specs {
        if spec.max == TargetSpec::EACH_OPPONENT {
            let each = TargetSpec { min: 0, max: 1, ..*spec };
            out.extend(state.seats().filter(|&p| p != controller).map(|p| (each, Some(p))));
        } else {
            out.push((*spec, None));
        }
    }
    out
}

/// The spec of an item's target `slot`, and the opponent the slot is for if it is one of an each-opponent spec: for a
/// modal item the slots of its chosen modes in printed order (CR 700.2c, 601.2c), a slot of a range holding several targets.
pub(crate) fn slot_spec(cards: &CardTable, state: &GameState, item: &StackItem, slot: usize) -> Option<(TargetSpec, Option<PlayerId>)> {
    if let Some(modal) = modal_of(cards, state, item) {
        let mut specs: Vec<&'static TargetSpec> = Vec::new();
        for (i, mode) in modal.modes.iter().enumerate() {
            for _ in 0..modal.times(item.modes, i) {
                specs.extend(mode.targets.iter());
            }
        }
        let owned: Vec<TargetSpec> = specs.iter().map(|s| **s).collect();
        let mut n = 0;
        for (spec, width) in specs.into_iter().zip(slot_widths(&owned, item.target_count as usize)) {
            if slot < n + width {
                return Some((*spec, None));
            }
            n += width;
        }
        return None;
    }
    let specs = target_specs(cards, state, item);
    // CR 702.47c: the slots of spliced text follow the spell's own, a target each, in the order the cards were spliced.
    if item.kind == StackKind::Spell && slot >= specs.len() && item.data.iter().any(|&d| d != 0) {
        let mut at = specs.len();
        for &spliced in item.data.iter().filter(|&&d| d != 0) {
            let added = cards.get(CardId(spliced as u16 - 1)).spell.map_or(&[][..], |s| s.targets);
            if slot < at + added.len() {
                return Some((added[slot - at], None));
            }
            at += added.len();
        }
    }
    if specs.iter().any(|s| s.max == TargetSpec::EACH_OPPONENT) {
        return expand_specs(state, item.controller, specs).get(slot).copied();
    }
    specs.get(slot.min(specs.len().saturating_sub(1))).map(|s| (*s, None))
}

/// Runs the chosen modes of a modal item in printed order (CR 700.2d), each as if it were a spell of its own with
/// its own targets in slot 0 onward; a mode whose every target has become illegal does nothing (CR 608.2b).
pub fn run_modes(cx: &mut Cx, item: &StackItem, modal: &ModalDef) -> Result<(), Suspend> {
    let mut next = 0usize;
    let widths = slot_widths(&crate::actions::mode_specs(modal, item.modes), item.target_count as usize);
    let mut spec_at = 0usize;
    for (i, mode) in modal.modes.iter().enumerate() {
        // A mode chosen more than once is that many instances in sequence, each with targets of its own (CR 700.2d).
        for _ in 0..modal.times(item.modes, i) {
            let n: usize = widths[spec_at..spec_at + mode.targets.len()].iter().sum();
            spec_at += mode.targets.len();
            let mut sub = *item;
            sub.modes = modal.once(i);
            sub.targets = [Target::None; cantrip_state::MAX_TARGETS];
            sub.targets[..n].copy_from_slice(&item.targets[next..next + n]);
            sub.target_count = n as u8;
            next += n;
            if sub.targets().iter().any(|&t| t != Target::None) && (0..n).all(|slot| !target_legal(cx.cards(), cx.s(), &cx.g.log, &sub, slot)) {
                continue;
            }
            (mode.effect)(cx, &sub)?;
        }
    }
    Ok(())
}

/// Whether target `slot` of an item is still legal (CR 608.2b): the target is still what it was, the slot's filter
/// still takes it, and nothing forbids the source to target it.
pub fn target_legal(cards: &CardTable, state: &GameState, events: &[Record], item: &StackItem, slot: usize) -> bool {
    let target = item.targets[slot];
    // A slot that chose no target (an "up to" slot) has nothing to be legal (CR 601.2c).
    if target == Target::None {
        return false;
    }
    let Some((spec, slot_player)) = slot_spec(cards, state, item, slot) else {
        return false;
    };
    let asker = Asker { source: item.object, controller: item.controller };
    if let Target::Object(id) = target
        && !live(state, id)
    {
        return false;
    }
    let q = Reader { cards, state, events };
    // A slot of its own sees what the slots before it chose (CR 601.2c).
    let earlier = if item.target_count > 1 { &item.targets[..slot] } else { &[][..] };
    (spec.filter)(&Given { q: &q, earlier, subject: item.subject, slot_player }, &asker, target) && layers::can_target(cards, state, &asker, target)
}

/// A `Query` that also says what the earlier target slots chose, for the filter of a later one.
pub struct Given<'a> {
    pub q: &'a dyn Query,
    pub earlier: &'a [Target],
    /// The subject of the event the ability triggered on.
    pub subject: Target,
    /// The opponent the slot is for, when it is one of a `TargetSpec::EACH_OPPONENT`.
    pub slot_player: Option<PlayerId>,
}

impl Query for Given<'_> {
    fn state(&self) -> &GameState {
        self.q.state()
    }

    fn events(&self) -> &[Record] {
        self.q.events()
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.q.card_id(def)
    }

    fn card(&self, id: CardId) -> &'static CardDef {
        self.q.card(id)
    }

    fn chars(&self, id: ObjectId) -> Chars {
        self.q.chars(id)
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        self.q.can_target(asker, target)
    }

    fn chosen_targets(&self) -> &[Target] {
        self.earlier
    }

    fn event_subject(&self) -> Target {
        self.subject
    }

    fn slot_opponent(&self) -> Option<PlayerId> {
        self.slot_player
    }
}

/// A read-only `Query` over a state, for checks made outside a frame's run.
pub struct Reader<'a> {
    pub cards: &'a CardTable,
    pub state: &'a GameState,
    pub events: &'a [Record],
}

impl Query for Reader<'_> {
    fn state(&self) -> &GameState {
        self.state
    }

    fn events(&self) -> &[Record] {
        self.events
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.cards.id_of(def)
    }

    fn card(&self, id: CardId) -> &'static CardDef {
        self.cards.get(id)
    }

    fn chars(&self, id: ObjectId) -> Chars {
        layers::chars(self.cards, self.state, id)
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        layers::can_target(self.cards, self.state, asker, target)
    }
}

/// Where a spell goes as it leaves the stack other than to the battlefield: its owner's graveyard, or exile when it
/// was cast with flashback (CR 702.34a).
fn leave_stack(cx: &mut Cx, item: &StackItem, reason: ZoneReason) {
    if item.kind != StackKind::Spell {
        return;
    }
    let obj = *cx.s().obj(item.object);
    // A copy of a card leaves the stack for nowhere: it ceases to exist (CR 707.10a, 722.3c).
    if obj.card_copy {
        cx.st().zone_remove(Zone::STACK, item.object);
        cx.st().update_obj(item.object, |o| o.zone = Zone::GONE);
        return;
    }
    // A spell cast as its Adventure that resolves is exiled, and its owner may cast the creature from there later
    // (CR 715.4, 715.5); countered, or its targets gone, it is put into the graveyard like any spell.
    if item.way == WAY_ADVENTURE && reason == ZoneReason::Resolve {
        let new = ops::move_to_for(cx, item.object, Zone::exile(obj.owner), reason);
        cx.st().update_obj(new, |o| o.on_adventure = true);
        return;
    }
    let def = cx.card(obj.card);
    // Rebound (CR 702.88a): a spell cast from its caster's hand is exiled as it resolves instead of going to the
    // graveyard (one that is countered, or has no legal target, is not), and its controller may cast it from exile at
    // the beginning of their next upkeep.
    if def.rebound && reason == ZoneReason::Resolve && obj.cast_from == Some(ZoneKind::Hand) {
        let new = ops::move_to_for(cx, item.object, Zone::exile(obj.owner), reason);
        cx.st().delayed_push(cantrip_state::Delayed {
            when: DelayedWhen::Step { step: Step::Upkeep, on_turn_of: Some(item.controller) },
            source: new,
            controller: item.controller,
            ability: AbilityRef::rebound(obj.card),
            subject: Target::None,
            members: cantrip_state::Span::default(),
        });
        return;
    }
    // Paradigm (CR 702.170a): "then exile this spell" as it resolves; a spell that is countered or fizzles is not exiled.
    if def.paradigm && reason == ZoneReason::Resolve {
        ops::move_to_for(cx, item.object, Zone::exile(obj.owner), reason);
        return;
    }
    let exiled = (item.way > 0 && item.way < WAY_SPLIT_RIGHT && def.ways[item.way as usize - 1].exile_after)
        || (item.exile_after && {
        // The spell it was cast as (CR 709.3b, 715.3b): a half or an Adventure is judged by its own types.
        let spell = cx.card(face(cx.s(), item.object));
        spell.is(CardType::Instant) || spell.is(CardType::Sorcery)
    });
    let to = if exiled { Zone::exile(obj.owner) } else { Zone::graveyard(obj.owner) };
    ops::move_to_for(cx, item.object, to, reason);
}

pub fn resolve(cx: &mut Cx) -> R<Flow> {
    let at = cx.s().stack.len() - 1;
    let item = cx.s().stack[at];

    // CR 608.2b: a spell or ability whose every target has become illegal does nothing.
    let count = item.target_count as usize;
    let chose_any = item.targets().iter().any(|&t| t != Target::None);
    let mut fizzles = chose_any && (0..count).all(|slot| !target_legal(cx.cards(), cx.s(), &cx.g.log, &item, slot));
    // CR 702.103e: a bestowed Aura spell whose target is illegal does not fizzle: it ceases to be bestowed and resolves as
    // the creature spell it is.
    if fizzles && item.kind == StackKind::Spell && item.way == WAY_BESTOW {
        cx.st().update_obj(item.object, |o| o.copy = None);
        fizzles = false;
    }

    match item.kind {
        StackKind::Spell => {
            let def = cx.card(face(cx.s(), item.object));
            if fizzles {
                cx.st().stack_remove(at);
                leave_stack(cx, &item, ZoneReason::Counter);
                return Ok(Flow::Done);
            }
            // Gift (CR 702.174b, 702.174j): the chosen opponent gets it before any of the spell's other effects, and
            // only as the spell resolves.
            if let (Some(Gift::Card), Some(to)) = (def.gift, item.gift)
                && !cx.s().player(to).lost
            {
                ops::draw(cx, to);
            }
            if let Some(spell) = def.spell {
                match def.modal {
                    Some(modal) => run_modes(cx, &item, modal)?,
                    None => (spell.effect)(cx, &item)?,
                };
                // The spell abilities spliced onto it follow its own (CR 702.47c).
                for &spliced in item.data.iter().filter(|&&d| d != 0) {
                    if let Some(added) = cx.card(CardId(spliced as u16 - 1)).spell {
                        (added.effect)(cx, &item)?;
                    }
                }
                // CR 608.2m: a spell that put itself somewhere as an instruction (Time Spiral exiles itself, Green Sun's
                // Zenith shuffles itself into its library) has left the stack already, and goes nowhere more.
                if let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == item.object) {
                    cx.st().stack_remove(at);
                    leave_stack(cx, &item, ZoneReason::Resolve);
                }
            } else if cx.s().obj(item.object).card_copy {
                // A copy of a permanent spell becomes a token as it resolves (CR 707.10f).
                ops::resolve_copy_as_token(cx, &item, at)?;
            } else {
                // A permanent spell becomes a permanent (CR 608.3), its entering replacements applied first, while it
                // is still on the stack (CR 614.12).
                let mut entering = Entering {
                    object: item.object,
                    controller: item.controller,
                    tapped: false,
                    counters: [0; CounterKind::COUNT],
                    // A bestowed spell is the Aura it was cast as, until it ceases to be bestowed (CR 702.103b).
                    copy: if item.way == WAY_BESTOW { cx.s().obj(item.object).copy } else { None },
                    prepared: false,
                    chosen_color: None,
                    chosen_subtype: None,
                    chosen_player: None,
                    chosen_card: None,
                    chosen_odd: None,
                    // An Aura spell enters attached to what it targeted (CR 303.4f, 608.3b).
                    attach: if def.enchant.is_some() { item.targets[0].object() } else { None },
                    attach_player: if def.enchant.is_some() { item.targets[0].player() } else { None },
                    x: item.x,
                    // Cast as its back face, it enters back face up (CR 712.8b).
                    transformed: item.way == WAY_BACK_FACE || item.way == WAY_TRANSFORMED,
                    protector: None,
                    instead: None,
                };
                ops::prepare_entry(cx, &mut entering)?;
                cx.st().stack_remove(at);
                ops::move_together(
                    cx,
                    &[ops::Moving { id: item.object, to: Zone::BATTLEFIELD, entering: Some(entering), reason: None }],
                    ZoneReason::Resolve,
                );
                if let Some(new) = cx.s().obj(item.object).became
                    && cx.s().obj(new).zone.is(ZoneKind::Battlefield)
                {
                    ops::carry_effects(cx, item.object, new);
                }
                // A creature spell paid for with mana that gives it haste has it until end of turn (CR 106.6).
                if cx.s().obj(item.object).marks.gains_haste
                    && let Some(new) = cx.s().obj(item.object).became
                    && cx.s().obj(new).zone.is(ZoneKind::Battlefield)
                {
                    Game::add_effect(cx, new, new, EffectKind::AddKeywords(Keywords::of(&[Keyword::Haste])), Duration::EndOfTurn);
                }
                // A granted way's "if you do, it gains ...": the permanent has the ability from then on.
                if let Some(ability) = cx.s().obj(item.object).marks.gains
                    && let Some(new) = cx.s().obj(item.object).became
                    && cx.s().obj(new).zone.is(ZoneKind::Battlefield)
                {
                    Game::add_effect(cx, new, new, EffectKind::AddAbility(ability), Duration::Indefinite);
                }
            }
        }
        StackKind::Activated | StackKind::Triggered => {
            // An intervening "if" no longer true: the ability does nothing (CR 603.4).
            let holds = item.kind != StackKind::Triggered || crate::settle::still_holds(cx, &item);
            if !fizzles && holds {
                match cx.cards().ability(item.ability) {
                    AbilityDef::Activated(ActivatedDef { modal: Some(modal), .. }) => run_modes(cx, &item, modal)?,
                    AbilityDef::Activated(a) => (a.effect)(cx, &item)?,
                    AbilityDef::Triggered(TriggeredDef { modal: Some(modal), .. }) => run_modes(cx, &item, modal)?,
                    AbilityDef::Triggered(t) => (t.effect)(cx, &item)?,
                }
                // It has resolved (CR 608.2m: "the first time this ability has resolved this turn").
                cx.record(Event::Resolved { source: item.object, ability: item.ability });
            }
            cx.st().stack_remove(at);
        }
    }
    Ok(Flow::Done)
}

/// Counters a spell on the stack (CR 701.6): it leaves the stack for its owner's graveyard, or exile.
/// Counters a spell and puts it somewhere other than its owner's graveyard (Remand's hand, Memory Lapse's library
/// top: CR 701.6a, the move is part of the countering). A spell that can't be countered stays.
pub fn counter_to(cx: &mut Cx, spell: ObjectId, to: CounterTo) {
    let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) else {
        return;
    };
    if cx.chars(spell).keywords.has(Keyword::CantBeCountered) {
        return;
    }
    cx.emit(Event::Countered { spell });
    send_spell(cx, at, to, ZoneReason::Counter);
}

/// A spell leaves the stack without being countered (CR 400.3): "return target spell to its owner's hand" (Sink
/// into Stupor), "its owner puts it on their choice of the top or bottom of their library" (Subtlety). Nothing says
/// "counter", so it works on a spell that can't be countered and no one sees it countered.
pub fn remove_spell(cx: &mut Cx, spell: ObjectId, to: CounterTo) {
    if let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) {
        send_spell(cx, at, to, ZoneReason::Other);
    }
}

/// Takes the spell at `at` off the stack and into the zone `to` names, its owner's.
fn send_spell(cx: &mut Cx, at: usize, to: CounterTo, reason: ZoneReason) {
    let item = cx.st().stack_remove(at);
    let obj = *cx.s().obj(item.object);
    if obj.card_copy {
        cx.st().zone_remove(Zone::STACK, item.object);
        cx.st().update_obj(item.object, |o| o.zone = Zone::GONE);
        return;
    }
    // Flashback (CR 702.34a): exiled instead of going anywhere else as it leaves the stack, so a flashed-back spell sent to
    // a hand or a library by the effect that removes it is exiled instead (CR 614.1: the replacement applies to the
    // move whichever zone it was headed for).
    let flashback = item.way > 0 && item.way < WAY_SPLIT_RIGHT && cx.card(obj.card).ways[item.way as usize - 1].exile_after;
    let zone = match to {
        _ if flashback => Zone::exile(obj.owner),
        CounterTo::Hand => Zone::hand(obj.owner),
        CounterTo::LibraryTop | CounterTo::LibraryBottom => Zone::library(obj.owner),
        CounterTo::Exile => Zone::exile(obj.owner),
    };
    let new = ops::move_to_for(cx, item.object, zone, reason);
    if to == CounterTo::LibraryBottom && !flashback && cx.s().obj(new).zone.is(ZoneKind::Library) {
        cx.st().zone_remove(zone, new);
        cx.st().zone_push(zone, new);
    }
}

pub fn counter(cx: &mut Cx, spell: ObjectId) {
    let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) else {
        return;
    };
    // A spell that can't be countered stays on the stack (CR 101.2, 701.6a).
    if cx.chars(spell).keywords.has(Keyword::CantBeCountered) || cx.s().obj(spell).marks.cant_be_countered {
        return;
    }
    let item = cx.st().stack_remove(at);
    cx.emit(Event::Countered { spell });
    leave_stack(cx, &item, ZoneReason::Counter);
}

/// Counters a spell or an ability on the stack (CR 701.6): `hint` is where the item was when it was announced, tried
/// first so that two items of one source are told apart; a countered ability just ceases to exist.
pub fn counter_item(cx: &mut Cx, object: ObjectId, hint: usize) {
    let stack = &cx.s().stack;
    let at = if stack.get(hint).is_some_and(|i| i.object == object) {
        hint
    } else if let Some(at) = stack.iter().rposition(|i| i.object == object) {
        at
    } else {
        return;
    };
    if cx.s().stack[at].kind == StackKind::Spell {
        counter(cx, object);
    } else {
        cx.st().stack_remove(at);
    }
}
