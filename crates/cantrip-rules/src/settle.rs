//! What happens each time a player would receive priority (CR 117.5): state-based actions until none apply
//! (CR 704), then triggered abilities onto the stack (CR 603.3), and again until neither does anything.

use cantrip_api::*;
use cantrip_state::{PendingTrigger, Span};

use crate::actions::target_candidates_given;
use crate::layers::{self, live};
use crate::{Cx, Flow, R, ops};

/// Whether a permanent is a bestowed Aura (CR 702.103b): the Aura face its card has while it is bestowed is what its copy
/// effect says it is, until it ceases to be bestowed (CR 702.103f).
fn bestowed(cx: &Cx, id: ObjectId) -> bool {
    let obj = cx.s().obj(id);
    obj.copy.is_some_and(|c| cx.card(obj.card).bestow.is_some_and(|b| std::ptr::eq(cx.card(c.card), b)))
}

pub fn settle(cx: &mut Cx) -> R<Flow> {
    loop {
        if state_based_actions(cx)? {
            continue;
        }
        if cx.s().is_over() {
            break;
        }
        if ops::return_exiled(cx)? {
            continue;
        }
        if !cx.s().triggers.is_empty() {
            put_triggers_on_stack(cx)?;
            continue;
        }
        break;
    }
    cx.st().clear_unsettled();
    Ok(Flow::Done)
}

