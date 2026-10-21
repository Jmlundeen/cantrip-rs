//! What happens each time a player would receive priority (CR 117.5): state-based actions until none apply
//! (CR 704), then triggered abilities onto the stack (CR 603.3), and again until neither does anything.

use cantrip_api::*;
use cantrip_state::{PendingTrigger, Span};

use crate::actions::target_candidates_given;
use crate::layers::{self, live};
use crate::{Cx, Flow, R, counters, ops};

/// Whether a permanent is a bestowed Aura (CR 702.103b): the Aura face its card has while it is bestowed is what its copy
/// effect says it is, until it ceases to be bestowed (CR 702.103f).
fn bestowed(cx: &Cx, id: ObjectId) -> bool {
    let obj = cx.s().obj(id);
    obj.copy.is_some_and(|c| cx.card(obj.card).bestow.is_some_and(|b| std::ptr::eq(cx.card(c.card), b)))
}

pub fn settle(cx: &mut Cx) -> R<Flow> {
    loop {
        // CR 603.8: a state trigger triggers as soon as its state is true, so the board a resolution leaves is asked before
        // the state-based actions change it (CR 704.3): Thespian's Stage that became a Dark Depths with no ice counters has
        // triggered even if the legend rule then puts it into the graveyard.
        state_triggers(cx);
        if state_based_actions(cx)? {
            continue;
        }
        if cx.s().is_over() {
            break;
        }
        if ops::return_exiled(cx)? || crate::phasing::phase_in_returning(cx) {
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

/// State-based actions checked and performed until none applies (CR 704.3), with the returns of cards exiled until a
/// permanent left that follow from them (CR 610.3), and no triggered ability put on the stack: the check of the cleanup
/// step (CR 514.3a) and of a turn being ended (CR 724.1c). Whether anything was done.
pub fn state_based_actions_alone(cx: &mut Cx) -> R<bool> {
    let mut acted = false;
    loop {
        if state_based_actions(cx)? {
            acted = true;
            continue;
        }
        if cx.s().is_over() {
            break;
        }
        if ops::return_exiled(cx)? || crate::phasing::phase_in_returning(cx) {
            acted = true;
            continue;
        }
        break;
    }
    cx.st().clear_unsettled();
    Ok(acted)
}

/// Checks every state-based action at once and performs those that apply, simultaneously (CR 704.3). Whether any did.
fn state_based_actions(cx: &mut Cx) -> R<bool> {
    let mut acted = false;
    let mut leavers = Vec::new();
    // CR 611.2b: an effect "for as long as" a condition ends once it is false; then, CR 613.1b: who controls each permanent
    // is what the control-changing effects that apply now say, before anything that asks it (the legend rule, an Aura's
    // legality, "you control").
    ops::end_lapsed_conditions(cx);
    ops::settle_control(cx);

    // CR 704.5a, 704.5b, 704.5c: a player at 0 life, who attempted to draw from an empty library since the last check, or
    // with ten or more poison counters loses - unless a static ability says they don't lose that way (CR 104.3: Platinum
    // Angel, Phyrexian Unlife), in which case the game goes on and the draw from an empty library is forgotten with this
    // check.
    for p in cx.s().seats().collect::<Vec<_>>() {
        let player = *cx.s().player(p);
        if player.lost {
            continue;
        }
        if let Some(reason) = ops::loses_to_state_based_actions(cx, p) {
            cx.st().update_player(p, |pl| {
                pl.lost = true;
                pl.drew_from_empty = false;
            });
            crate::designations::monarch_left(cx);
            leavers.push((p, reason));
            acted = true;
        } else if player.drew_from_empty {
            cx.st().update_player(p, |pl| pl.drew_from_empty = false);
        }
    }
    // They all lost at once; the game goes on without them, or ends with the one left (CR 800.4a, 104.2a).
    for (p, reason) in leavers {
        cx.record(Event::GameLost { player: p, reason });
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
        ops::cease_to_exist(cx, zone, id);
        acted = true;
    }

    // CR 704.5d: a token anywhere but the battlefield ceases to exist; a phased-out one is still on it (CR 702.26d).
    let gone: Vec<(Zone, ObjectId)> = cx
        .s()
        .zones
        .iter()
        .flatten()
        .copied()
        .filter(|&id| cx.s().obj(id).token && !cx.s().obj(id).zone.is(ZoneKind::Battlefield) && !cx.s().obj(id).zone.is(ZoneKind::PhasedOut))
        .map(|id| (cx.s().obj(id).zone, id))
        .collect();
    for (zone, id) in gone {
        ops::cease_to_exist(cx, zone, id);
        acted = true;
    }

    // CR 702.131b, 702.195a: the city's blessing and an enduring story, for the rest of the game, by a board that changed
    // with no event (an effect ending, a type changing).
    if crate::designations::keyword_designations(cx) {
        acted = true;
    }
    // CR 704.5aa: a player with no speed who controls a permanent with start your engines! gets a speed of 1.
    if crate::speed::engines_start(cx) {
        acted = true;
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
    let mut annihilate: Vec<(ObjectId, i16)> = Vec::new();
    let mut roles: Vec<(ObjectId, ObjectId, PlayerId, u32)> = Vec::new();
    for &id in cx.s().battlefield() {
        // CR 704.5q: a permanent with both +1/+1 and -1/-1 counters loses N of each, N the smaller number. Power and
        // toughness are the same either way, so the other actions of this check are judged alike; removed below.
        let both = cx.s().obj(id).counters(CounterKind::PlusOne).min(cx.s().obj(id).counters(CounterKind::MinusOne));
        if both > 0 {
            annihilate.push((id, both));
        }
        let chars = cx.chars(id);
        if let Some(daybound) = crate::day_night::role(chars.keywords) {
            bound.push(crate::day_night::Bound { id, daybound });
        }
        let obj = cx.s().obj(id);
        // CR 704.5j: not for a player for whom an effect says the legend rule doesn't apply (Hall of Echoes), nor for a
        // permanent a static ability exempts (Mirror Gallery).
        if chars.supertypes.legendary
            && !cx.s().effects.iter().any(|e| e.kind == EffectKind::NoLegendRule(obj.controller))
            && !crate::static_rules::legend_rule_exempt(cx, id)
        {
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
        if let Some(host) = obj.attached_to
            && chars.has_subtype(Subtype::Role)
        {
            roles.push((id, host, obj.controller, obj.timestamp));
        }
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
        // CR 704.5i, unless a static ability says a planeswalker isn't put there for having 0 loyalty.
        if chars.is(CardType::Planeswalker)
            && obj.counters(CounterKind::Loyalty) <= 0
            && !crate::static_rules::zero_loyalty_stays(cx, id)
        {
            to_graveyard.push(id);
        }
    }
    // CR 702.145d, 702.145g, 702.145c, 702.145f: day and night, as far as the permanents with daybound or nightbound go.
    if crate::day_night::apply(cx, &bound)? {
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
                    up_to: false, kind: "", total: None };
                let keep = match cx.ask_core(p, ask, false)? {
                    Answer::Cards(cards) => cards.first().copied().unwrap_or(group[0]),
                    _ => group[0],
                };
                surplus.extend(group.into_iter().filter(|&id| id != keep));
            }
        }
    }
    // CR 704.5z, 303.7a: of the Roles one player controls attached to one permanent, all but the one with the most
    // recent timestamp go to their owners' graveyards, judged on the same board as the rest (CR 704.3), so a Role leaving
    // for another reason in this check still counts. Roles that became attached at the same moment have their relative
    // timestamps in an order their controller would choose (CR 613.7m); the engine gives them consecutive ones, and only
    // copies of one token (a token doubler's) can arrive together on one permanent, so which is kept is the same game.
    let mut roles_surplus = Vec::new();
    for &(id, host, controller, timestamp) in &roles {
        if roles.iter().any(|&(other, h, c, t)| other != id && h == host && c == controller && t > timestamp) {
            roles_surplus.push(id);
        }
    }
    // A permanent that satisfies several of these changes zones once, and what is not destruction wins over it. A Saga is
    // sacrificed by its controller (CR 714.4, 704.5s: "that Saga's controller sacrifices it"), so "whenever you sacrifice"
    // sees it; the rest are put into graveyards, which is no sacrifice.
    let sagas_sacrificed = sacrificed.clone();
    for id in sacrificed.into_iter().chain(surplus).chain(roles_surplus) {
        if !to_graveyard.contains(&id) {
            to_graveyard.push(id);
        }
    }
    destroyed.retain(|id| !to_graveyard.contains(id));
    if !to_graveyard.is_empty() || !destroyed.is_empty() {
        // All at once (CR 704.3): a creature that dies to its toughness and one destroyed by damage leave together.
        let mut moves: Vec<ops::Moving> = Vec::new();
        for id in to_graveyard.into_iter().chain(destroyed) {
            let mut moving = ops::Moving::to(id, Zone::graveyard(cx.s().obj(id).owner));
            if sagas_sacrificed.contains(&id) {
                moving.reason = Some(ZoneReason::Sacrifice);
            }
            moves.push(moving);
        }
        ops::move_together(cx, &moves, ZoneReason::StateBased);
        acted = true;
    }
    // CR 704.5q, at the same time as the rest (CR 704.3): a permanent leaving in this check leaves with the counters it had.
    for (id, n) in annihilate {
        if live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
            counters::remove(cx, id, CounterKind::PlusOne, n);
            counters::remove(cx, id, CounterKind::MinusOne, n);
            acted = true;
        }
    }

    // CR 704.5t: a player on the bottommost room of a dungeon with none of its abilities on the stack completes it.
    acted |= crate::designations::complete_dungeons(cx);

    // CR 104.2a: the game ends when one player is left (or none).
    if acted {
        let left: Vec<PlayerId> = cx.s().apnap().collect();
        match left.len() {
            0 => {
                cx.st().set_outcome(Outcome::Draw);
                cx.record(Event::GameDrawn { reason: DrawReason::AllLost });
            }
            1 => cx.st().set_outcome(Outcome::Won(left[0])),
            _ => {}
        }
    }

    // Effects on objects that are gone apply to nothing (CR 611.2c); dropping them keeps the list short.
    // "For as long as [source] remains on the battlefield" ends with it (CR 611.2b), and "for as long as it has a [kind]
    // counter on it" with the last such counter (`ops::end_counter_durations` ends it as the counter comes off; this is the
    // backstop for any other path).
    let state = cx.s();
    let applies = |e: &cantrip_state::ContinuousEffect| {
        (live(state, e.affected) || e.kind.outlives_affected())
            && (e.duration != Duration::WhileSourceOnBattlefield
                || (live(state, e.source) && state.obj(e.source).zone.is(ZoneKind::Battlefield)))
            && e.duration.holds_for(state.obj(e.affected))
    };
    if state.effects.iter().any(|e| !applies(e)) {
        let alive: Vec<bool> = state.effects.iter().map(applies).collect();
        // By index: `effects_retain` visits the effects from the end, so a counter over them would mismatch.
        cx.st().effects_keep(&alive);
        // A control-changing effect that ended with them gives its permanent back now (CR 613.1b), and the state-based
        // actions are checked again with the new controllers.
        acted |= ops::settle_control(cx);
    }
    // "For as long as you control [this permanent]" permissions end with the permanent's leaving (CR 611.2b, 400.7); a
    // change of control or phasing out ends them as it happens (`refresh_control`, `phase_out`).
    if !cx.s().permissions.is_empty() {
        cx.st().end_lapsed_permissions();
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
    of: &mut Vec<u8>,
    first_spec: u8,
) -> R<bool> {
    let asker = Asker { source: trigger.source, controller: trigger.controller };
    let start = targets.len();
    let slots = crate::resolve::expand_specs(cx.s(), trigger.controller, specs);
    // "If [it] was kicked, instead choose ..." (CR 702.33a, 601.2c), of the source.
    let kicked = cx.s().obj(trigger.source).kicked;
    for (spec, slot_player, at) in &slots {
        let spec = &spec.when_kicked(kicked);
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
            // A bound that is the "that many" of a reflexive ability (CR 603.12), or else the number of times the source
            // was kicked, or its X (CR 702.33d, 107.3m).
            let source = cx.s().obj(trigger.source);
            let counted = trigger.count.unwrap_or(if source.x > 0 { source.x } else { source.kicked as u16 });
            let (candidates, total) = crate::actions::within_total(cx, spec, counted, candidates);
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
            if all_of_few && spec.min > 0 && candidates.len() <= spec.max as usize && total.is_none() {
                targets.extend(candidates);
                break 'slot;
            }
            if most == 0 {
                break 'slot;
            }
            // A stack item holds at most MAX_TARGETS targets: a count past what is left of them is cut to it (known-issues KI-3).
            let room = cantrip_state::MAX_TARGETS.saturating_sub(targets.len());
            let count = (most as usize).min(candidates.len()).min(room) as u8;
            let ask = RequestKind::ChooseTarget { label: spec.label, slot_player: *slot_player,
                source: trigger.source,
                purpose: Purpose::Targets,
                candidates,
                optional: least == 0,
                least: least.min(count),
                count,
                // "Any number of targets" (CR 115.1d) is said so, not as "up to" the candidates there are.
                any_number: spec.is_any_number(),
                total,
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
        // Each target with the spec it was chosen for (`StackItem::spec_of`), numbered across a modal ability's modes.
        of.resize(targets.len(), first_spec + at);
    }
    while targets.len() > start && targets.last() == Some(&Target::None) {
        targets.pop();
    }
    of.truncate(targets.len());
    Ok(true)
}

fn put_trigger(cx: &mut Cx, trigger: PendingTrigger) -> R<()> {
    let def = triggered(cx, trigger.ability);
    let asker = Asker { source: trigger.source, controller: trigger.controller };
    let mut targets = Vec::new();
    let mut of: Vec<u8> = Vec::new();
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
        let mut first_spec = 0u8;
        for (n, mode) in modal.modes.iter().enumerate() {
            for _ in 0..modal.times(mask, n) {
                if !choose_trigger_targets(cx, &trigger, mode.targets, true, &mut targets, &mut of, first_spec)? {
                    return Ok(());
                }
                first_spec += mode.targets.len() as u8;
            }
        }
    } else if let Some(divide) = def.divide
        && let Some(spec) = def.targets.first()
    {
        // CR 601.2d, 603.3d: a divided slot's targets and shares are chosen as the ability goes on the stack.
        let x = trigger.count.unwrap_or(cx.s().obj(trigger.source).x);
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
    } else if !choose_trigger_targets(cx, &trigger, def.targets, false, &mut targets, &mut of, 0)? {
        return Ok(());
    }
    let mut item = StackItem {
        kind: StackKind::Triggered,
        object: trigger.source,
        ability: trigger.ability,
        controller: trigger.controller,
        targets: [Target::None; cantrip_state::MAX_TARGETS],
        target_count: targets.len() as u8,
        spec_of: [0; cantrip_state::MAX_TARGETS],
        x: trigger.count.unwrap_or(0),
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
        flashback: false,
        // Put on the stack now, or, a delayed triggered ability, created when its `Delayed` was (CR 701.27f).
        since: trigger.since.unwrap_or(cx.g.log.len() as u32),
        copied: false,
    };
    item.targets[..targets.len()].copy_from_slice(&targets);
    item.spec_of[..of.len()].copy_from_slice(&of);
    item.shares[..shares.len()].copy_from_slice(&shares);
    cx.st().stack_push(item);
    ops::became_targets(cx, targets.iter().copied(), trigger.source, trigger.controller);
    // The controller of a triggered ability puts it on the stack (CR 603.3a) and is who commits its crime (CR 700.13).
    ops::commit_crime(cx, trigger.controller, trigger.source, &targets);
    Ok(())
}

/// The static abilities that made one triggering trigger additional times (CR 603.2d): each the permanent and the index of its
/// `CardDef::additional_triggers` entry, so the same one met again among a batch's events adds nothing more.
pub(crate) type Again = Vec<(ObjectId, u8)>;

type Found = Vec<(PendingTrigger, &'static TriggeredDef, Again)>;

/// The triggered abilities of `source` that `event` triggers, onto `found`: those that look back (CR 603.10a) or those that
/// don't, as `look_back` says. With `later`, both, the object's characteristics found once: the look-back ones onto `found`,
/// the others onto `later`.
///
/// `permanent`: the source is on the battlefield (or was, for a look-back ability); false for an emblem in the command zone,
/// which is no permanent (CR 114.4), so "a triggered ability of a permanent" does not mean its abilities.
#[allow(clippy::too_many_arguments)]
fn consider(
    cx: &Cx,
    source: ObjectId,
    look_back: bool,
    permanent: bool,
    event: &Event,
    mask: EventMask,
    statics: &mut TriggerStatics,
    found: &mut Found,
    mut later: Option<&mut Found>,
) {
    let chars = layers::chars(cx.cards(), cx.s(), &cx.g.log, source);
    let controller = cx.s().obj(source).controller;
    for ability in chars.abilities() {
        // A state trigger (CR 603.8) triggers as soon as its state is true, so it is asked after every event, of the board as
        // it is now (never of a look-back pass), as well as each time a player would receive priority (`state_triggers`).
        if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
            && let Some(state) = t.state
        {
            if (later.is_some() || !look_back)
                && state(cx, &Asker { source, controller })
                && !state_trigger_waiting(cx, source, ability, found, later.as_deref())
            {
                found.push((state_trigger(source, ability, controller), t, Again::new()));
            }
            continue;
        }
        if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
            && (later.is_some() || t.look_back == look_back)
            && let Some(fired) = if t.chapter > 0 { chapter_reached(source, t.chapter, event) } else if t.listens.intersects(mask) { t.fires(cx, &Asker { source, controller }, event) } else { None }
        {
            if let Some(condition) = t.condition
                && !condition(cx, &Asker { source, controller })
            {
                continue;
            }
            // "Ward abilities of those creatures don't trigger" (CR 702.21a, Nowhere to Run): asked only of a ward that matched.
            if t.ward && layers::targeting_rule(cx.cards(), cx.s(), &cx.g.log, TargetingRuleKind::SuppressWard, source) {
                continue;
            }
            let triggering = Triggering { source, controller, def: t, permanent };
            if statics.stop(cx, event, &triggering) {
                continue;
            }
            let mut again = Again::new();
            statics.additional(cx, event, &triggering, &mut again);
            let into = match later.as_deref_mut() {
                Some(later) if !t.look_back => later,
                _ => &mut *found,
            };
            into.push((
                PendingTrigger { source, ability, controller, subject: fired.subject, data: fired.data, members: Span::default(), count: None, since: None },
                t,
                again,
            ));
        }
    }
}

/// The triggering of a state trigger (CR 603.8): about no event, so nothing is its subject.
fn state_trigger(source: ObjectId, ability: AbilityRef, controller: PlayerId) -> PendingTrigger {
    PendingTrigger { source, ability, controller, subject: Target::None, data: [0; 2], members: Span::default(), count: None, since: None }
}

/// Whether a state trigger of `source` has triggered and not yet left the stack (CR 603.8): found for this batch, waiting
/// to be put on the stack, or on it. Until then it doesn't trigger again. A copy of it on the stack (Return the Favor) is
/// not it: it was not triggered (CR 707.10). An object that has changed zones is a new object (CR 400.7), whose ability
/// is another.
fn state_trigger_waiting(cx: &Cx, source: ObjectId, ability: AbilityRef, found: &Found, later: Option<&Found>) -> bool {
    let this = |t: &PendingTrigger| t.source == source && t.ability == ability;
    found.iter().chain(later.into_iter().flatten()).any(|(t, _, _)| this(t))
        || cx.s().triggers.iter().any(this)
        || cx.s().stack.iter().any(|i| i.kind == StackKind::Triggered && !i.copied && i.object == source && i.ability == ability)
}

/// The state triggers whose states are true each time a player would receive priority (CR 603.8, 117.5): of the permanents
/// and of the emblems in the command zone (CR 114.4). A state that became true with no event to ask it after, such as an
/// effect ending, is found here.
fn state_triggers(cx: &mut Cx) {
    let mut found: Found = Vec::new();
    let sources: Vec<ObjectId> = cx.s().battlefield().iter().chain(cx.s().zone(Zone::COMMAND)).copied().collect();
    for source in sources {
        let chars = layers::chars(cx.cards(), cx.s(), &cx.g.log, source);
        let controller = cx.s().obj(source).controller;
        for ability in chars.abilities() {
            if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
                && let Some(state) = t.state
                && state(cx, &Asker { source, controller })
                && !state_trigger_waiting(cx, source, ability, &found, None)
            {
                found.push((state_trigger(source, ability, controller), t, Again::new()));
            }
        }
    }
    for (trigger, _, _) in found {
        cx.st().trigger_push(trigger);
    }
}

/// A Saga's chapter ability (CR 714.2b) triggers when lore counters are put on it and their number becomes at least
/// its chapter number, having been less than it.
fn chapter_reached(source: ObjectId, chapter: u8, event: &Event) -> Option<Fired> {
    match event {
        Event::CountersPut { object, kind: CounterKind::Lore, amount, total, .. } if *object == source => {
            if total - amount < chapter as i16 && *total >= chapter as i16 { Fired::PLAIN } else { None }
        }
        _ => None,
    }
}

/// The permanents with static abilities that stop abilities from triggering (CR 603.2, `CardDef::trigger_rules`) or make them
/// trigger additional times (CR 603.2d, `CardDef::additional_triggers`): found from the board, once per batch and only when an
/// ability has matched, so a batch that triggers nothing pays nothing. An ability that triggers on the board as it is after the
/// events asks those on the battlefield now; a leaves-the-battlefield ability, which looks back (CR 603.10a), asks those that
/// were there before, the ones leaving with the event included.
pub(crate) struct TriggerStatics<'a> {
    before: Option<&'a [ObjectId]>,
    now: Option<Vec<ObjectId>>,
    then: Option<Vec<ObjectId>>,
}

impl<'a> TriggerStatics<'a> {
    pub(crate) fn new(before: Option<&'a [ObjectId]>) -> Self {
        TriggerStatics { before, now: None, then: None }
    }

    /// The permanents whose rules apply to this triggering (CR 603.10).
    fn sources(&mut self, cx: &Cx, triggering: &Triggering) -> &[ObjectId] {
        if triggering.def.look_back && self.before.is_some() {
            let before = self.before.unwrap_or_default();
            self.then.get_or_insert_with(|| trigger_rule_sources(cx, before))
        } else {
            self.now.get_or_insert_with(|| trigger_rule_sources(cx, cx.s().battlefield()))
        }
    }

    /// Whether a static ability stops this ability triggering on this event.
    pub(crate) fn stop(&mut self, cx: &Cx, event: &Event, triggering: &Triggering) -> bool {
        self.sources(cx, triggering).iter().any(|&id| {
            let obj = cx.s().obj(id);
            let asker = Asker { source: id, controller: obj.controller };
            cx.card(cx.cards().current(obj)).trigger_rules.iter().any(|rule| rule(cx, &asker, event, triggering))
        })
    }

    /// The static abilities that make this ability, which this event triggered, trigger an additional time (CR 603.2d): one
    /// more for each, added to `again` unless already there.
    pub(crate) fn additional(&mut self, cx: &Cx, event: &Event, triggering: &Triggering, again: &mut Again) {
        for &id in self.sources(cx, triggering) {
            let obj = cx.s().obj(id);
            let asker = Asker { source: id, controller: obj.controller };
            for (n, rule) in cx.card(cx.cards().current(obj)).additional_triggers.iter().enumerate() {
                if rule(cx, &asker, event, triggering) && !again.contains(&(id, n as u8)) {
                    again.push((id, n as u8));
                }
            }
        }
    }
}

/// Those of `ids` whose card has a rule that stops abilities from triggering or makes them trigger again, and has not lost
/// its abilities (CR 613.1f).
fn trigger_rule_sources(cx: &Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter()
        .copied()
        .filter(|&id| cx.cards().trigger_rules[cx.cards().current(cx.s().obj(id)).0 as usize] && !cx.lost_all_abilities(id))
        .collect()
}

/// Every triggered ability this event triggers (CR 603.2).
pub fn collect_triggers(cx: &mut Cx, event: &Event, before: Option<&[ObjectId]>) {
    collect_batch(cx, std::slice::from_ref(event), before);
}

/// Every triggered ability a batch of simultaneous events triggers (CR 603.2): leaves-the-battlefield abilities among
/// the permanents that were on the battlefield before it (`before`), the rest among those there now, and abilities
/// that function everywhere wherever they are. A "one or more" ability (CR 603.2c) triggers once for the batch, its
/// members the subjects of the events it matched; a limited one ("triggers only N times each turn", no numbered rule; CR 603.2h is "do this only once each turn") stops triggering once it has used up its
/// turn, a triggering that happens being noted in the history.
pub fn collect_batch(cx: &mut Cx, events: &[Event], before: Option<&[ObjectId]>) {
    // A designation the events gave is had before anything they trigger is looked for, and the continuous effects that
    // read it apply to that look (CR 702.131d, 702.195c): Bifur, Melodic Rider entering for the third storied permanent
    // makes its own enters ability trigger an additional time.
    crate::designations::keyword_designations(cx);
    let mut found: Found = Vec::new();
    let mut statics = TriggerStatics::new(before);
    for event in events {
        let mask = EventMask::of(event);
        if mask.0 == 0 {
            continue;
        }
        let now: Vec<ObjectId> = cx.s().battlefield().to_vec();
        if let Some(before) = before {
            for &id in before {
                consider(cx, id, true, true, event, mask, &mut statics, &mut found, None);
            }
            for &id in &now {
                consider(cx, id, false, true, event, mask, &mut statics, &mut found, None);
            }
        } else {
            // The same permanents for both: each one's characteristics are found once, and the abilities that don't look
            // back come after every one that does, as they would from two passes.
            let mut later: Found = Vec::new();
            for &id in &now {
                consider(cx, id, true, true, event, mask, &mut statics, &mut found, Some(&mut later));
            }
            found.append(&mut later);
        }
        // An emblem's triggered abilities function in the command zone (CR 114.4), which it never leaves, so it is there
        // both before the event and after it: its look-back abilities (a "whenever a creature dies"; CR 603.10a) and the
        // others alike are asked of it, each once, the look-back ones judged by what stopped or multiplied triggering
        // before the event (`TriggerStatics`), as any other's are.
        if !cx.s().zone(Zone::COMMAND).is_empty() {
            let mut later: Found = Vec::new();
            for id in cx.s().zone(Zone::COMMAND).to_vec() {
                consider(cx, id, true, false, event, mask, &mut statics, &mut found, Some(&mut later));
            }
            found.append(&mut later);
        }
        // Delayed abilities of "whenever [event] this turn" (CR 603.7b), each of which triggers on every match.
        if cx.s().delayed.iter().any(|d| d.when.is_event()) {
            for d in cx.s().delayed.clone() {
                // "You" in its trigger event is its own controller, the player who controlled the spell or ability that
                // created it (CR 603.7d-e), whoever controls its source now and wherever the source has gone. So is "you"
                // in an intervening "if" (CR 603.4).
                let asker = Asker { source: d.source, controller: d.controller };
                if d.when.is_event()
                    && let AbilityDef::Triggered(t) = cx.cards().ability(d.ability)
                    // A delayed triggered mana ability never goes on the stack: it resolves as the mana ability that
                    // triggered it does (CR 605.1b, 605.4a), in `tapped_for_mana`.
                    && !t.mana_ability
                    && t.listens.intersects(mask)
                    && let Some(fired) = t.fires(cx, &asker, event)
                    && t.condition.is_none_or(|condition| condition(cx, &asker))
                    // The end of an "until" exile is no triggered ability but the second one-shot effect the rules create
                    // immediately after the event (CR 610.3), so nothing that stops abilities triggering stops it, and the
                    // event that ends the wait always uses it up below. A "this turn" delayed ability is a triggered ability
                    // (CR 603.7b) and is stopped like any other. It is no ability of a permanent, and nothing makes it trigger an
                    // additional time (CR 603.2d).
                    && (d.when == DelayedWhen::UntilEvent
                        || !statics.stop(cx, event, &Triggering { source: d.source, controller: d.controller, def: t, permanent: false }))
                {
                    found.push((
                        PendingTrigger {
                            source: d.source,
                            ability: d.ability,
                            controller: d.controller,
                            subject: if d.when == DelayedWhen::UntilEvent { d.subject } else { fired.subject },
                            data: fired.data,
                            members: Span::default(),
                            count: None,
                            // A delayed ability goes on the stack with the moment it was created (CR 701.27f).
                            since: Some(d.since),
                        },
                        t,
                        Again::new(),
                    ));
                    // "Until" is used up by the event that ends the wait (CR 603.7b), and "when you next" by the event it
                    // triggers on (CR 603.7b).
                    if matches!(d.when, DelayedWhen::UntilEvent | DelayedWhen::NextThisTurn)
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
            let chars = layers::chars(cx.cards(), cx.s(), &cx.g.log, id);
            for ability in chars.abilities() {
                if let AbilityDef::Triggered(t) = cx.cards().ability(ability)
                    && t.anywhere
                    && t.listens.intersects(mask)
                    && let Some(fired) = t.fires(cx, &Asker { source: id, controller: cx.s().obj(id).controller }, event)
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
                    let triggering = Triggering { source: id, controller, def: t, permanent: false };
                    if statics.stop(cx, event, &triggering) {
                        continue;
                    }
                    let mut again = Again::new();
                    statics.additional(cx, event, &triggering, &mut again);
                    found.push((
                        PendingTrigger { source: id, ability, controller, subject: fired.subject, data: fired.data, members: Span::default(), count: None, since: None },
                        t,
                        again,
                    ));
                }
            }
        }
    }
    if found.is_empty() {
        return;
    }

    // One trigger per "one or more" ability and source for the batch, keeping the first; the rest add their subjects, and the
    // static abilities that make it trigger again for their events (CR 603.2d).
    let mut kept: Vec<(PendingTrigger, &'static TriggeredDef, Vec<ObjectId>, Again)> = Vec::with_capacity(found.len());
    for (trigger, def, again) in found {
        if def.once_per_batch
            && let Some(existing) =
                kept.iter_mut().find(|(k, d, _, _)| {
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
            add_again(&mut existing.3, again);
            continue;
        }
        if def.sums_per_subject
            && let Some(existing) = kept.iter_mut().find(|(k, d, _, _)| {
                d.sums_per_subject && k.source == trigger.source && k.ability == trigger.ability && k.subject == trigger.subject
            })
        {
            existing.0.data[0] += trigger.data[0];
            add_again(&mut existing.3, again);
            continue;
        }
        let mut members = Vec::new();
        if def.once_per_batch
            && let Target::Object(member) = trigger.subject
        {
            members.push(member);
        }
        kept.push((trigger, def, members, again));
    }

    for (mut trigger, def, members, again) in kept {
        // It triggers once, and once more for each static ability that says so (CR 603.2d): each a triggering of its own,
        // counted against "this ability triggers only N times each turn" (no numbered rule; CR 603.2h is "do this only once each turn").
        for n in 0..=again.len() {
            if def.triggers_each_turn > 0 {
                let limit = def.triggers_each_turn as usize;
                if crate::loops::settled_read(cx, |h| h.triggerings(trigger.source, trigger.ability) >= limit, |&reached| reached) {
                    break;
                }
                cx.record(Event::Triggered { source: trigger.source, ability: trigger.ability });
            }
            if n == 0 && !members.is_empty() {
                trigger.members = cx.st().members_push(&members);
            }
            cx.st().trigger_push(trigger);
        }
    }
}

/// The static abilities that make a "one or more" or summed ability trigger again for one more of its batch's events, added
/// to those already found for it: each counts once for the batch (CR 603.2c, 603.2d).
fn add_again(into: &mut Again, again: Again) {
    for a in again {
        if !into.contains(&a) {
            into.push(a);
        }
    }
}

/// A step has begun: its "at the beginning of" abilities trigger, and so do the delayed abilities waiting for it,
/// which are used up (CR 603.7c) unless they wait for each such step for the rest of the game.
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
        let (due, used_up) = match d.when {
            DelayedWhen::Step { step: s, on_turn_of } => (s == step && on_turn_of.is_none_or(|p| p == active), true),
            DelayedWhen::EachStep { step: s, on_turn_of } => (s == step && on_turn_of.is_none_or(|p| p == active), false),
            DelayedWhen::ExtraTurnStep { step: s, turn } => (s == step && cx.s().turn.extra == turn, true),
            DelayedWhen::ThisTurn | DelayedWhen::UntilEvent | DelayedWhen::UntilNextTurnOf(_) | DelayedWhen::NextThisTurn => (false, false),
        };
        if due {
            if used_up {
                cx.st().delayed_remove(at);
            } else {
                at += 1;
            }
            cx.st().trigger_push(PendingTrigger { source: d.source, ability: d.ability, controller: d.controller, subject: d.subject, data: [0; 2], members: d.members, count: None, since: Some(d.since) });
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
            count: None,
            since: None,
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