/// Checks every state-based action at once and performs those that apply, simultaneously (CR 704.3). Whether any did.
fn state_based_actions(cx: &mut Cx) -> R<bool> {
    let mut acted = false;
    let mut leavers = Vec::new();

    // CR 704.5a, 704.5b: a player at 0 life or who drew from an empty library loses.
    for p in cx.s().seats().collect::<Vec<_>>() {
        let player = *cx.s().player(p);
        // CR 704.5c: ten or more poison counters lose the game too.
        if !player.lost && (player.life <= 0 || player.drew_from_empty || player.poison >= 10) {
            cx.st().update_player(p, |pl| {
                pl.lost = true;
                pl.drew_from_empty = false;
            });
            crate::designations::monarch_left(cx);
            leavers.push(p);
            acted = true;
        }
    }
    // They all lost at once; the game goes on without them, or ends with the one left (CR 800.4a, 104.2a).
    for p in leavers {
        ops::leave_game(cx, p);
    }

    // A prepare spell's copy ceases to exist out of exile, or once what prepared it is gone or no longer prepared
    // (CR 722.3c); on the stack it is a spell like any other.
    let stale: Vec<(Zone, ObjectId)> = cx
        .s()
        .zones
        .iter()
        .flatten()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            obj.card_copy && !obj.zone.is(ZoneKind::Stack) && {
                let kept = obj.zone.is(ZoneKind::Exile)
                    && obj.prepared_from.is_some_and(|from| {
                        let f = cx.s().obj(from);
                        live(cx.s(), from) && f.zone.is(ZoneKind::Battlefield) && f.prepared
                    });
                !kept
            }
        })
        .map(|id| (cx.s().obj(id).zone, id))
        .collect();
    for (zone, id) in stale {
        cx.st().zone_remove(zone, id);
        cx.st().update_obj(id, |o| o.zone = Zone::GONE);
        acted = true;
    }

    // CR 704.5d: a token anywhere but the battlefield ceases to exist.
    let gone: Vec<(Zone, ObjectId)> = cx
        .s()
        .zones
        .iter()
        .flatten()
        .copied()
        .filter(|&id| cx.s().obj(id).token && !cx.s().obj(id).zone.is(ZoneKind::Battlefield))
        .map(|id| (cx.s().obj(id).zone, id))
        .collect();
    for (zone, id) in gone {
        cx.st().zone_remove(zone, id);
        cx.st().update_obj(id, |o| o.zone = Zone::GONE);
        acted = true;
    }

    // CR 702.131b: a player who controls a permanent with ascend and ten or more permanents gets the city's blessing, for the
    // rest of the game.
    let big = cx.s().battlefield().len() >= 10;
    let players: Vec<PlayerId> = if big { cx.s().seats().collect() } else { Vec::new() };
    for p in players {
        if cx.s().player(p).citys_blessing || cx.s().player(p).lost {
            continue;
        }
        let mut permanents = 0;
        let mut ascends = false;
        for &id in cx.s().battlefield() {
            let obj = cx.s().obj(id);
            if obj.controller == p {
                permanents += 1;
                ascends |= cx.card(obj.copy.map_or(obj.card, |c| c.card)).ascend && !cx.s().lost_all_abilities(id);
            }
        }
        if ascends && permanents >= 10 {
            cx.st().update_player(p, |pl| pl.citys_blessing = true);
            acted = true;
        }
    }

    // CR 704.5f, g, h, i: creatures with no toughness or lethal damage, planeswalkers with no loyalty.
    let mut to_graveyard = Vec::new();
    let mut destroyed = Vec::new();
    let mut unattach = Vec::new();
    let mut unbestow = Vec::new();
    let mut sagas = Vec::new();
    let mut legends: Vec<(ObjectId, &'static str)> = Vec::new();
    let mut bound: Vec<crate::day_night::Bound> = Vec::new();
    let mut battles: Vec<(ObjectId, bool)> = Vec::new();
    for &id in cx.s().battlefield() {
        let chars = cx.chars(id);
        if let Some(daybound) = crate::day_night::role(chars.keywords) {
            bound.push(crate::day_night::Bound { id, daybound });
        }
        let obj = cx.s().obj(id);
        if chars.supertypes.legendary {
            legends.push((id, chars.name));
        }
        if chars.has_subtype(Subtype::Saga) {
            sagas.push((id, chars));
        }
        if chars.is(CardType::Battle) {
            // CR 704.5v-w: a battle with no defense counters is put into its owner's graveyard, not destroyed. A
            // Siege is spared while it is the source of an ability that has triggered and not yet left the stack: its
            // own defeat ability (CR 310.12b).
            battles.push((id, chars.has_subtype(Subtype::Siege)));
            let pending = cx.s().triggers.iter().any(|t| t.source == id)
                || cx.s().stack.iter().any(|i| i.kind != StackKind::Spell && i.object == id);
            if cx.s().obj(id).counters(CounterKind::Defense) <= 0 && !(chars.has_subtype(Subtype::Siege) && pending) {
                to_graveyard.push(id);
                continue;
            }
        }
        // CR 704.5m: an Aura attached to an illegal object, or to nothing, goes to its owner's graveyard. CR 704.5n:
        // an Equipment attached to an illegal permanent becomes unattached; so does anything else attached to
        // anything (CR 704.5p).
        if chars.has_subtype(Subtype::Aura) {
            if !(obj.attached_to.is_some_and(|host| crate::attach::host_allows(&*cx, id, host, false))
                || obj.attached_player.is_some_and(|p| crate::attach::player_allows(&*cx, id, p)))
            {
                // CR 702.103f: a bestowed Aura attached to an illegal object, or to nothing, becomes unattached and
                // ceases to be bestowed, a creature again, instead of going to the graveyard.
                if bestowed(&*cx, id) {
                    unbestow.push(id);
                } else {
                    to_graveyard.push(id);
                }
                continue;
            }
        } else if let Some(host) = obj.attached_to
            && !(chars.has_subtype(Subtype::Equipment) && crate::attach::host_allows(&*cx, id, host, false))
        {
            unattach.push(id);
        }
        if chars.is(CardType::Creature) {
            let toughness = chars.toughness.unwrap_or(0);
            if toughness <= 0 {
                to_graveyard.push(id);
                continue;
            }
            if obj.damage as i16 >= toughness || obj.deathtouched {
                destroyed.push(id);
                continue;
            }
        }
        if chars.is(CardType::Planeswalker) && obj.counters(CounterKind::Loyalty) <= 0 {
            to_graveyard.push(id);
        }
    }
    // CR 702.145d, 702.145g, 702.145c, 702.145f: day and night, as far as the permanents with daybound or nightbound go.
    if crate::day_night::apply(cx, &bound) {
        acted = true;
    }
    destroyed.retain(|&id| !cx.chars(id).has(Keyword::Indestructible));
    // CR 701.19a, 614.8: a creature a regeneration shield saves from lethal damage regenerates instead.
    if !destroyed.is_empty() && cx.s().effects.iter().any(|e| e.kind == EffectKind::RegenerationShield) {
        let saved = ops::regenerate_instead(cx, &destroyed);
        if !saved.is_empty() {
            destroyed.retain(|id| !saved.contains(id));
            acted = true;
        }
    }
    for id in unattach {
        ops::unattach(cx, id);
        acted = true;
    }
    for id in unbestow {
        ops::unattach(cx, id);
        cx.st().update_obj(id, |o| o.copy = None);
        acted = true;
    }

    // CR 714.4, 704.5s: a Saga whose lore counters have reached its final chapter is sacrificed, unless it is the
    // source of a chapter ability that has triggered and not yet left the stack. Judged before the triggers of the
    // counter that reached it go on the stack, so the last chapter always gets to resolve.
    let mut sacrificed = Vec::new();
    for (id, chars) in sagas {
        let last = chars
            .abilities()
            .filter_map(|a| match cx.cards().ability(a) {
                AbilityDef::Triggered(t) if t.chapter > 0 => Some((a, t.chapter)),
                _ => None,
            })
            .fold(0, |most, (_, chapter)| most.max(chapter));
        if last == 0 || cx.s().obj(id).counters(CounterKind::Lore) < last as i16 {
            continue;
        }
        let chapter_ability = |ability: AbilityRef| matches!(cx.cards().ability(ability), AbilityDef::Triggered(t) if t.chapter > 0);
        let pending = cx.s().triggers.iter().any(|t| t.source == id && chapter_ability(t.ability))
            || cx.s().stack.iter().any(|i| i.kind == StackKind::Triggered && i.object == id && chapter_ability(i.ability));
        if !pending {
            sacrificed.push(id);
        }
    }

    // CR 310.11, 704.5x-y: a battle whose protector can no longer protect it, or that has none in the game, has its
    // controller choose another; nobody to choose puts it into its owner's graveyard. A battle being attacked keeps a
    // protector who has left the game until combat ends (704.5x).
    for (id, siege) in battles {
        if to_graveyard.contains(&id) {
            continue;
        }
        let controller = cx.s().obj(id).controller;
        let candidates = ops::protector_candidates(cx, siege, controller);
        let current = cx.s().obj(id).marks.protector;
        if current.is_some_and(|p| candidates.contains(&p)) {
            continue;
        }
        let unprotected = current.is_none_or(|p| cx.s().player(p).lost);
        if unprotected && cx.s().attackers.iter().any(|a| a.target == Target::Object(id)) {
            continue;
        }
        if candidates.is_empty() {
            to_graveyard.push(id);
            continue;
        }
        let protector = if candidates.len() > 1 {
            let ask = RequestKind::ChoosePlayer { source: id, candidates: candidates.clone(), verb: "Protect", amount: 0 };
            match cx.ask_core(controller, ask, false)? {
                Answer::Player(p) if candidates.contains(&p) => p,
                _ => candidates[0],
            }
        } else {
            candidates[0]
        };
        cx.st().update_obj(id, |o| o.marks.protector = Some(protector));
        acted = true;
    }

    // CR 704.5j, the legend rule: a player who controls two or more legendary permanents with the same name chooses
    // one and puts the rest into their owners' graveyards, not destroyed. Asked in turn order, a name at a time in the
    // order its first permanent entered; decided on the same board as the other actions of this check (CR 704.3),
    // so a legend that is dying anyway may still be the one kept.
    let mut surplus = Vec::new();
    if legends.len() > 1 {
        for p in cx.s().apnap().collect::<Vec<_>>() {
            let mine: Vec<(ObjectId, &'static str)> =
                legends.iter().copied().filter(|&(id, _)| cx.s().obj(id).controller == p).collect();
            let mut asked: Vec<&'static str> = Vec::new();
            for &(_, name) in &mine {
                if asked.contains(&name) {
                    continue;
                }
                asked.push(name);
                let group: Vec<ObjectId> = mine.iter().filter(|&&(_, n)| n == name).map(|&(id, _)| id).collect();
                if group.len() < 2 {
                    continue;
                }
                let ask = RequestKind::ChooseCards {
                    source: group[0],
                    purpose: Purpose::KeepLegend,
                    cards: group.clone(),
                    eligible: group.clone(),
                    count: 1,
                    up_to: false, kind: "" };
                let keep = match cx.ask_core(p, ask, false)? {
                    Answer::Cards(cards) => cards.first().copied().unwrap_or(group[0]),
                    _ => group[0],
                };
                surplus.extend(group.into_iter().filter(|&id| id != keep));
            }
        }
    }
    // A permanent that satisfies several of these changes zones once, and what is not destruction wins over it.
    for id in sacrificed.into_iter().chain(surplus) {
        if !to_graveyard.contains(&id) {
            to_graveyard.push(id);
        }
    }
    destroyed.retain(|id| !to_graveyard.contains(id));
    if !to_graveyard.is_empty() || !destroyed.is_empty() {
        // All at once (CR 704.3): a creature that dies to its toughness and one destroyed by damage leave together.
        let mut moves: Vec<ops::Moving> = Vec::new();
        for id in to_graveyard.into_iter().chain(destroyed) {
            moves.push(ops::Moving::to(id, Zone::graveyard(cx.s().obj(id).owner)));
        }
        ops::move_together(cx, &moves, ZoneReason::StateBased);
        acted = true;
    }

    // CR 704.5t: a player on the bottommost room of a dungeon with none of its abilities on the stack completes it.
    acted |= crate::designations::complete_dungeons(cx);

    // CR 104.2a: the game ends when one player is left (or none).
    if acted {
        let left: Vec<PlayerId> = cx.s().apnap().collect();
        match left.len() {
            0 => cx.st().set_outcome(Outcome::Draw),
            1 => cx.st().set_outcome(Outcome::Won(left[0])),
            _ => {}
        }
    }

    // Effects on objects that are gone apply to nothing (CR 611.2c); dropping them keeps the list short.
    // "For as long as [source] remains on the battlefield" ends with it (CR 611.2b).
    let state = cx.s();
    let applies = |e: &cantrip_state::ContinuousEffect| {
        (live(state, e.affected) || e.kind.outlives_affected())
            && (e.duration != Duration::WhileSourceOnBattlefield
                || (live(state, e.source) && state.obj(e.source).zone.is(ZoneKind::Battlefield)))
    };
    if state.effects.iter().any(|e| !applies(e)) {
        let alive: Vec<bool> = state.effects.iter().map(applies).collect();
        // By index: `effects_retain` visits the effects from the end, so a counter over them would mismatch.
        cx.st().effects_keep(&alive);
    }
    Ok(acted)
}

fn triggered(cx: &Cx, ability: AbilityRef) -> &'static TriggeredDef {
    match cx.cards().ability(ability) {
        AbilityDef::Triggered(t) => t,
        AbilityDef::Activated(_) | AbilityDef::Static(_) => unreachable!("a triggered ability"),
    }
}

/// Puts waiting triggered abilities on the stack (CR 603.3b): the active player's first, then each other player's
/// in turn order; a player with more than one chooses their order, and each ability's targets are chosen as it goes
/// on (CR 603.3d).
fn put_triggers_on_stack(cx: &mut Cx) -> R<()> {
    let pending: Vec<PendingTrigger> = cx.s().triggers.clone();
    while !cx.s().triggers.is_empty() {
        let last = cx.s().triggers.len() - 1;
        cx.st().trigger_remove(last);
    }
    for p in cx.s().apnap().collect::<Vec<_>>() {
        let mut theirs: Vec<PendingTrigger> = pending.iter().copied().filter(|t| t.controller == p).collect();
        if theirs.len() > 1 {
            let ask = RequestKind::OrderTriggers {
                sources: theirs.iter().map(|t| t.source).collect(),
                abilities: theirs.iter().map(|t| t.ability).collect(),
            };
            if let Answer::Order(order) = cx.ask_core(p, ask, false)? {
                theirs = order.iter().map(|&i| theirs[i as usize]).collect();
            }
        }
        for trigger in theirs {
            put_trigger(cx, trigger)?;
        }
    }
    Ok(())
}

/// Chooses the targets of a triggered ability's slots as it is put on the stack (CR 603.3d). Whether it can be put
/// there at all: a required slot with no legal target removes it. `all_of_few` takes every candidate of a required
/// slot without asking when there are no more than it takes, as a mode's target is (CR 601.2c).
fn choose_trigger_targets(
    cx: &mut Cx,
    trigger: &PendingTrigger,
    specs: &[TargetSpec],
    all_of_few: bool,
    targets: &mut Vec<Target>,
) -> R<bool> {
    let asker = Asker { source: trigger.source, controller: trigger.controller };
    let start = targets.len();
    let slots = crate::resolve::expand_specs(cx.s(), trigger.controller, specs);
    for (spec, slot_player) in &slots {
        let before = targets.len();
        'slot: {
            let candidates = target_candidates_given(
                cx,
                &asker,
                spec,
                if slots.len() > 1 { &targets[start..] } else { &[] },
                trigger.subject,
                *slot_player,
            );
            // A bound that is the number of times the source was kicked, or its X (CR 702.33d, 107.3m).
            let source = cx.s().obj(trigger.source);
            let counted = if source.x > 0 { source.x } else { source.kicked as u16 };
            let (least, most) = spec.bounds(counted);
            if candidates.len() < least as usize {
                // Fewer legal targets than the slot's minimum: it is removed from the stack (CR 603.3d, 601.2c).
                return Ok(false);
            }
            if candidates.is_empty() {
                if spec.required() {
                    // With no legal target, it is removed from the stack (CR 603.3d).
                    return Ok(false);
                }
                break 'slot;
            }
            // A lone required candidate is no choice, for an ability with one target slot only (the C# engine asks each of several).
            if spec.min > 0 && spec.max == 1 && candidates.len() == 1 && (slots.len() == 1 || all_of_few) {
                targets.push(candidates[0]);
                break 'slot;
            }
            if all_of_few && spec.min > 0 && candidates.len() <= spec.max as usize {
                targets.extend(candidates);
                break 'slot;
            }
            if most == 0 {
                break 'slot;
            }
            let count = (most as usize).min(candidates.len()) as u8;
            let ask = RequestKind::ChooseTarget { label: spec.label, slot_player: *slot_player,
                source: trigger.source,
                purpose: Purpose::Targets,
                candidates,
                optional: least == 0,
                least: least.min(count),
                count,
                any_number: false,
            };
            // The ability's controller chooses its targets (CR 603.3d), unless it says that player does: the one it is about.
            let chooser = match trigger.subject {
                Target::Player(p) if triggered(cx, trigger.ability).subject_chooses_targets && !cx.s().player(p).lost => p,
                _ => trigger.controller,
            };
            if let Answer::Targets(chosen) = cx.ask_core(chooser, ask, false)? {
                targets.extend(chosen);
            }
        }
        // A slot that chose nothing keeps its place (CR 601.2c, 115.1).
        if slots.len() > 1 && spec.max <= 1 && targets.len() == before {
            targets.push(Target::None);
        }
    }
    while targets.len() > start && targets.last() == Some(&Target::None) {
        targets.pop();
    }
    Ok(true)
}

fn put_trigger(cx: &mut Cx, trigger: PendingTrigger) -> R<()> {
    let def = triggered(cx, trigger.ability);
    let asker = Asker { source: trigger.source, controller: trigger.controller };
    let mut targets = Vec::new();
    let mut shares: Vec<u8> = Vec::new();
    let mut mask = 0u8;
    if let Some(modal) = def.modal {
        // A modal ability's modes are chosen as it is put on the stack (CR 603.3c, 700.2), then each chosen mode's
        // target, in printed order. A mode whose required target has no candidate cannot be chosen (CR 700.2a), and with
        // too few left the ability is removed. As many eligible modes as it chooses is no choice, and is not asked.
        let eligible = crate::actions::eligible_modes(cx, &asker, modal);
        if eligible.len() < modal.min.max(1) as usize {
            return Ok(());
        }
        let chosen = if modal.max == modal.min && eligible.len() == modal.min as usize {
            eligible
        } else {
            let ask = RequestKind::ChooseModes {
                source: trigger.source,
                modes: (0..modal.modes.len() as u8).map(|i| Label::Mode(trigger.ability, i)).collect(),
                eligible: eligible.clone(),
                count: modal.min,
                at_most: if modal.max > modal.min { modal.max } else { 0 },
                affordable: Vec::new(),
                additional_cost: false,
                repeat: modal.repeat,
            };
            match cx.ask_core(trigger.controller, ask, false)? {
                Answer::Modes(modes) => modes,
                _ => eligible,
            }
        };
        for &n in &chosen {
            mask = modal.with(mask, n as usize);
        }
        // Chosen as it goes on the stack: chosen even if the ability is later countered (CR 603.3c, 700.2).
        cx.record(Event::ModesChosen { source: trigger.source, ability: trigger.ability, modes: mask });
        for (n, mode) in modal.modes.iter().enumerate() {
            for _ in 0..modal.times(mask, n) {
                if !choose_trigger_targets(cx, &trigger, mode.targets, true, &mut targets)? {
                    return Ok(());
                }
            }
        }
    } else if let Some(divide) = def.divide
        && let Some(spec) = def.targets.first()
    {
        // CR 601.2d, 603.3d: a divided slot's targets and shares are chosen as the ability goes on the stack.
        let x = cx.s().obj(trigger.source).x;
        let total = match divide.of_x {
            Some(of_x) => of_x(x),
            None if divide.total == TargetSpec::COUNTED => x,
            None => divide.total as u16,
        };
        match crate::actions::ask_divided(cx, trigger.controller, trigger.source, spec, total, false)? {
            crate::actions::Divided::Chosen(chosen, given) => {
                targets = chosen;
                shares = given;
            }
            crate::actions::Divided::Skipped => {}
            crate::actions::Divided::Impossible => return Ok(()),
        }
    } else if !choose_trigger_targets(cx, &trigger, def.targets, false, &mut targets)? {
        return Ok(());
    }
    let mut item = StackItem {
        kind: StackKind::Triggered,
        object: trigger.source,
        ability: trigger.ability,
        controller: trigger.controller,
        targets: [Target::None; cantrip_state::MAX_TARGETS],
        target_count: targets.len() as u8,
        x: 0,
        kicked: 0,
        paid: 0,
        modes: mask,
        way: 0,
        choice: 0,
        subject: trigger.subject,
        data: trigger.data,
        members: trigger.members,
        serial: 0,
        // CR 603.10, 702.174b: what was true of the source as the ability triggered, for a gift triggered ability to
        // give its gift after the permanent has left.
        gift: cx.s().obj(trigger.source).gift,
        shares: [0; cantrip_state::MAX_TARGETS],
        exile_after: false,
    };
    item.targets[..targets.len()].copy_from_slice(&targets);
    item.shares[..shares.len()].copy_from_slice(&shares);
    cx.st().stack_push(item);
    for t in targets.into_iter().filter(|&t| t != Target::None) {
        cx.emit(Event::BecameTarget { target: t, by: trigger.source, controller: trigger.controller });
    }
    Ok(())
}

type Found = Vec<(PendingTrigger, &'static TriggeredDef)>;

fn consider(cx: &Cx, source: ObjectId, look_back: bool, event: &Event, mask: EventMask, stoppers: &mut Stoppers, found: &mut Found) {
    let chars = layers::chars(cx.cards(), cx.s(), source);
    let controller = cx.s().obj(source).controller;
    for ability in chars.abilities() {
        if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
            && t.look_back == look_back
            && let Some(fired) = if t.chapter > 0 { chapter_reached(cx, source, t.chapter, event) } else if t.listens.intersects(mask) { (t.matches)(cx, source, event) } else { None }
        {
            if let Some(condition) = t.condition
                && !condition(cx, &Asker { source, controller })
            {
                continue;
            }
            // "Ward abilities of those creatures don't trigger" (CR 702.21a, Nowhere to Run): asked only of a ward that matched.
            if t.ward && layers::targeting_rule(cx.cards(), cx.s(), TargetingRuleKind::SuppressWard, source) {
                continue;
            }
            if stoppers.stop(cx, event, &Triggering { source, controller, def: t }) {
                continue;
            }
            found.push((
                PendingTrigger { source, ability, controller, subject: fired.subject, data: fired.data, members: Span::default() },
                t,
            ));
        }
    }
}

/// A Saga's chapter ability (CR 714.2b) triggers when lore counters are put on it and their number becomes at least
/// its chapter number, having been less than it.
fn chapter_reached(cx: &Cx, source: ObjectId, chapter: u8, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::Lore, amount, .. } if *object == source => {
            let now = cx.s().obj(source).counters(CounterKind::Lore);
            if now - *amount < chapter as i16 && now >= chapter as i16 { Fired::PLAIN } else { None }
        }
        _ => None,
    }
}

/// The permanents with static abilities that stop abilities from triggering (CR 603.2, `CardDef::trigger_rules`): found from
/// the board, once per batch and only when an ability has matched, so a batch that triggers nothing pays nothing. An
/// ability that triggers on the board as it is after the events asks those on the battlefield now; a leaves-the-battlefield
/// ability, which looks back (CR 603.10a), asks those that were there before, the ones leaving with the event included.
struct Stoppers<'a> {
    before: Option<&'a [ObjectId]>,
    now: Option<Vec<ObjectId>>,
    then: Option<Vec<ObjectId>>,
}

impl<'a> Stoppers<'a> {
    fn new(before: Option<&'a [ObjectId]>) -> Self {
        Stoppers { before, now: None, then: None }
    }

    /// Whether a static ability stops this ability triggering on this event.
    fn stop(&mut self, cx: &Cx, event: &Event, triggering: &Triggering) -> bool {
        let sources = if triggering.def.look_back && self.before.is_some() {
            let before = self.before.unwrap_or_default();
            self.then.get_or_insert_with(|| trigger_rule_sources(cx, before))
        } else {
            self.now.get_or_insert_with(|| trigger_rule_sources(cx, cx.s().battlefield()))
        };
        sources.iter().any(|&id| {
            let obj = cx.s().obj(id);
            let asker = Asker { source: id, controller: obj.controller };
            cx.card(cx.cards().current(obj)).trigger_rules.iter().any(|rule| rule(cx, &asker, event, triggering))
        })
    }
}

/// Those of `ids` whose card has a rule that stops abilities from triggering, and has not lost its abilities (CR 613.1f).
fn trigger_rule_sources(cx: &Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter()
        .copied()
        .filter(|&id| cx.cards().trigger_rules[cx.cards().current(cx.s().obj(id)).0 as usize] && !cx.s().lost_all_abilities(id))
        .collect()
}

/// Every triggered ability this event triggers (CR 603.2).
pub fn collect_triggers(cx: &mut Cx, event: &Event, before: Option<&[ObjectId]>) {
    collect_batch(cx, std::slice::from_ref(event), before);
}

/// Every triggered ability a batch of simultaneous events triggers (CR 603.2): leaves-the-battlefield abilities among
/// the permanents that were on the battlefield before it (`before`), the rest among those there now, and abilities
/// that function everywhere wherever they are. A "one or more" ability (CR 603.2c) triggers once for the batch, its
/// members the subjects of the events it matched; a limited one (CR 603.2h) stops triggering once it has used up its
/// turn, a triggering that happens being noted in the history.
pub fn collect_batch(cx: &mut Cx, events: &[Event], before: Option<&[ObjectId]>) {
    let mut found: Found = Vec::new();
    let mut stoppers = Stoppers::new(before);
    for event in events {
        let mask = EventMask::of(event);
        if mask.0 == 0 {
            continue;
        }
        let now: Vec<ObjectId> = cx.s().battlefield().to_vec();
        for &id in before.unwrap_or(&now) {
            consider(cx, id, true, event, mask, &mut stoppers, &mut found);
        }
        for &id in &now {
            consider(cx, id, false, event, mask, &mut stoppers, &mut found);
        }
        // An emblem's triggered abilities function in the command zone (CR 114.4).
        if !cx.s().zone(Zone::COMMAND).is_empty() {
            for id in cx.s().zone(Zone::COMMAND).to_vec() {
                consider(cx, id, false, event, mask, &mut stoppers, &mut found);
            }
        }
        // Delayed abilities of "whenever [event] this turn" (CR 603.7b), each of which triggers on every match.
        if cx.s().delayed.iter().any(|d| d.when.is_event()) {
            for d in cx.s().delayed.clone() {
                if d.when.is_event()
                    && let AbilityDef::Triggered(t) = cx.cards().ability(d.ability)
                    && t.listens.intersects(mask)
                    && let Some(fired) = (t.matches)(cx, d.source, event)
                    // The end of an "until" exile is no triggered ability but the second one-shot effect the rules create
                    // immediately after the event (CR 610.3), so nothing that stops abilities triggering stops it, and the
                    // event that ends the wait always uses it up below. A "this turn" delayed ability is a triggered ability
                    // (CR 603.7b) and is stopped like any other.
                    && (d.when == DelayedWhen::UntilEvent
                        || !stoppers.stop(cx, event, &Triggering { source: d.source, controller: d.controller, def: t }))
                {
                    found.push((
                        PendingTrigger {
                            source: d.source,
                            ability: d.ability,
                            controller: d.controller,
                            subject: if d.when == DelayedWhen::UntilEvent { d.subject } else { fired.subject },
                            data: fired.data,
                            members: Span::default(),
                        },
                        t,
                    ));
                    // "Until" is used up by the event that ends the wait (CR 603.7b).
                    if d.when == DelayedWhen::UntilEvent
                        && let Some(at) = cx.s().delayed.iter().position(|x| *x == d)
                    {
                        cx.st().delayed_remove(at);
                    }
                }
            }
        }
        // Abilities that function in other zones (they say where themselves): only cards that have one are looked at.
        // A spell on the stack is among them: "when you cast this spell" (storm, replicate; CR 702.40a, 702.56a)
        // triggers from the stack.
        // A spell may also be given such an ability by a static ability (Thrumming Stone's ripple, CR 613.1f), which only
        // a game that has had static abilities in play can have done.
        let cards = cx.cards();
        let granted = mask.intersects(EventMask::SPELL_CAST) && cx.s().statics_seen;
        let elsewhere: Vec<ObjectId> = cx
            .s()
            .seats()
            .flat_map(|p| [Zone::hand(p), Zone::graveyard(p), Zone::exile(p)])
            .chain(std::iter::once(Zone::STACK))
            .flat_map(|z| cx.s().zone(z).iter().copied())
            .filter(|&id| cards.anywhere[cx.s().obj(id).card.0 as usize] || (granted && cx.s().obj(id).zone.is(ZoneKind::Stack)))
            .collect();
        for id in elsewhere {
            let chars = layers::chars(cx.cards(), cx.s(), id);
            for ability in chars.abilities() {
                if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
                    && t.anywhere
                    && t.listens.intersects(mask)
                    && let Some(fired) = (t.matches)(cx, id, event)
                {
                    // A spell is controlled by whoever cast it; a card in another zone answers to its owner.
                    let controller = if cx.s().obj(id).zone.is(ZoneKind::Stack) {
                        cx.s().obj(id).controller
                    } else {
                        cx.s().obj(id).owner
                    };
                    if let Some(condition) = t.condition
                        && !condition(cx, &Asker { source: id, controller })
                    {
                        continue;
                    }
                    if stoppers.stop(cx, event, &Triggering { source: id, controller, def: t }) {
                        continue;
                    }
                    found.push((
                        PendingTrigger { source: id, ability, controller, subject: fired.subject, data: fired.data, members: Span::default() },
                        t,
                    ));
                }
            }
        }
    }
    if found.is_empty() {
        return;
    }

    // One trigger per "one or more" ability and source for the batch, keeping the first; the rest add their subjects.
    let mut kept: Vec<(PendingTrigger, &'static TriggeredDef, Vec<ObjectId>)> = Vec::with_capacity(found.len());
    for (trigger, def) in found {
        if def.once_per_batch
            && let Some(existing) =
                kept.iter_mut().find(|(k, d, _)| {
                    d.once_per_batch
                        && k.source == trigger.source
                        && k.ability == trigger.ability
                        && (!d.batch_by_data || k.data[1] == trigger.data[1])
                })
        {
            if let Target::Object(member) = trigger.subject
                && !existing.2.contains(&member)
            {
                existing.2.push(member);
            }
            continue;
        }
        if def.sums_per_subject
            && let Some(existing) = kept.iter_mut().find(|(k, d, _)| {
                d.sums_per_subject && k.source == trigger.source && k.ability == trigger.ability && k.subject == trigger.subject
            })
        {
            existing.0.data[0] += trigger.data[0];
            continue;
        }
        let mut members = Vec::new();
        if def.once_per_batch
            && let Target::Object(member) = trigger.subject
        {
            members.push(member);
        }
        kept.push((trigger, def, members));
    }

    for (mut trigger, def, members) in kept {
        if def.triggers_each_turn > 0 {
            if cx.history().triggerings(trigger.source, trigger.ability) >= def.triggers_each_turn as usize {
                continue;
            }
            cx.record(Event::Triggered { source: trigger.source, ability: trigger.ability });
        }
        if !members.is_empty() {
            trigger.members = cx.st().members_push(&members);
        }
        cx.st().trigger_push(trigger);
    }
}

/// A step has begun: its "at the beginning of" abilities trigger, and so do the delayed abilities waiting for it,
/// which are used up (CR 603.7c).
pub fn step_began(cx: &mut Cx, step: Step) {
    let active = cx.s().turn.active;
    // CR 500.4: a duration of "until your next end step" ends as the step begins, before its abilities trigger; a
    // permission granted during that step is for the next one.
    if step == Step::End && cx.s().permissions.iter().any(|p| p.grant.expiry == Expiry::UntilYourNextEndStep) {
        cx.st().permissions_retain(|p| !(p.grant.expiry == Expiry::UntilYourNextEndStep && p.player == active));
    }
    let mut at = 0;
    while at < cx.s().delayed.len() {
        let d = cx.s().delayed[at];
        let due = match d.when {
            DelayedWhen::Step { step: s, on_turn_of } => s == step && on_turn_of.is_none_or(|p| p == active),
            DelayedWhen::ThisTurn | DelayedWhen::UntilEvent | DelayedWhen::UntilNextTurnOf(_) => false,
        };
        if due {
            cx.st().delayed_remove(at);
            cx.st().trigger_push(PendingTrigger { source: d.source, ability: d.ability, controller: d.controller, subject: d.subject, data: [0; 2], members: d.members });
        } else {
            at += 1;
        }
    }
    cx.emit(Event::StepBegan { step });
    crate::designations::step_began(cx, step);
    if step == Step::Upkeep {
        suspended_cards(cx, active);
    }
}

/// "At the beginning of your upkeep, if this card is suspended, remove a time counter from it" (CR 702.62a): the rules'
/// own ability for each suspended card its owner has in exile. A card is looked at only if it has time counters.
fn suspended_cards(cx: &mut Cx, active: PlayerId) {
    let intrinsic = cx.cards().intrinsic;
    let suspended: Vec<ObjectId> = cx
        .s()
        .zone(Zone::exile(active))
        .iter()
        .copied()
        .filter(|&id| cx.s().obj(id).counters(CounterKind::Time) > 0 && cx.card(cx.s().obj(id).card).suspend.is_some())
        .collect();
    for source in suspended {
        cx.st().trigger_push(PendingTrigger {
            source,
            ability: AbilityRef { card: intrinsic, index: cantrip_api::intrinsic::SUSPEND_UPKEEP },
            controller: active,
            subject: Target::None,
            data: [0; 2],
            members: Span::default(),
        });
    }
}

/// Whether a triggered ability's intervening "if" still holds as it resolves (CR 603.4).
pub fn still_holds(cx: &Cx, item: &StackItem) -> bool {
    match cx.cards().ability(item.ability) {
        AbilityDef::Triggered(TriggeredDef { condition: Some(condition), .. }) => {
            condition(cx, &Asker { source: item.object, controller: item.controller })
        }
        _ => true,
    }
}
