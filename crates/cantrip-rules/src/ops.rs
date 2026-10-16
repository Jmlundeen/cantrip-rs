//! The operations everything else is built from: zone changes, draws, damage, life, counters. Each is one rules
//! procedure, each writes through the journal, and each announces what it did.

use cantrip_api::*;
use cantrip_state::PendingTrigger;

use crate::layers::{self, live};
use crate::{Cx, R, replace};

/// A player commits a crime (CR 700.13) as they cast a spell, activate an ability or put a triggered ability on the stack
/// that targets at least one opponent; at least one permanent, spell or ability an opponent controls; and/or at least one
/// card in an opponent's graveyard: called by those three procedures, once each with the targets chosen, after their
/// `BecameTarget` events. One crime is recorded however many targets qualify. Each target is judged as it is now, as the
/// object becomes cast, activated or put on the stack; a permanent a cost took away is judged by its row as it left. A
/// card in exile, a hand or a library is no crime, nor is anything of the player's own.
pub(crate) fn commit_crime(cx: &mut Cx, player: PlayerId, by: ObjectId, targets: &[Target]) {
    let s = cx.s();
    let opponent = |p: PlayerId| p != player && s.opponents(player).any(|o| o == p);
    let crime = targets.iter().any(|&t| match t {
        Target::Player(p) => opponent(p),
        Target::Object(id) => {
            let o = s.obj(id);
            match o.zone.kind() {
                ZoneKind::Battlefield | ZoneKind::Stack => opponent(o.controller),
                ZoneKind::Graveyard => opponent(o.zone.player),
                _ => false,
            }
        }
        Target::Ability(serial) => s.stack.iter().any(|i| i.serial == serial && i.kind != StackKind::Spell && opponent(i.controller)),
        Target::None => false,
    });
    if crime {
        cx.emit(Event::CrimeCommitted { player, by });
    }
}

/// One object about to move, and how it arrives if it is entering the battlefield.
#[derive(Clone, Copy)]
pub struct Moving {
    pub id: ObjectId,
    pub to: Zone,
    pub entering: Option<Entering>,
    /// Why this move happens when it is not why the rest of the batch does: a sacrifice and a return made at once (CR 101.4).
    pub reason: Option<ZoneReason>,
}

impl Moving {
    pub fn to(id: ObjectId, to: Zone) -> Moving {
        Moving { id, to, entering: None, reason: None }
    }
}

/// Moves objects at once (CR 400.7: each arrives as a new object). Leaves-the-battlefield abilities are judged on
/// the board as it was before any of them moved (CR 603.10a); everything else on the board after. A library is
/// entered on top. Returns the new objects, in order.
pub fn move_together(cx: &mut Cx, moves: &[Moving], reason: ZoneReason) -> Vec<ObjectId> {
    // Destruction a regeneration shield replaces never happens (CR 701.19a, 614.8).
    let unregenerated;
    let moves = if reason == ZoneReason::Destroy {
        unregenerated = without_regenerated(cx, moves);
        &unregenerated[..]
    } else {
        moves
    };
    let before: Vec<ObjectId> = cx.s().battlefield().to_vec();
    let mut events = Vec::with_capacity(moves.len());
    let mut created = Vec::with_capacity(moves.len());
    // CR 701.3d: every attachment these moves end, read before any is made: an attachment leaving the battlefield,
    // and every one attached to something leaving. Each pair once.
    for &a in &before {
        let Some(host) = cx.s().obj(a).attached_to else { continue };
        let leaving = |id: ObjectId| moves.iter().any(|m| m.id == id);
        if live(cx.s(), host)
            && cx.s().obj(host).zone.is(ZoneKind::Battlefield)
            && (leaving(a) || leaving(host))
            && !events.iter().any(|e| matches!(e, Event::Unattached { attachment, .. } if *attachment == a))
        {
            events.push(Event::Unattached { attachment: a, from: host });
        }
    }
    for m in moves {
        let reason = m.reason.unwrap_or(reason);
        let old = *cx.s().obj(m.id);
        let from = old.zone;
        // A permanent that would leave the battlefield for anywhere else is exiled instead (CR 702.84a).
        let mut to = m.to;
        // An entry its own replacement turned into a move elsewhere (CR 614.12): it does not enter at all.
        let mut m = *m;
        if let Some(instead) = m.entering.and_then(|e| e.instead) {
            to = instead;
            m.entering = None;
        }
        let m = &m;
        if from.is(ZoneKind::Battlefield) && old.exile_on_leave && !to.is(ZoneKind::Exile) {
            to = Zone::exile(old.owner);
        }
        // CR 122.1h: a permanent with a finality counter that would be put into a graveyard from the battlefield is exiled
        // instead.
        if from.is(ZoneKind::Battlefield) && to.is(ZoneKind::Graveyard) && old.counters(CounterKind::Finality) > 0 {
            to = Zone::exile(old.owner);
        }
        // A card goes to its owner's own hand, library or graveyard, whoever puts it there (CR 400.3).
        let to = match to.kind() {
            ZoneKind::Library | ZoneKind::Hand | ZoneKind::Graveyard | ZoneKind::Exile => Zone::new(to.kind(), old.owner),
            _ => to,
        };
        // Replacement effects of the permanents in play, and of the card itself (CR 614, 616): they change where it goes,
        // how it arrives, or what it arrives with.
        let entry = m.entering.filter(|_| to.is(ZoneKind::Battlefield));
        let change = replace::zone_change(
            cx,
            ZoneChange {
                object: m.id,
                from,
                to,
                reason,
                controller: entry.map_or(old.controller, |e| e.controller),
                tapped: entry.is_some_and(|e| e.tapped),
                counters: [0; CounterKind::COUNT],
                reveal: false,
                shuffle: false,
                is_card: !old.token && !old.card_copy,
                madness: false,
            },
        );
        let to = change.to;
        // Madness (CR 702.35a) exiled a discarded card in place of its graveyard, one of the replacements above: it was
        // still discarded, and the linked ability is queued below, once the exiled card exists - if no later effect sent
        // it on elsewhere, so that it was "exiled this way".
        let madness = change.madness && to.is(ZoneKind::Exile);
        if from.is(ZoneKind::Battlefield) {
            let c = layers::chars(cx.cards(), cx.s(), m.id);
            let words = cantrip_api::words(cx.s(), m.id);
            cx.st().last_known_push(LastKnown {
                id: m.id,
                types: c.types,
                subtypes: c.subtypes,
                colors: c.colors,
                power: c.power,
                toughness: c.toughness,
                keywords: c.keywords,
                abilities: c.abilities,
                words,
            });
        }
        cx.st().zone_remove(from, m.id);
        // A spell that leaves the stack by any move other than resolving or being countered (exiled, bounced) does not
        // resolve: it is gone from the stack.
        if from.is(ZoneKind::Stack)
            && let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == m.id)
        {
            cx.st().stack_remove(at);
        }
        // Revolt (CR 207.2c) asks whether a permanent left the battlefield under its controller this turn.
        if from.is(ZoneKind::Battlefield) {
        }

        let turn = cx.s().turn.number;
        let timestamp = cx.st().timestamp();
        let entering = to.is(ZoneKind::Battlefield);
        let mut new = Obj {
            card: old.card,
            owner: old.owner,
            controller: if entering || to.is(ZoneKind::Stack) { old.controller } else { old.owner },
            zone: to,
            token: old.token,
            tapped: false,
            controlled_since: turn,
            damage: 0,
            deathtouched: false,
            counters: [0; CounterKind::COUNT],
            timestamp,
            copy: None,
            loyalty_turn: 0,
            became: None,
            exile_on_leave: false,
            kicked: if entering && from.is(ZoneKind::Stack) { old.kicked } else { 0 },
            gift: if entering && from.is(ZoneKind::Stack) { old.gift } else { None },
            paid: if entering && from.is(ZoneKind::Stack) { old.paid } else { 0 },
            spent: if entering && from.is(ZoneKind::Stack) { old.spent } else { ManaPool::default() },
            cast_way: if entering && from.is(ZoneKind::Stack) { old.cast_way } else { 0 },
            class_level: 0,
            prepared: false,
            card_copy: old.card_copy,
            prepared_from: None,
            attached_to: None,
            attached_player: None,
            entered_with: None,
            chosen_color: None,
            chosen_subtype: None,
            on_adventure: false,
            cast_from: None,
            activated_turn: 0,
            activated_mask: 0,
            x: if entering && from.is(ZoneKind::Stack) { old.x } else { 0 },
            marks: Marks {
                cast_untimely: entering && from.is(ZoneKind::Stack) && old.marks.cast_untimely,
                // CR 709.5d: a Room enters with the unlocked designation of the half that was cast as a spell, and with
                // neither if neither was.
                doors: if entering && from.is(ZoneKind::Stack) && cx.cards().get(old.card).is_room() { cast_door(old.cast_way) } else { 0 },
                ..Marks::default()
            },
            transformed: false,
            exiled: None,
            face_down: false,
            seen_by: 0,
        };
        if from.is(ZoneKind::Graveyard) && !old.token {
        }
        // What the replacements left of its arrival: tapped, and the counters they give it, a void counter on a card
        // exiled in place of a graveyard as much as those of a permanent (CR 122.6).
        new.tapped = entering && change.tapped;
        new.counters = change.counters;
        if let Some(e) = m.entering.filter(|_| entering) {
            for (n, with) in new.counters.iter_mut().zip(e.counters) {
                *n += with;
            }
            new.controller = e.controller;
            new.copy = e.copy;
            new.chosen_color = e.chosen_color;
            new.chosen_subtype = e.chosen_subtype;
            new.marks.chosen_player = e.chosen_player;
            new.marks.chosen_card = e.chosen_card;
            new.marks.chosen_odd = e.chosen_odd;
            new.marks.protector = e.protector;
            new.attached_to = e.attach;
            new.attached_player = e.attach_player;
            new.transformed = e.transformed;
        }
        // CR 708.3: an object turned face down before it entered is a face-down permanent as it arrives; its card's own
        // values are what it is turned face up as (CR 708.2a).
        if entering && !from.is(ZoneKind::Battlefield) && old.copy.is_some_and(|c| c.card == cx.cards().face_down) {
            new.copy = old.copy;
            new.face_down = true;
        }
        if entering {
            enters_with_intrinsic_counters(cx.cards(), &mut new);
            // Counters it enters with are counters put on it (CR 122.6, 614.1a): replacements that change how many
            // apply to them, the permanent judged as it would exist on the battlefield (CR 614.12).
            new.counters = replace::counters(cx, m.id, new.controller, true, new.counters);
        }
        let entered_with = new.counters;
        let new_id = cx.st().push_obj(new);
        note_statics(cx, new_id);
        if to.is(ZoneKind::Library) {
            cx.st().zone_insert(to, 0, new_id);
        } else {
            cx.st().zone_push(to, new_id);
        }
        cx.st().update_obj(m.id, |o| o.became = Some(new_id));
        // "Reveal it and shuffle it into its owner's library instead" (CR 701.20a, 701.24a).
        if change.reveal {
            cx.emit(Event::Revealed { player: old.owner, card: new_id });
        }
        if change.shuffle && to.is(ZoneKind::Library) {
            crate::turn::shuffle(cx.st(), to);
        }
        if madness {
            // "When you do" (CR 702.35a, 603.12): a reflexive trigger of the exiled card, its owner's.
            cx.st().trigger_push(PendingTrigger {
                source: new_id,
                ability: AbilityRef::madness(old.card),
                controller: old.owner,
                subject: Target::None,
                data: [0; 2],
                members: cantrip_state::Span::default(),
                count: None,
            });
        }

        // A permanent that leaves the battlefield leaves combat (CR 506.4).
        if from.is(ZoneKind::Battlefield) {
            let id = m.id;
            if cx.s().attackers.iter().any(|a| a.id == id) {
                let attackers = cx.s().attackers.iter().copied().filter(|a| a.id != id).collect();
                cx.st().set_attackers(attackers);
            }
            if cx.s().blocks.iter().any(|b| b.blocker == id || b.attacker == id) {
                let blocks = cx.s().blocks.iter().copied().filter(|b| b.blocker != id && b.attacker != id).collect();
                cx.st().set_blocks(blocks);
            }
        }
        events.push(Event::ZoneChange { old: m.id, new: new_id, from, to, reason });
        // CR 709.5h: a door unlocked as the Room enters is unlocked all the same ("when you unlock this door" triggers).
        if entering && cx.s().obj(new_id).marks.doors != 0 {
            events.push(Event::DoorUnlocked { permanent: new_id, door: cx.s().obj(new_id).marks.doors, player: cx.s().obj(new_id).controller });
        }
        // The counters a permanent enters with were put on it (CR 122.6).
        if entering {
            let by = cx.s().obj(new_id).controller;
            counters_entered_with(&mut events, new_id, by, &entered_with);
        }
        created.push(new_id);
        if m.entering.is_some_and(|e| e.prepared) && entering {
            become_prepared(cx, new_id);
        }
    }
    cx.emit_batch(&events, Some(&before));
    created
}

/// Attaches an Aura or Equipment to a permanent (CR 701.3a). Nothing happens when it could not be attached there
/// (CR 701.3b) or already is. Attaching reassigns its timestamp (CR 701.3c, 613.7e).
pub fn attach(cx: &mut Cx, attachment: ObjectId, host: ObjectId) {
    if !live(cx.s(), attachment)
        || !cx.s().obj(attachment).zone.is(ZoneKind::Battlefield)
        || cx.s().obj(attachment).attached_to == Some(host)
        || !crate::attach::host_allows(cx, attachment, host, true)
        || !cx.s().obj(host).zone.is(ZoneKind::Battlefield)
    {
        return;
    }
    // Moving it off what it was on unattaches it from that first (CR 701.3d).
    if let Some(old) = cx.s().obj(attachment).attached_to
        && live(cx.s(), old)
        && cx.s().obj(old).zone.is(ZoneKind::Battlefield)
    {
        cx.emit(Event::Unattached { attachment, from: old });
    }
    let timestamp = cx.st().timestamp();
    cx.st().update_obj(attachment, |o| {
        o.attached_to = Some(host);
        o.timestamp = timestamp;
    });
}

/// Unattaches an Aura or Equipment (CR 701.3d); it stays where it is.
pub fn unattach(cx: &mut Cx, attachment: ObjectId) {
    let Some(host) = cx.s().obj(attachment).attached_to else { return };
    if !live(cx.s(), attachment) {
        return;
    }
    cx.st().update_obj(attachment, |o| o.attached_to = None);
    // A host that has already left was unattached from as it left, and announced then.
    if live(cx.s(), host) && cx.s().obj(host).zone.is(ZoneKind::Battlefield) {
        cx.emit(Event::Unattached { attachment, from: host });
    }
}

/// Exiles cards with a permanent's ability (CR 607.2a, 610.3), linking each to it and to where it came from.
pub fn exile_with(cx: &mut Cx, cards: &[ObjectId], by: ObjectId, until_leaves: bool) -> Vec<ObjectId> {
    // CR 610.3b: the duration has ended before the exile began, so nothing is exiled.
    if until_leaves && !(live(cx.s(), by) && cx.s().obj(by).zone.is(ZoneKind::Battlefield)) {
        return Vec::new();
    }
    let from: Vec<Zone> = cards.iter().map(|&c| cx.s().obj(c).zone).collect();
    let moves: Vec<Moving> = cards.iter().map(|&c| Moving::to(c, Zone::exile(cx.s().obj(c).owner))).collect();
    let new = move_together(cx, &moves, ZoneReason::Exile);
    for (&n, from) in new.iter().zip(from) {
        if cx.s().obj(n).zone.is(ZoneKind::Exile) {
            cx.st().update_obj(n, |o| o.exiled = Some(ExileLink { by, from, until_leaves, encoded: false }));
        }
    }
    new
}

/// Links cards already in exile to `by` (CR 607.2a): `Game::link_exiled`, for the exiled cards used to craft a permanent
/// (CR 702.167c). Each keeps the zone it came from; none is "until" anything.
pub fn link_exiled(cx: &mut Cx, cards: &[ObjectId], by: ObjectId) {
    for &c in cards {
        if live(cx.s(), c) && cx.s().obj(c).zone.is(ZoneKind::Exile) {
            let from = cx.s().obj(c).exiled.map_or(cx.s().obj(c).zone, |l| l.from);
            cx.st().update_obj(c, |o| o.exiled = Some(ExileLink { by, from, until_leaves: false, encoded: false }));
        }
    }
}

/// Exiles a spell card encoded on a creature (CR 702.99a, 702.99b): exiled linked to it, and marked as encoded there.
pub fn encode(cx: &mut Cx, card: ObjectId, creature: ObjectId) -> Option<ObjectId> {
    let new = *exile_with(cx, &[card], creature, false).first()?;
    if !cx.s().obj(new).zone.is(ZoneKind::Exile) {
        return None;
    }
    cx.st().update_obj(new, |o| {
        if let Some(link) = &mut o.exiled {
            link.encoded = true;
        }
    });
    Some(new)
}

/// Exiles cards together face down (CR 406.3, 708.2): linked to `by` when an ability exiled them (CR 607.2a). A face-down
/// card in exile has no characteristics for a player who may not look at it; those who could see where it came from may
/// (CR 406.3): its owner for a card from a hand, everyone for one from a public zone, nobody for one from a library.
pub fn exile_face_down(cx: &mut Cx, cards: &[ObjectId], by: Option<ObjectId>) -> Vec<ObjectId> {
    let from: Vec<Zone> = cards.iter().map(|&c| cx.s().obj(c).zone).collect();
    let owners: Vec<PlayerId> = cards.iter().map(|&c| cx.s().obj(c).owner).collect();
    let new = match by {
        Some(by) => exile_with(cx, cards, by, false),
        None => {
            let moves: Vec<Moving> = cards.iter().map(|&c| Moving::to(c, Zone::exile(cx.s().obj(c).owner))).collect();
            move_together(cx, &moves, ZoneReason::Exile)
        }
    };
    let everyone = cx.s().seats().fold(0u8, |mask, p| mask | 1 << p.0);
    for ((&n, from), owner) in new.iter().zip(from).zip(owners) {
        if cx.s().obj(n).zone.is(ZoneKind::Exile) {
            let seen_by = match from.kind() {
                ZoneKind::Hand => 1 << owner.0,
                ZoneKind::Library => 0,
                _ => everyone,
            };
            cx.st().update_obj(n, |o| {
                o.face_down = true;
                o.seen_by = seen_by;
            });
        }
    }
    new
}

/// `player` looks at face-down cards in exile (CR 406.3): they may look at them from now on.
pub fn look_at(cx: &mut Cx, player: PlayerId, cards: &[ObjectId]) {
    for &c in cards {
        if live(cx.s(), c) {
            cx.emit(Event::Looked { player, card: c });
        }
        if live(cx.s(), c) && cx.s().obj(c).face_down && cx.s().obj(c).zone.is(ZoneKind::Exile) {
            cx.st().update_obj(c, |o| o.seen_by |= 1 << player.0);
        }
    }
}

/// Puts an object onto the battlefield face down (CR 708.3, 701.40a): it is turned face down before it enters, so what
/// replaces its entering judges a nameless 2/2 creature, and its card's own "enters" replacements do not apply.
pub fn put_onto_battlefield_face_down(cx: &mut Cx, id: ObjectId, controller: PlayerId) -> R<ObjectId> {
    let values = Copiable::of(cx.cards().face_down);
    cx.st().update_obj(id, |o| o.copy = Some(values));
    Ok(put_onto_battlefield(cx, &[(id, controller, false)], ZoneReason::Other)?[0])
}

/// Turns a face-down permanent face up (CR 701.40b, 708.2): it is the card it is, the same object (CR 708.8).
pub fn turn_face_up(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) || !cx.s().obj(id).face_down {
        return;
    }
    cx.st().update_obj(id, |o| {
        o.face_down = false;
        o.copy = None;
    });
    note_statics(cx, id);
}

/// The door whose half was cast, by the way it was cast (CR 709.3, 709.5d): none for anything but a split card's halves.
fn cast_door(way: u8) -> u8 {
    match way {
        WAY_SPLIT_LEFT => DOOR_LEFT,
        WAY_SPLIT_RIGHT => DOOR_RIGHT,
        _ => 0,
    }
}

/// Unlocks a door of a Room (CR 709.5f): the permanent is given the half's unlocked designation, the same object
/// (CR 709.5c), and "when you unlock this door" triggers (CR 709.5h). Nothing if it is not a Room on the battlefield or
/// the door is already unlocked.
pub fn unlock_door(cx: &mut Cx, id: ObjectId, door: u8, player: PlayerId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) || cx.s().obj(id).marks.doors & door != 0 {
        return;
    }
    if !cx.card(cx.chars(id).card).is_room() {
        return;
    }
    cx.st().update_obj(id, |o| o.marks.doors |= door);
    cx.emit(Event::DoorUnlocked { permanent: id, door, player });
}

/// A card name chosen for a permanent as an ability of it resolves (CR 607.2d): kept by the permanent as the object it is
/// now, so a new object it becomes has none (CR 400.7), and a later choice for it replaces an earlier one. Nothing if it is
/// no longer on the battlefield.
pub fn remember_chosen_name(cx: &mut Cx, id: ObjectId, name: CardId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    cx.st().update_obj(id, |o| o.marks.chosen_card = Some(name));
}

/// CR 610.3: cards exiled "until [a permanent] leaves the battlefield" return, as a one-shot effect that follows the
/// event, to the zone they were exiled from; a permanent comes back under its owner's control. Whether any did.
pub fn return_exiled(cx: &mut Cx) -> R<bool> {
    let mut due: Vec<(ObjectId, ExileLink)> = Vec::new();
    for p in cx.s().seats() {
        for &id in cx.s().zone(Zone::exile(p)) {
            if let Some(link) = cx.s().obj(id).exiled
                && link.until_leaves
                && !(live(cx.s(), link.by) && cx.s().obj(link.by).zone.is(ZoneKind::Battlefield))
            {
                due.push((id, link));
            }
        }
    }
    if due.is_empty() {
        return Ok(false);
    }
    let owner = |cx: &Cx, id: ObjectId| cx.s().obj(id).owner;
    let to_battlefield: Vec<(ObjectId, PlayerId, bool)> =
        due.iter().filter(|(_, l)| l.from.is(ZoneKind::Battlefield)).map(|&(id, _)| (id, owner(cx, id), false)).collect();
    let elsewhere: Vec<Moving> = due
        .iter()
        .filter(|(_, l)| !l.from.is(ZoneKind::Battlefield))
        .map(|&(id, l)| Moving::to(id, Zone::new(l.from.kind(), owner(cx, id))))
        .collect();
    if !elsewhere.is_empty() {
        move_together(cx, &elsewhere, ZoneReason::Other);
    }
    if !to_battlefield.is_empty() {
        put_onto_battlefield(cx, &to_battlefield, ZoneReason::Other)?;
    }
    Ok(true)
}

pub fn move_to(cx: &mut Cx, id: ObjectId, to: Zone) -> ObjectId {
    move_to_for(cx, id, to, ZoneReason::Other)
}

pub fn move_to_for(cx: &mut Cx, id: ObjectId, to: Zone, reason: ZoneReason) -> ObjectId {
    move_together(cx, &[Moving::to(id, to)], reason)[0]
}

/// The entering replacements an object about to enter has of its own (CR 614.12), applied to `entering`.
pub fn prepare_entry(cx: &mut Cx, entering: &mut Entering) -> R<()> {
    // CR 708.3: a face-down permanent has no abilities, so none of its card's "enters" replacements apply.
    if cx.s().obj(entering.object).copy.is_some_and(|c| c.card == cx.cards().face_down) {
        return Ok(());
    }
    // CR 702.145b: at night a daybound card enters transformed.
    if crate::day_night::enters_transformed(cx, entering.object) {
        entering.transformed = true;
    }
    choose_protector(cx, entering)?;
    let card = cx.cards().face(cx.s().obj(entering.object).card, entering.transformed);
    if let Some(enters) = cx.cards().get(card).enters {
        enters(cx, entering)?;
    }
    // CR 707.9a: what it entered as a copy of has its own "enters with" and "enters tapped" to apply, as the copy
    // enters; a copy of a kicked permanent was never kicked itself (CR 702.33d).
    if let Some(copy) = entering.copy
        && copy.card != card
        && let Some(enters) = cx.cards().get(copy.card).enters
    {
        enters(cx, entering)?;
    }
    Ok(())
}

/// Who may protect a battle `controller` controls (CR 310.9a, 310.12a), in turn order: for a Siege each opponent still in
/// the game, for a battle of no type its controller alone.
pub fn protector_candidates(cx: &Cx, siege: bool, controller: PlayerId) -> Vec<PlayerId> {
    if siege {
        cx.s().opponents(controller).collect()
    } else if cx.s().player(controller).lost {
        Vec::new()
    } else {
        vec![controller]
    }
}

/// CR 310.9a, 310.12a: a battle's controller chooses its protector as it enters, out of the players its battle type
/// allows, asked only when there is more than one. A choice, not a target. A battle nobody can protect enters with
/// none, and CR 704.5x puts it into its owner's graveyard.
fn choose_protector(cx: &mut Cx, entering: &mut Entering) -> R<()> {
    let card = entering.copy.map_or_else(|| cx.cards().face(cx.s().obj(entering.object).card, entering.transformed), |c| c.card);
    let def = cx.cards().get(card);
    if !def.is(CardType::Battle) {
        return Ok(());
    }
    let candidates = protector_candidates(cx, def.subtypes.has(Subtype::Siege), entering.controller);
    entering.protector = match candidates.len() {
        0 => None,
        1 => Some(candidates[0]),
        _ => {
            let ask = RequestKind::ChoosePlayer { source: entering.object, candidates: candidates.clone(), verb: "Protect", amount: 0 };
            match cx.ask_core(entering.controller, ask, false)? {
                Answer::Player(p) if candidates.contains(&p) => Some(p),
                _ => Some(candidates[0]),
            }
        }
    };
    Ok(())
}

/// Puts objects onto the battlefield (CR 110.2a): each one's entering replacements are applied, then they all enter
/// at once. An Aura put onto the battlefield without being cast chooses what it enchants as it enters (CR 303.4f),
/// and one with nothing it could enchant stays where it is (CR 303.4g).
pub fn put_onto_battlefield(cx: &mut Cx, objects: &[(ObjectId, PlayerId, bool)], reason: ZoneReason) -> R<Vec<ObjectId>> {
    put_onto_battlefield_with(cx, objects, reason, &[])
}

/// The counters an effect says a permanent enters with (CR 122.6, 614.1c): "return it with a -1/-1 counter on it".
pub fn entry_counters(with: &[(CounterKind, i16)]) -> [i16; CounterKind::COUNT] {
    let mut counters = [0; CounterKind::COUNT];
    for &(kind, n) in with {
        counters[kind as usize] += n;
    }
    counters
}

/// `put_onto_battlefield`, each entering with these counters as well (CR 122.6).
pub fn put_onto_battlefield_with(cx: &mut Cx, objects: &[(ObjectId, PlayerId, bool)], reason: ZoneReason, with: &[(CounterKind, i16)]) -> R<Vec<ObjectId>> {
    put_onto_battlefield_along(cx, Vec::new(), objects, reason, with)
}

/// `put_onto_battlefield_with`, in one move with these others (CR 101.4): they leave or arrive at the same moment, so what
/// watches either sees the board once, as it was before and after all of it.
pub fn put_onto_battlefield_along(
    cx: &mut Cx,
    mut moves: Vec<Moving>,
    objects: &[(ObjectId, PlayerId, bool)],
    reason: ZoneReason,
    with: &[(CounterKind, i16)],
) -> R<Vec<ObjectId>> {

    for &(id, controller, tapped) in objects {
        // A card that has changed zones since it was chosen is a new object, not the one to put onto the battlefield
        // (CR 400.7), as for the other moves of a chosen object.
        if !live(cx.s(), id) {
            continue;
        }
        let mut entering =
            Entering { object: id, controller, tapped, counters: entry_counters(with), copy: None, prepared: false, chosen_color: None, chosen_subtype: None, chosen_player: None, chosen_card: None, chosen_odd: None, attach: None, attach_player: None, x: 0 , transformed: false, protector: None, instead: None };
        prepare_entry(cx, &mut entering)?;
        moves.push(Moving { id, to: Zone::BATTLEFIELD, entering: Some(entering), reason: None });
    }
    crate::attach::choose_aura_hosts(cx, &mut moves)?;
    Ok(move_together(cx, &moves, reason))
}

/// Sacrifices permanents and returns cards to the battlefield at the same moment (CR 101.4, 701.21): Goblin Welder's "simultaneously
/// sacrifices the artifact and returns the artifact card". One move, so the returning permanent's entering abilities see the
/// sacrificed one already gone, and the sacrificed one's leaving abilities see its replacement already there; each entering
/// replacement is applied as for any entry (CR 614.12). The sacrificed are moved for the reason `Sacrifice`.
pub fn sacrifice_and_return(cx: &mut Cx, sacrificed: &[ObjectId], returned: &[(ObjectId, PlayerId)]) -> R<()> {
    let moves: Vec<Moving> = sacrificed
        .iter()
        .copied()
        .filter(|&id| live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield))
        .map(|id| Moving { reason: Some(ZoneReason::Sacrifice), ..Moving::to(id, Zone::graveyard(cx.s().obj(id).owner)) })
        .collect();
    let arriving: Vec<(ObjectId, PlayerId, bool)> = returned.iter().map(|&(id, p)| (id, p, false)).collect();
    put_onto_battlefield_along(cx, moves, &arriving, ZoneReason::Other, &[])?;
    Ok(())
}

/// Puts one object onto the battlefield, with its back face up if it enters transformed (CR 712.14a).
pub fn put_onto_battlefield_as(
    cx: &mut Cx,
    id: ObjectId,
    controller: PlayerId,
    tapped: bool,
    transformed: bool,
    reason: ZoneReason,
) -> R<ObjectId> {
    // Only a double-faced card has a back face to enter with: any other stays where it is (CR 712.14a).
    if transformed && cx.cards().backs[cx.s().obj(id).card.0 as usize].is_none() {
        return Ok(id);
    }
    let mut entering = Entering {
        object: id,
        controller,
        tapped,
        counters: [0; CounterKind::COUNT],
        copy: None,
        prepared: false,
        chosen_color: None,
        chosen_subtype: None,
        chosen_player: None,
        chosen_card: None,
        chosen_odd: None,
        attach: None, attach_player: None,
        x: 0,
        transformed,
        protector: None,
        instead: None,
    };
    prepare_entry(cx, &mut entering)?;
    Ok(move_together(cx, &[Moving { id, to: Zone::BATTLEFIELD, entering: Some(entering), reason: None }], reason)[0])
}

/// What a permanent enters with because of what it is: a planeswalker its printed loyalty (CR 306.5b), a Saga a lore
/// counter (CR 714.3a), announced with the rest of the counters it enters with once it is there.
fn enters_with_intrinsic_counters(cards: &CardTable, new: &mut Obj) {
    let card = new.copy.map(|c| c.card).unwrap_or_else(|| cards.face(new.card, new.transformed));
    let def = cards.get(card);
    if let Some(loyalty) = def.loyalty {
        new.counters[CounterKind::Loyalty as usize] += loyalty;
    }
    // A battle its printed defense (CR 310.4b).
    if let Some(defense) = def.defense {
        new.counters[CounterKind::Defense as usize] += defense;
    }
    let saga = def.subtypes.has(Subtype::Saga) && def.abilities.iter().any(|a| matches!(a, AbilityDef::Triggered(t) if t.chapter > 0));
    if saga {
        new.counters[CounterKind::Lore as usize] += 1;
    }
}

/// Transforms a permanent (CR 701.28): its other face is up. The same object, so nothing about it is new (CR 701.28f).
/// Only a double-faced card on the battlefield transforms; a copy of one does not (CR 701.28c).
///
/// A permanent with daybound or nightbound "can't transform except due to its daybound (nightbound) ability"
/// (CR 702.145b, 702.145e): day and night turn it over (`turn_over`), no effect does.
pub fn transform(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || crate::day_night::role(cx.chars(id).keywords).is_some() {
        return;
    }
    turn_over(cx, id);
}

/// Turns a permanent over (CR 701.28), once whatever is doing it has established that it may.
pub fn turn_over(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) {
        return;
    }
    let obj = cx.s().obj(id);
    if !obj.zone.is(ZoneKind::Battlefield) || obj.copy.is_some() || cx.cards().backs[obj.card.0 as usize].is_none() {
        return;
    }
    cx.st().update_obj(id, |o| o.transformed = !o.transformed);
    note_statics(cx, id);
    cx.emit(Event::Transformed { object: id });
}

/// Creates an emblem (CR 114.2): an object in the command zone, owned by the player, that nothing moves.
pub fn create_emblem(cx: &mut Cx, def: CardId, player: PlayerId) -> ObjectId {
    // The emblem's abilities are the creating text's (CR 114.2, 612.4): words that text changed are changed in them.
    let words = match cx.cause() {
        Cause::Spell(source) | Cause::Ability { source, .. } => cantrip_api::words(cx.s(), source),
        _ => Words::NONE,
    };
    let copy = (!words.is_printed()).then(|| Copiable { words, ..Copiable::of(def) });
    let turn = cx.s().turn.number;
    let timestamp = cx.st().timestamp();
    let id = cx.st().push_obj(Obj {
        card: def,
        owner: player,
        controller: player,
        zone: Zone::COMMAND,
        token: false,
        tapped: false,
        controlled_since: turn,
        damage: 0,
        deathtouched: false,
        counters: [0; CounterKind::COUNT],
        timestamp,
        copy,
        loyalty_turn: 0,
        became: None,
        exile_on_leave: false,
        kicked: 0,
        paid: 0,
        gift: None,
        spent: ManaPool::default(),
        cast_way: 0,
        class_level: 0,
        prepared: false,
        card_copy: false,
        prepared_from: None,
        attached_to: None,
        attached_player: None,
        entered_with: None,
        chosen_color: None,
        chosen_subtype: None,
        on_adventure: false,
        cast_from: None,
        activated_turn: 0,
        activated_mask: 0,
        x: 0,
        marks: Marks::default(),
        transformed: false,
        exiled: None,
        face_down: false,
        seen_by: 0,
    });
    cx.st().zone_push(Zone::COMMAND, id);
    note_statics(cx, id);
    id
}

/// Creates tokens that are copies of a permanent (CR 707.2, 111.1): the permanent's copiable values, its own copy
/// effect's exceptions included.
pub fn create_token_copies(cx: &mut Cx, values: Copiable, controller: PlayerId, count: u32) -> R<Vec<ObjectId>> {
    create_tokens_as(cx, values.card, controller, count, false, if values.has_exceptions() { Some(values) } else { None }, &[])
}

/// Creates tokens (CR 111.2): each a new object on the battlefield, from no zone.
/// The effects of resolved spells and abilities that change a permanent spell's characteristics go on applying to the
/// permanent it becomes (CR 400.7a): a spell whose text was changed is a permanent with that text. Only text changes
/// are carried, the one such effect a spell can have now.
pub fn carry_effects(cx: &mut Cx, spell: ObjectId, permanent: ObjectId) {
    let carries = |e: &cantrip_state::ContinuousEffect| e.affected == spell && matches!(e.kind, EffectKind::ChangeText { .. });
    let carried: Vec<cantrip_state::ContinuousEffect> = cx.s().effects.iter().filter(|e| carries(e)).copied().collect();
    if carried.is_empty() {
        return;
    }
    let keep: Vec<bool> = cx.s().effects.iter().map(|e| !carries(e)).collect();
    cx.st().effects_keep(&keep);
    for e in carried {
        cx.st().effect_push(cantrip_state::ContinuousEffect { affected: permanent, ..e });
    }
}

pub fn create_tokens(cx: &mut Cx, card: CardId, controller: PlayerId, count: u32, tapped: bool) -> R<Vec<ObjectId>> {
    create_tokens_as(cx, card, controller, count, tapped, None, &[])
}

fn create_tokens_as(
    cx: &mut Cx,
    card: CardId,
    controller: PlayerId,
    count: u32,
    tapped: bool,
    copy: Option<Copiable>,
    with: &[(CounterKind, i16)],
) -> R<Vec<ObjectId>> {
    let tokens: Vec<(CardId, PlayerId)> = (0..count).map(|_| (card, controller)).collect();
    create_tokens_batch(cx, &tokens, tapped, copy, with)
}

/// Creates tokens of several definitions as one instruction (CR 111.1, 101.4): all enter at once, and what watches
/// them sees one event (CR 603.2c).
pub fn create_tokens_batch(cx: &mut Cx, tokens: &[(CardId, PlayerId)], tapped: bool, copy: Option<Copiable>, with: &[(CounterKind, i16)]) -> R<Vec<ObjectId>> {
    // CR 612.4, 612.2a: a token is what the text that makes it says, so a changed text makes tokens of the colors and
    // creature types it names now.
    let words = match cx.cause() {
        Cause::Spell(source) | Cause::Ability { source, .. } if copy.is_none() => cantrip_api::words(cx.s(), source),
        _ => Words::NONE,
    };
    create_tokens_worded(cx, tokens, tapped, copy, with, words, None)
}

/// Creates tokens whose text has `words` (CR 612.4): `create_tokens_batch` with the words given, not the creating text's.
/// `spell` is the copy of a permanent spell, as it resolves, the token is the permanent of (CR 707.10f), if it is: such a token is not
/// created (CR 111.13) but enters from the stack as the spell resolves, so its `ZoneChange` is `old: spell, from: Stack,
/// reason: Resolve`, as any permanent spell's is, and "if you cast it" (`cast_of(old)`, CR 707.12, 603.4) finds the copy's cast.
/// An Aura spell's token enters attached to what the spell targeted, wherever that is (CR 303.4f, 608.3b, 707.10c): a
/// creature card in a graveyard for Animate Dead (CR 303.4a), as the Aura card itself would.
pub fn create_tokens_worded(
    cx: &mut Cx,
    tokens: &[(CardId, PlayerId)],
    tapped: bool,
    copy: Option<Copiable>,
    with: &[(CounterKind, i16)],
    words: Words,
    spell: Option<&StackItem>,
) -> R<Vec<ObjectId>> {
    // CR 614.1a: how many of each there are is decided by what would replace the creation, before any of them is made. The
    // token a copy of a permanent spell becomes as it resolves is not created (CR 111.13, 707.10f), so nothing replaces it.
    let replaced = if spell.is_some() { None } else { replace::token_batch(cx, tokens) };
    let tokens = replaced.as_deref().unwrap_or(tokens);
    let before: Vec<ObjectId> = cx.s().battlefield().to_vec();
    let mut events = Vec::new();
    let mut created = Vec::new();
    for &(card, controller) in tokens {
        let turn = cx.s().turn.number;
        let timestamp = cx.st().timestamp();
        let id = cx.st().push_obj(Obj {
            card,
            owner: controller,
            controller,
            zone: Zone::GONE,
            token: true,
            tapped: false,
            controlled_since: turn,
            damage: 0,
            deathtouched: false,
            counters: [0; CounterKind::COUNT],
            timestamp,
            copy: None,
            loyalty_turn: 0,
            became: None,
            exile_on_leave: false,
            kicked: 0,
            paid: 0,
            // The token a copy of a permanent spell becomes was promised the gift the spell was (CR 707.10, ruling 2024-07-26).
            gift: spell.and_then(|s| s.gift),
            spent: ManaPool::default(),
            cast_way: 0,
            class_level: 0,
            prepared: false,
            card_copy: false,
            prepared_from: None,
            attached_to: None,
            attached_player: None,
            entered_with: None,
            chosen_color: None,
            chosen_subtype: None,
            on_adventure: false,
            cast_from: None,
            activated_turn: 0,
            activated_mask: 0,
            x: 0,
            marks: Marks::default(),
            transformed: false,
            exiled: None,
            face_down: false,
            seen_by: 0,
        });
        let copy = if words.is_printed() { copy } else { Some(Copiable { words, ..Copiable::of(card) }) };
        let aura = spell.filter(|_| cx.card(card).enchant.is_some());
        let mut entering = Entering { object: id, controller, tapped, counters: entry_counters(with), copy, prepared: false, chosen_color: None, chosen_subtype: None, chosen_player: None, chosen_card: None, chosen_odd: None, attach: aura.and_then(|i| i.targets[0].object()), attach_player: aura.and_then(|i| i.targets[0].player()), x: 0 , transformed: false, protector: None, instead: None };
        prepare_entry(cx, &mut entering)?;
        // The token is judged as it would exist on the battlefield (CR 614.12): a token copy as what it copies.
        let copy = entering.copy;
        cx.st().update_obj(id, |o| o.copy = copy);
        // A token enters the battlefield as it is created (CR 111.1): the replacements of a permanent's entry apply to it
        // as to any other's (CR 614.1c), "enters tapped" and the counters one gives as it arrives.
        let change = replace::zone_change(
            cx,
            ZoneChange {
                object: id,
                from: if spell.is_some() { Zone::STACK } else { Zone::GONE },
                to: Zone::BATTLEFIELD,
                reason: if spell.is_some() { ZoneReason::Resolve } else { ZoneReason::Created },
                controller,
                tapped: entering.tapped,
                counters: [0; CounterKind::COUNT],
                reveal: false,
                shuffle: false,
                is_card: false,
                madness: false,
            },
        );
        // An effect that sends it elsewhere instead (CR 614.1a): a token anywhere but the battlefield ceases to exist
        // (CR 111.7), so it is never created. No implemented effect does this to a token yet.
        if !change.to.is(ZoneKind::Battlefield) {
            continue;
        }
        entering.tapped = change.tapped;
        for (n, with) in entering.counters.iter_mut().zip(change.counters) {
            *n += with;
        }
        entering.counters = replace::counters(cx, id, controller, true, entering.counters);
        cx.st().update_obj(id, |o| {
            o.zone = Zone::BATTLEFIELD;
            o.tapped = entering.tapped;
            o.counters = entering.counters;
            o.copy = entering.copy;
            // What the token chose as it entered (CR 614.12): a copy of Painter's Servant picks its own color.
            o.chosen_color = entering.chosen_color;
            o.chosen_subtype = entering.chosen_subtype;
            // An Aura enters attached to what it targeted (CR 303.4f, 608.3b).
            o.attached_to = entering.attach;
            o.attached_player = entering.attach_player;
        });
        // A token copy of a planeswalker or a Saga enters with its loyalty or lore counter too.
        let mut with = *cx.s().obj(id);
        enters_with_intrinsic_counters(cx.cards(), &mut with);
        cx.st().update_obj(id, |o| o.counters = with.counters);
        // Once every counter it enters with is on it, a hone counter among them (CR 122.1j).
        note_statics(cx, id);
        cx.st().zone_push(Zone::BATTLEFIELD, id);
        events.push(match spell {
            Some(spell) => Event::ZoneChange { old: spell.object, new: id, from: Zone::STACK, to: Zone::BATTLEFIELD, reason: ZoneReason::Resolve },
            None => Event::ZoneChange { old: id, new: id, from: Zone::GONE, to: Zone::BATTLEFIELD, reason: ZoneReason::Created },
        });
        counters_entered_with(&mut events, id, controller, &with.counters);
        created.push(id);
    }
    cx.emit_batch(&events, Some(&before));
    Ok(created)
}

/// Announces the counters a permanent entered with, as counters put on it (CR 122.6): a Saga's lore counter too, so its
/// first chapter triggers (CR 714.3a, 714.2b), and all of them, so "when the Nth ... counter is put" sees each it entered with.
fn counters_entered_with(events: &mut Vec<Event>, object: ObjectId, by: PlayerId, counters: &[i16; CounterKind::COUNT]) {
    for kind in CounterKind::ALL {
        let amount = counters[kind as usize];
        if amount > 0 {
            events.push(Event::CountersPut { object, kind, amount, total: amount, by: Some(by) });
        }
    }
}

/// Becomes prepared (CR 722.2): the permanent is marked, and a copy of its prepare spell is put into exile under its
/// controller, linked to it.
/// A copy of a card (CR 707.12) in the zone the card is in, owned and controlled by `controller`, who is to cast it: a
/// card_copy object, not a card, which ceases to exist anywhere but the stack it was cast onto and where it began.
pub fn copy_card(cx: &mut Cx, card: ObjectId, controller: PlayerId) -> ObjectId {
    let zone = cx.s().obj(card).zone;
    copy_card_into(cx, card, controller, zone)
}

/// A copy of an object (CR 707.12) created in `zone`, owned and controlled by `controller`: `copy_card` for a copy made
/// somewhere other than where the object is, or of one that has left its zone (paradigm's "create a copy of this object in
/// exile", CR 702.192a). Its copiable values are the card's, as it last existed (CR 608.2h).
pub fn copy_card_into(cx: &mut Cx, card: ObjectId, controller: PlayerId, zone: Zone) -> ObjectId {
    let def = cx.s().obj(card).card;
    let turn = cx.s().turn.number;
    let timestamp = cx.st().timestamp();
    let copy = cx.st().push_obj(Obj { card_copy: true, controlled_since: turn, timestamp, ..Obj::new(def, controller, zone) });
    cx.st().zone_push(zone, copy);
    copy
}

pub fn become_prepared(cx: &mut Cx, permanent: ObjectId) {
    if !live(cx.s(), permanent) || !cx.s().obj(permanent).zone.is(ZoneKind::Battlefield) || cx.s().obj(permanent).prepared {
        return;
    }
    let card = layers::chars(cx.cards(), cx.s(), permanent).card;
    let Some(spell) = cx.cards().get(card).prepare else { return };
    let spell = cx.cards().id_of(spell);
    let controller = cx.s().obj(permanent).controller;
    cx.st().update_obj(permanent, |o| o.prepared = true);
    let turn = cx.s().turn.number;
    let timestamp = cx.st().timestamp();
    let copy = cx.st().push_obj(Obj {
        card: spell,
        owner: controller,
        controller,
        zone: Zone::exile(controller),
        token: false,
        tapped: false,
        controlled_since: turn,
        damage: 0,
        deathtouched: false,
        counters: [0; CounterKind::COUNT],
        timestamp,
        copy: None,
        loyalty_turn: 0,
        became: None,
        exile_on_leave: false,
        kicked: 0,
        paid: 0,
        gift: None,
        spent: ManaPool::default(),
        cast_way: 0,
        class_level: 0,
        prepared: false,
        card_copy: true,
        prepared_from: Some(permanent),
        attached_to: None,
        attached_player: None,
        entered_with: None,
        chosen_color: None,
        chosen_subtype: None,
        on_adventure: false,
        cast_from: None,
        activated_turn: 0,
        activated_mask: 0,
        x: 0,
        marks: Marks::default(),
        transformed: false,
        exiled: None,
        face_down: false,
        seen_by: 0,
    });
    cx.st().zone_push(Zone::exile(controller), copy);
}

/// Copies the spell on the stack (CR 707.10): the copy is a new spell object under `copier`, put onto the stack above
/// everything with the original's targets, modes, X and kicks. It is not cast (CR 707.10a), so nothing that watches
/// casts sees it, and it ceases to exist when it leaves the stack. With `new_targets` the copier may choose new
/// targets (CR 707.10c), slot by slot, each offered every legal choice (the old target among them); a slot with
/// nothing else it could be is not asked, and an answer that is not legal leaves the old target. Every target of the
/// copy, kept or new, becomes a target of it. Nothing if the spell was never on the stack or `copier` has left the game.
/// A copy of a permanent spell becomes a token as it resolves (CR 707.10f): `resolve_copy_as_token`.
pub fn copy_spell(cx: &mut Cx, spell: ObjectId, copier: PlayerId, new_targets: bool) -> R<Option<ObjectId>> {
    // A spell that has left the stack is copied as it last was on it (CR 707.10, 608.2h): the copy keeps its targets,
    // X and modes.
    let mut item = match cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) {
        Some(at) => cx.s().stack[at],
        None => match cx.s().left_stack(spell) {
            Some(&left) => left,
            None => return Ok(None),
        },
    };
    if cx.s().player(copier).lost {
        return Ok(None);
    }
    let original = *cx.s().obj(spell);
    // An instant or sorcery, or a permanent spell (a copy of one becomes a token as it resolves, CR 707.10f).

    let turn = cx.s().turn.number;
    let timestamp = cx.st().timestamp();
    let copy = cx.st().push_obj(Obj {
        owner: copier,
        controller: copier,
        zone: Zone::STACK,
        timestamp,
        controlled_since: turn,
        // A copy was paid for by no one: no mana spent (CR 707.10), so none of what mana does to the spell it pays for
        // (CR 106.6: Cavern of Souls' "can't be countered", Arena of Glory's haste) is the copy's.
        spent: ManaPool::default(),
        marks: Marks { cant_be_countered: false, gains_haste: false, ..original.marks },
        card_copy: true,
        prepared_from: None,
        prepared: false,
        became: None,
        ..original
    });
    cx.st().zone_push(Zone::STACK, copy);
    item.object = copy;
    item.controller = copier;
    item.serial = 0;
    cx.st().stack_push(item);
    let top = cx.s().stack.len() - 1;

    if new_targets {
        item = retarget(cx, top, copy, spell, copier)?;
    }
    for &t in item.targets().iter().filter(|&&t| t != Target::None) {
        cx.emit(Event::BecameTarget { target: t, by: copy, controller: copier });
    }
    cx.emit(Event::Copied { copy, original: spell, player: copier });
    Ok(Some(copy))
}

/// "You may choose new targets" (CR 115.7d, 707.10c) for the item at `top` of the stack: each slot it was announced with
/// is offered every legal choice it has now, the one it has among them; a slot with nothing else it could be is not
/// asked, and an answer that is not legal leaves the old target. `object` is the spell whose slots they are and
/// `asked_about` what the question names. Returns the item as it now is.
fn retarget(cx: &mut Cx, top: usize, object: ObjectId, asked_about: ObjectId, chooser: PlayerId) -> R<StackItem> {
    let mut item = cx.s().stack[top];
    let obj = *cx.s().obj(object);
    // An instant's or sorcery's targets, or a permanent spell's: an Aura's is its enchant ability (CR 303.4a, 707.10c). Those
    // of the face it is on the stack as - an Adventure, an Omen, a half (CR 715.3b, 720.3b, 709.3b) - not of its card.
    let specs = cx.card(obj.copy.map_or(obj.card, |c| c.card)).spell_targets();
    let asker = Asker { source: object, controller: chooser };
    for (at, spec) in specs.iter().enumerate() {
        // The targets chosen for this slot, as `target_legal` reads them (`StackItem::spec_of`).
        let run = item.spec_targets(at);
        let current: Vec<Target> = item.targets[run.clone()].to_vec();
        // A slot that chose no target (CR 601.2c) has none to change (CR 115.7d).
        if current.iter().all(|&t| t == Target::None) {
            continue;
        }
        let candidates = crate::actions::target_candidates(cx, &asker, spec);
        if candidates.len() <= current.len() && candidates.iter().all(|c| current.contains(c)) {
            continue;
        }
        let ask = RequestKind::ChooseTarget { label: "", slot_player: None,
            source: asked_about,
            purpose: Purpose::Targets,
            candidates: candidates.clone(),
            optional: false,
            least: current.len() as u8,
            count: current.len() as u8,
            any_number: false,
        };
        if let Answer::Targets(chosen) = cx.ask_core(chooser, ask, false)?
            && chosen.len() == current.len()
            && chosen.iter().all(|t| candidates.contains(t))
        {
            item.targets[run].copy_from_slice(&chosen);
        }
    }
    cx.st().stack_set(top, item);
    Ok(item)
}

/// `chooser` may choose new targets for a spell on the stack (CR 115.7d): Sudden Substitution's "then the spell's controller
/// may choose new targets for it". Nothing for anything but a spell on the stack. Each new target becomes a target.
pub fn choose_new_targets(cx: &mut Cx, spell: ObjectId, chooser: PlayerId) -> R<()> {
    let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) else {
        return Ok(());
    };
    let before = cx.s().stack[at];
    let after = retarget(cx, at, spell, spell, chooser)?;
    for &t in after.targets().iter().filter(|&&t| t != Target::None) {
        if !before.targets().contains(&t) {
            cx.emit(Event::BecameTarget { target: t, by: spell, controller: after.controller });
        }
    }
    Ok(())
}

/// "Change the target of target spell or ability" (CR 115.7a), a spell or an ability on the stack: each target must be changed
/// to another legal target (its current one is never offered), and must be if it can be. If any target has no other legal
/// target, none changes (CR 115.7a, 115.7e). A target with exactly one other choice is changed without asking. Legality is the
/// spell's or ability's own (its source and controller); `chooser` is who answers. Each new target becomes a target.
pub fn change_target(cx: &mut Cx, of: Target, chooser: PlayerId) -> R<()> {
    let at = match of {
        Target::Object(o) => cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == o),
        Target::Ability(serial) => cx.s().stack.iter().position(|i| i.kind != StackKind::Spell && i.serial == serial),
        _ => None,
    };
    let Some(at) = at else { return Ok(()) };
    let item = cx.s().stack[at];
    let count = item.target_count as usize;
    let asker = Asker { source: item.object, controller: item.controller };
    // Nobody is asked anything when some target has nowhere else to go, since then nothing changes at all.
    let mut options: Vec<Vec<Target>> = Vec::with_capacity(count);
    for slot in 0..count {
        // A slot that chose no target has none to change.
        if item.targets[slot] == Target::None {
            options.push(Vec::new());
            continue;
        }
        let Some((spec, slot_player)) = crate::resolve::slot_spec(cx.cards(), cx.s(), &item, slot) else { return Ok(()) };
        let others: Vec<Target> =
            crate::actions::target_candidates_given(cx, &asker, &spec, &item.targets[..slot], Target::None, slot_player)
                .into_iter()
                .filter(|&t| t != item.targets[slot])
                .collect();
        if others.is_empty() {
            return Ok(());
        }
        options.push(others);
    }
    let mut changed = item;
    for (slot, others) in options.into_iter().enumerate() {
        if others.is_empty() {
            continue;
        }
        changed.targets[slot] = if others.len() == 1 {
            others[0]
        } else {
            let ask = RequestKind::ChooseTarget { label: "", slot_player: None,
                source: item.object,
                purpose: Purpose::Targets,
                candidates: others.clone(),
                optional: false,
                least: 1,
                count: 1,
                any_number: false,
            };
            match cx.ask_core(chooser, ask, false)? {
                Answer::Targets(chosen) if chosen.len() == 1 && others.contains(&chosen[0]) => chosen[0],
                _ => others[0],
            }
        };
    }
    cx.st().stack_set(at, changed);
    for &t in changed.targets().iter().filter(|&&t| t != Target::None) {
        cx.emit(Event::BecameTarget { target: t, by: item.object, controller: item.controller });
    }
    Ok(())
}

/// A spell changes controllers on the stack (CR 112.2): only who controls it moves, so it resolves for its new controller
/// and "you" in it means them. Nothing it was cast with changes.
pub fn change_spell_controller(cx: &mut Cx, spell: ObjectId, player: PlayerId) {
    let Some(at) = cx.s().stack.iter().position(|i| i.kind == StackKind::Spell && i.object == spell) else {
        return;
    };
    let mut item = cx.s().stack[at];
    if item.controller == player || cx.s().player(player).lost {
        return;
    }
    item.controller = player;
    cx.st().stack_set(at, item);
    cx.st().update_obj(spell, |o| o.controller = player);
}

/// A copy of a permanent spell resolves (CR 707.10f): the copy is put onto the battlefield as a token that is a copy of
/// the spell's permanent. An Aura token enters attached to what the spell targeted (CR 303.4f, 608.3b, 707.10c); the copy itself
/// ceases to exist on the stack (CR 707.10a).
pub fn resolve_copy_as_token(cx: &mut Cx, item: &StackItem, at: usize) -> R<()> {
    let spell = *cx.s().obj(item.object);
    cx.st().stack_remove(at);
    cx.st().zone_remove(Zone::STACK, item.object);
    cx.st().update_obj(item.object, |o| o.zone = Zone::GONE);
    // A copy of a Room spell keeps the choice of door and the characteristics of both (CR 709.5b); the token it becomes is a
    // Room that enters with that door unlocked (CR 709.5d, 707.10f).
    let door = if cx.cards().get(spell.card).is_room() { cast_door(spell.cast_way) } else { 0 };
    let values = if door != 0 { Copiable::of(spell.card) } else { spell.copy.unwrap_or_else(|| Copiable::of(spell.card)) };
    let words = if values.has_exceptions() { Words::NONE } else { cantrip_api::words(cx.s(), item.object) };
    let copy = if values.has_exceptions() { Some(values) } else { None };
    let tokens = create_tokens_worded(cx, &[(values.card, item.controller)], false, copy, &[], words, Some(item))?;
    for token in tokens.into_iter().filter(|_| door != 0) {
        unlock_door(cx, token, door, item.controller);
    }
    Ok(())
}

/// Draws a card (CR 121): from an empty library, nothing is drawn and the player will lose (CR 704.5b).
pub fn draw(cx: &mut Cx, player: PlayerId) {
    // CR 121.6: a replacement applies even with no cards in the library to draw.
    if replace::draw_replaced(cx, player) {
        return;
    }
    let Some(&top) = cx.s().zone(Zone::library(player)).first() else {
        cx.st().update_player(player, |p| p.drew_from_empty = true);
        return;
    };
    let card = move_to_for(cx, top, Zone::hand(player), ZoneReason::Draw);
    // Miracle (CR 702.94a): "you may reveal this card from your hand as you draw it" if it is the first card drawn this
    // turn, before anything else happens; only if it is revealed does the linked ability trigger. The window is the
    // drawing player's alone, so nobody else learns of a "no". The card has its own miracle and any a permanent gives
    // cards in its controller's hand (Lorehold, the Historian); with more than one, the player says which it is revealed
    // with, and only that one's linked ability triggers (CR 607.5).
    if cx.s().turn.number > 0 && cx.s().obj(card).zone == Zone::hand(player) {
        let miracles = cantrip_api::miracles(&*cx, card);
        if let Some(&first) = miracles.first()
            && cx.history().drawn(player) == 0
            && matches!(cx.ask_core(player, RequestKind::May { source: card, purpose: Purpose::RevealForMiracle(only_miracle(cx, card, &miracles)) }, false), Ok(Answer::Yes))
            && let Some(miracle) = which_miracle(cx, player, card, &miracles, first)
            && cx.s().obj(card).zone == Zone::hand(player)
        {
            cx.emit(Event::Revealed { player, card });
            let ability = AbilityRef::miracle(cx.s().obj(card).card);
            cx.st().trigger_push(PendingTrigger {
                source: card,
                ability,
                controller: player,
                subject: Target::None,
                data: miracle.data(),
                members: cantrip_state::Span::default(),
                count: None,
            });
        }
    }
    cx.emit(Event::Drew { player, card });
}

/// A card's miracle ability, named for the question whether to reveal it, when it has only one (CR 702.94a).
fn only_miracle(cx: &Cx, card: ObjectId, miracles: &[cantrip_api::Miracle]) -> Option<MiracleOf> {
    match miracles {
        [one] => Some(one.of(cx.s().obj(card).card)),
        _ => None,
    }
}

/// The miracle ability a card being revealed as it is drawn is revealed with (CR 702.94a, 607.5): its only one, or the one
/// its player chooses.
fn which_miracle(cx: &mut Cx, player: PlayerId, card: ObjectId, miracles: &[cantrip_api::Miracle], first: cantrip_api::Miracle) -> Option<cantrip_api::Miracle> {
    if miracles.len() == 1 {
        return Some(first);
    }
    let own = cx.s().obj(card).card;
    let ask = RequestKind::ChooseModes {
        source: card,
        modes: miracles.iter().map(|m| Label::Miracle(m.of(own))).collect(),
        eligible: (0..miracles.len() as u8).collect(),
        count: 1,
        at_most: 0,
        affordable: Vec::new(),
        additional_cost: false,
        repeat: false,
    };
    match cx.ask_core(player, ask, false) {
        Ok(Answer::Modes(chosen)) => chosen.first().and_then(|&i| miracles.get(i as usize)).copied(),
        _ => None,
    }
}

/// An instruction to draw `count` cards (CR 121.2): effects that change how many apply first (CR 121.2a), before any
/// of the individual draws, which are then made one at a time.
pub fn draw_cards(cx: &mut Cx, player: PlayerId, count: u32) {
    for _ in 0..replace::draw_count(cx, player, count) {
        draw(cx, player);
    }
}

pub fn discard(cx: &mut Cx, card: ObjectId) {
    discard_together(cx, &[card]);
}

/// Discards cards together (CR 701.9).
pub fn discard_together(cx: &mut Cx, cards: &[ObjectId]) -> Vec<ObjectId> {
    // CR 101.2: a card an opponent's spell or ability can't make its owner discard stays where it is.
    let causer = cx.effect_controller();
    let moves: Vec<Moving> = cards
        .iter()
        .filter(|&&c| causer.is_none_or(|by| cantrip_api::may_cause_discard_or_sacrifice(&*cx, by, cx.s().obj(c).owner, false)))
        .map(|&c| Moving::to(c, Zone::graveyard(cx.s().obj(c).owner)))
        .collect();
    let new = move_together(cx, &moves, ZoneReason::Discard);
    // One event for the whole discard, so that "whenever you discard one or more cards" sees it once (CR 603.2c).
    let events: Vec<Event> = new.iter().map(|&card| Event::Discarded { player: cx.s().obj(card).owner, card }).collect();
    cx.emit_batch(&events, None);
    new
}

/// Mills (CR 701.17): the top cards together, top first, so the second card milled ends up above the first.
pub fn mill(cx: &mut Cx, player: PlayerId, count: u32) -> Vec<ObjectId> {
    let top: Vec<ObjectId> = cx.s().zone(Zone::library(player)).iter().take(count as usize).copied().collect();
    let moves: Vec<Moving> = top.iter().map(|&c| Moving::to(c, Zone::graveyard(player))).collect();
    move_together(cx, &moves, ZoneReason::Mill)
        .into_iter()
        .filter(|&n| cx.s().obj(n).zone.is(ZoneKind::Graveyard) || cx.s().obj(n).zone.is(ZoneKind::Exile))
        .collect()
}

/// Deals damage (CR 120): marked on a creature, removed as loyalty from a planeswalker, lost as life by a player.
/// Protection prevents damage from sources of its colors (CR 702.16e).
pub fn damage(cx: &mut Cx, source: ObjectId, target: Target, amount: u16, combat: bool) -> u16 {
    match deal_damage(cx, source, target, amount, combat) {
        Some(event) => {
            cx.emit(event);
            match event {
                Event::Damage { amount, .. } => amount,
                _ => 0,
            }
        }
        None => 0,
    }
}

/// Deals damage to several recipients at once (CR 120.2, 603.2c): announced as one batch, as combat damage is. How much
/// each was dealt, in order.
pub fn damage_together(cx: &mut Cx, hits: &[(ObjectId, Target, u16)]) -> Vec<u16> {
    let mut batch = Vec::new();
    let mut events = Vec::with_capacity(hits.len());
    let mut dealt = Vec::with_capacity(hits.len());
    for &(source, target, amount) in hits {
        match deal_damage_with(cx, source, target, amount, false, &mut batch) {
            Some(event) => {
                dealt.push(match event {
                    Event::Damage { amount, .. } => amount,
                    _ => 0,
                });
                events.push(event);
            }
            None => dealt.push(0),
        }
    }
    cx.emit_batch(&events, None);
    dealt
}

/// Deals damage as `damage` does, but hands back the event for the caller to announce, so that damage dealt
/// simultaneously (combat damage, CR 510.2) can be announced as one batch (CR 603.2c): "whenever equipped creature
/// deals combat damage" triggers once however many recipients there are.
pub fn deal_damage(cx: &mut Cx, source: ObjectId, target: Target, amount: u16, combat: bool) -> Option<Event> {
    deal_damage_with(cx, source, target, amount, combat, &mut Vec::new())
}

/// `deal_damage` as one assignment of a simultaneous event: `batch` holds what the event has dealt to each permanent
/// so far, for an effect that heals the damage marked before the event (it must not heal the event's own).
pub fn deal_damage_with(
    cx: &mut Cx,
    source: ObjectId,
    target: Target,
    amount: u16,
    combat: bool,
    batch: &mut Vec<(ObjectId, u16)>,
) -> Option<Event> {
    if amount == 0 {
        return None;
    }
    let event = replace::damage(cx, DamageEvent { source, target, amount, combat, heal_marked: false });
    let (amount, heal_marked) = (event.amount, event.heal_marked);
    if amount == 0 {
        return None;
    }
    let source_chars = layers::chars(cx.cards(), cx.s(), source);
    match target {
        Target::None | Target::Ability(_) => None,
        Target::Player(p) => {
            // Protection from everything prevents all damage to the player (CR 702.16j, 702.16e), unless it can't be
            // prevented (CR 615.12).
            if cx.s().effects.iter().any(|e| e.kind == EffectKind::PlayerProtectionFromEverything && e.controller == p)
                && damage_can_be_prevented(cx, source, combat)
            {
                return None;
            }
            if source_chars.has(Keyword::Infect) {
                // Damage from a source with infect is dealt to a player as that many poison counters (CR 702.90b,
                // 120.3b).
                put_player_counters(cx, p, amount, 0);
            } else {
                change_life(cx, p, -(amount as i32));
            }
            lifelink(cx, source, &source_chars, amount);
            if combat {
                crate::designations::combat_damage(cx, source, p, amount);
            }
            Some(Event::Damage { source, target, amount, combat })
        }
        Target::Object(id) => {
            if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
                return None;
            }
            let target_chars = layers::chars(cx.cards(), cx.s(), id);
            // Protection prevents the damage (CR 702.16e) unless it can't be prevented (CR 615.12).
            let source_controller = cx.s().obj(source).controller;
            if target_chars.protected_from_by(&source_chars, source_controller) && damage_can_be_prevented(cx, source, combat) {
                return None;
            }
            let deathtouch = source_chars.has(Keyword::Deathtouch);
            if target_chars.is(CardType::Creature) {
                if source_chars.has(Keyword::Infect) {
                    // Damage from a source with infect is dealt to a creature as that many -1/-1 counters (CR 702.90c,
                    // 120.3d); it is still damage dealt, so deathtouch counts (CR 702.2b).
                    add_counters(cx, id, CounterKind::MinusOne, amount.min(i16::MAX as u16) as i16);
                    cx.st().update_obj(id, |o| o.deathtouched |= deathtouch);
                } else {
                    if heal_marked {
                        // CR 614.1a: the damage marked before this event is healed; this event's own stays.
                        let kept = batch.iter().filter(|(b, _)| *b == id).map(|(_, a)| *a).sum();
                        cx.st().update_obj(id, |o| o.damage = kept);
                    }
                    batch.push((id, amount));
                    cx.st().update_obj(id, |o| {
                        o.damage += amount;
                        o.deathtouched |= deathtouch;
                    });
                }
            }
            // CR 120.3h, 310.6: damage dealt to a battle removes that many defense counters.
            if target_chars.is(CardType::Battle) {
                let removed = amount.min(cx.s().obj(id).counters(CounterKind::Defense).max(0) as u16) as i16;
                if removed > 0 {
                    add_counters(cx, id, CounterKind::Defense, -removed);
                }
            }
            if target_chars.is(CardType::Planeswalker) {
                cx.st().update_obj(id, |o| {
                    let loyalty = &mut o.counters[CounterKind::Loyalty as usize];
                    *loyalty = (*loyalty - amount as i16).max(0);
                });
            }
            lifelink(cx, source, &source_chars, amount);
            Some(Event::Damage { source, target, amount, combat })
        }
    }
}

/// Whether a prevention effect may prevent the damage `source` is about to deal (CR 615.12): not in a turn an effect said
/// damage can't be prevented this turn (Bonecrusher Giant's Stomp), nor while a static ability says this damage can't be
/// (Sunspine Lynx's "Damage can't be prevented", Questing Beast's combat damage). Protection (CR 702.16e) is the only
/// prevention effect the engine has; any other must ask this too.
fn damage_can_be_prevented(cx: &Cx, source: ObjectId, combat: bool) -> bool {
    let turn = cx.s().turn;
    turn.no_prevention_turn != turn.number && !crate::static_rules::damage_cant_be_prevented(cx, source, combat)
}

/// Damage dealt by a source with lifelink also causes its controller to gain that much life (CR 702.15b), as part of
/// the damage event, not a trigger.
fn lifelink(cx: &mut Cx, source: ObjectId, chars: &Chars, amount: u16) {
    if chars.has(Keyword::Lifelink) {
        let controller = cx.s().obj(source).controller;
        change_life(cx, controller, amount as i32);
    }
}

/// Each opponent of `player` still in the game loses `amount` life (CR 119.3): the total lost.
pub fn each_opponent_loses_life(cx: &mut Cx, player: PlayerId, amount: i32) -> i32 {
    let opponents: Vec<PlayerId> = cx.s().opponents(player).collect();
    let mut total = 0;
    for p in opponents {
        let before = cx.s().player(p).life;
        change_life(cx, p, -amount);
        total += before - cx.s().player(p).life;
    }
    total
}

/// `player`'s life total goes up or down by `delta` (CR 119.3). Every gain and loss of life comes here: an effect's, a
/// cost's (CR 119.4), damage (CR 120.3a), lifelink (CR 702.15b) and a life total set to a number (CR 119.5).
pub fn change_life(cx: &mut Cx, player: PlayerId, delta: i32) {
    if delta == 0 {
        return;
    }
    // CR 119.7: a player who can't gain life gains none, and nothing happens: no life gain event, so nothing that
    // triggers on one (CR 119.9) triggers.
    if delta > 0 && crate::static_rules::cant_gain_life(&*cx, cx.cards(), player) {
        return;
    }
    cx.st().update_player(player, |p| p.life += delta);
    cx.emit(Event::LifeChange { player, delta });
}

/// The moves left of a destruction once the permanents a regeneration shield saves have regenerated instead
/// (CR 701.19a, 614.8): one shield is used up for each.
fn without_regenerated(cx: &mut Cx, moves: &[Moving]) -> Vec<Moving> {
    if !cx.s().effects.iter().any(|e| e.kind == EffectKind::RegenerationShield) {
        return moves.to_vec();
    }
    let ids: Vec<ObjectId> = moves.iter().map(|m| m.id).collect();
    let saved = regenerate_instead(cx, &ids);
    moves.iter().copied().filter(|m| !saved.contains(&m.id)).collect()
}

/// Regeneration (CR 701.19a): each permanent with a shield uses one up; it is tapped, all damage is removed from it and
/// it is removed from combat instead of being destroyed. The permanents that did.
pub fn regenerate_instead(cx: &mut Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    let mut saved = Vec::new();
    for &id in ids {
        if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
            continue;
        }
        let Some(at) = cx.s().effects.iter().position(|e| e.kind == EffectKind::RegenerationShield && e.affected == id) else {
            continue;
        };
        let keep: Vec<bool> = (0..cx.s().effects.len()).map(|i| i != at).collect();
        cx.st().effects_keep(&keep);
        cx.st().update_obj(id, |o| {
            o.damage = 0;
            o.deathtouched = false;
        });
        tap(cx, id);
        if cx.s().attackers.iter().any(|a| a.id == id) {
            let attackers = cx.s().attackers.iter().copied().filter(|a| a.id != id).collect();
            cx.st().set_attackers(attackers);
        }
        if cx.s().blocks.iter().any(|b| b.blocker == id || b.attacker == id) {
            let blocks = cx.s().blocks.iter().copied().filter(|b| b.blocker != id && b.attacker != id).collect();
            cx.st().set_blocks(blocks);
        }
        saved.push(id);
    }
    saved
}


/// Destroys permanents together (CR 701.8a): each that is still the object it was, on the battlefield and not indestructible
/// (CR 702.12b) goes to its owner's graveyard, all at once (CR 603.10a). With `Destroy` a regeneration shield replaces the
/// destruction (CR 701.19a); with `DestroyNoRegen` it does not (CR 701.19c). The new objects.
pub fn destroy_together(cx: &mut Cx, ids: &[ObjectId], reason: ZoneReason) -> Vec<ObjectId> {
    let moves: Vec<Moving> = ids
        .iter()
        .copied()
        .filter(|&id| live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield))
        .filter(|&id| !layers::chars(cx.cards(), cx.s(), id).has(Keyword::Indestructible))
        .map(|id| Moving::to(id, Zone::graveyard(cx.s().obj(id).owner)))
        .collect();
    move_together(cx, &moves, reason)
}

/// Sacrifices permanents (CR 701.21), together, in the order given.
pub fn sacrifice(cx: &mut Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    // CR 101.2: a permanent an opponent's spell or ability can't make its controller sacrifice stays.
    let causer = cx.effect_controller();
    let moves: Vec<Moving> = ids
        .iter()
        .copied()
        .filter(|&id| live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield))
        .filter(|&id| causer.is_none_or(|by| cantrip_api::may_cause_discard_or_sacrifice(&*cx, by, cx.s().obj(id).controller, true)))
        .map(|id| Moving::to(id, Zone::graveyard(cx.s().obj(id).owner)))
        .collect();
    move_together(cx, &moves, ZoneReason::Sacrifice)
}

/// The objects among `ids` that are still the objects they were, for a verb that moves them (CR 400.7).
fn live_ones(cx: &Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    ids.iter().copied().filter(|&id| live(cx.s(), id)).collect()
}

/// Puts objects together into their owners' graveyards (CR 400.3), in the order given.
pub fn put_into_graveyard(cx: &mut Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    let moves: Vec<Moving> = live_ones(cx, ids).into_iter().map(|id| Moving::to(id, Zone::graveyard(cx.s().obj(id).owner))).collect();
    move_together(cx, &moves, ZoneReason::Other)
}

/// Exiles objects together, each to its owner's exile zone (CR 406.2).
pub fn exile_all(cx: &mut Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    let moves: Vec<Moving> = live_ones(cx, ids).into_iter().map(|id| Moving::to(id, Zone::exile(cx.s().obj(id).owner))).collect();
    move_together(cx, &moves, ZoneReason::Exile)
}

/// A number below `bound` from the game's own generator (none is drawn when there is no choice).
pub fn random_below(cx: &mut Cx, bound: u32) -> u32 {
    if bound <= 1 { 0 } else { cx.st().random(bound) }
}

/// Puts cards of a player's library on its top in this order, the first on top, or on its bottom, the last lowest. The
/// cards are already in the library; nothing changes zones.
pub fn arrange_library(cx: &mut Cx, player: PlayerId, top: &[ObjectId], bottom: &[ObjectId]) {
    let library = Zone::library(player);
    for &id in top.iter().chain(bottom) {
        cx.st().zone_remove(library, id);
    }
    for (at, &id) in top.iter().enumerate() {
        cx.st().zone_insert(library, at, id);
    }
    for &id in bottom {
        cx.st().zone_push(library, id);
    }
}

/// Puts cards already in a player's library on its bottom in a random order (CR 401.4), the order drawn from the game's own
/// generator.
pub fn bottom_in_random_order(cx: &mut Cx, player: PlayerId, cards: &[ObjectId]) {
    let mut cards = cards.to_vec();
    for i in (1..cards.len()).rev() {
        let j = random_below(cx, i as u32 + 1) as usize;
        cards.swap(i, j);
    }
    arrange_library(cx, player, &[], &cards);
}

/// Puts objects into their owners' libraries together and arranges them there (CR 401.4).
pub fn put_into_library(cx: &mut Cx, ids: &[ObjectId], place: LibraryPlace) -> Vec<ObjectId> {
    let moves: Vec<Moving> = live_ones(cx, ids).into_iter().map(|id| Moving::to(id, Zone::library(cx.s().obj(id).owner))).collect();
    let new = move_together(cx, &moves, ZoneReason::Other);
    let arrived: Vec<ObjectId> = new.iter().copied().filter(|&n| cx.s().obj(n).zone.is(ZoneKind::Library)).collect();
    let mut owners: Vec<PlayerId> = Vec::new();
    for &n in &arrived {
        let owner = cx.s().obj(n).owner;
        if !owners.contains(&owner) {
            owners.push(owner);
        }
    }
    for owner in owners {
        let mine: Vec<ObjectId> = arrived.iter().copied().filter(|&n| cx.s().obj(n).owner == owner).collect();
        match place {
            LibraryPlace::Top => arrange_library(cx, owner, &mine, &[]),
            LibraryPlace::Bottom => arrange_library(cx, owner, &[], &mine),
            LibraryPlace::BottomInRandomOrder => bottom_in_random_order(cx, owner, &mine),
            LibraryPlace::ShuffledIn => crate::turn::shuffle(cx.st(), Zone::library(owner)),
            LibraryPlace::FromTop(depth) => {
                let library = Zone::library(owner);
                for &id in &mine {
                    cx.st().zone_remove(library, id);
                }
                for (k, &id) in mine.iter().enumerate() {
                    let at = (depth + k).min(cx.s().zone(library).len());
                    cx.st().zone_insert(library, at, id);
                }
            }
        }
    }
    new
}

/// Returns objects to their owners' hands together (CR 400.3, 101.4).
pub fn return_to_hand(cx: &mut Cx, ids: &[ObjectId]) -> Vec<ObjectId> {
    let moves: Vec<Moving> = live_ones(cx, ids).into_iter().map(|id| Moving::to(id, Zone::hand(cx.s().obj(id).owner))).collect();
    move_together(cx, &moves, ZoneReason::Other)
}

/// Puts counters on an object, or takes them off with a negative amount. Putting them is an event replacement effects
/// can change (CR 122.6, 614.1a): "that many plus one" instead.
pub fn add_counters(cx: &mut Cx, id: ObjectId, kind: CounterKind, amount: i16) {
    if !live(cx.s(), id) {
        return;
    }
    let mut counts = [0; CounterKind::COUNT];
    counts[kind as usize] = amount;
    if amount > 0 && cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        let controller = cx.s().obj(id).controller;
        counts = replace::counters(cx, id, controller, false, counts);
    }
    let defense = cx.s().obj(id).counters(CounterKind::Defense);
    cx.st().update_obj(id, |o| {
        for (n, add) in o.counters.iter_mut().zip(counts) {
            if add != 0 {
                *n = (*n + add).max(0);
            }
        }
    });
    // The layers look for hone counters from now on (CR 122.1j).
    if counts[CounterKind::Hone as usize] > 0 {
        cx.st().see_statics();
    }
    // CR 310.12b: when the last defense counter comes off a Siege, its defeat ability triggers.
    if kind == CounterKind::Defense && defense > 0 && cx.s().obj(id).counters(CounterKind::Defense) == 0 {
        siege_defeat_triggers(cx, id);
    }
    // The counters that were put are announced (CR 121.1, 122.6a), for a Saga's chapters (CR 714.2b) and for "whenever counters
    // are put on this", as many as the replacements left, with the player who put them.
    let put = counts[kind as usize];
    if put > 0 {
        let by = cx.actor();
        let total = cx.s().obj(id).counters(kind);
        cx.emit(Event::CountersPut { object: id, kind, amount: put, total, by });
    }
}

/// A Siege that has lost its last defense counter triggers its defeat ability (CR 310.12b).
fn siege_defeat_triggers(cx: &mut Cx, id: ObjectId) {
    if !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let chars = layers::chars(cx.cards(), cx.s(), id);
    if !chars.has_subtype(Subtype::Siege) {
        return;
    }
    let ability = AbilityRef { card: cx.cards().intrinsic, index: cantrip_api::intrinsic::SIEGE_DEFEAT };
    let controller = cx.s().obj(id).controller;
    cx.st().trigger_push(cantrip_state::PendingTrigger {
        source: id,
        ability,
        controller,
        subject: Target::None,
        data: [0; 2],
        members: cantrip_state::Span::default(),
        count: None,
    });
}

/// CR 714.3b: as the active player's precombat main phase begins, they put a lore counter on each Saga they control
/// that has chapter abilities. A turn-based action, so it does not use the stack.
pub fn add_lore_counters(cx: &mut Cx) {
    let active = cx.s().turn.active;
    for id in cx.s().battlefield().to_vec() {
        if cx.s().obj(id).controller != active {
            continue;
        }
        let chars = layers::chars(cx.cards(), cx.s(), id);
        if chars.has_subtype(Subtype::Saga)
            && chars.abilities().any(|a| matches!(cx.cards().ability(a), AbilityDef::Triggered(t) if t.chapter > 0))
        {
            add_counters(cx, id, CounterKind::Lore, 1);
        }
    }
}

/// Puts poison and energy counters on a player (CR 122.1, 122.6): an event replacement effects can change.
pub fn put_player_counters(cx: &mut Cx, player: PlayerId, poison: u16, energy: u16) {
    let (poison, energy) = replace::player_counters(cx, player, poison, energy);
    cx.st().update_player(player, |p| {
        p.poison = (p.poison as u32 + poison as u32).min(255) as u8;
        p.energy = (p.energy as u32 + energy as u32).min(255) as u8;
    });
}

/// A player gets energy counters, or pays them with a negative `delta` (CR 107.14, 122.1).
pub fn change_energy(cx: &mut Cx, player: PlayerId, delta: i32) {
    if delta > 0 {
        put_player_counters(cx, player, 0, delta as u16);
        return;
    }
    cx.st().update_player(player, |p| p.energy = (p.energy as i32 + delta).clamp(0, 255) as u8);
}

/// Proliferate (CR 701.34a): the permanents on the battlefield and the players in the game that have a counter are
/// offered, battlefield order first; any number are chosen, and each gets one more of every kind it had. What each had is
/// read before any counter goes on, so one put on by a replacement effect is not "already there" for a later one.
pub fn proliferate(cx: &mut Cx, player: PlayerId, source: ObjectId) -> R<()> {
    let mut candidates: Vec<Target> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| cx.s().obj(id).counters.iter().any(|&n| n > 0))
        .map(Target::Object)
        .collect();
    for p in cx.s().seats() {
        let pl = cx.s().player(p);
        if !pl.lost && (pl.poison > 0 || pl.energy > 0) {
            candidates.push(Target::Player(p));
        }
    }
    if candidates.is_empty() {
        return Ok(());
    }
    let ask = RequestKind::ChooseTarget { label: "", slot_player: None,
        source,
        purpose: Purpose::Targets,
        count: candidates.len().min(255) as u8,
        candidates,
        optional: true,
            least: 0,
        any_number: true,
    };
    let Answer::Targets(chosen) = cx.ask_core(player, ask, false)? else { return Ok(()) };
    // Which kinds each one has, read first: objects' counters, then a player's poison and energy.
    let plan: Vec<(Target, [i16; CounterKind::COUNT], bool, bool)> = chosen
        .iter()
        .map(|&t| match t {
            Target::Object(id) => (t, cx.s().obj(id).counters, false, false),
            Target::Player(p) => (t, [0; CounterKind::COUNT], cx.s().player(p).poison > 0, cx.s().player(p).energy > 0),
            _ => (t, [0; CounterKind::COUNT], false, false),
        })
        .collect();
    for (target, counters, poison, energy) in plan {
        match target {
            Target::Object(id) => {
                for kind in CounterKind::ALL {
                    if counters[kind as usize] > 0 && live(cx.s(), id) {
                        add_counters(cx, id, kind, 1);
                    }
                }
            }
            Target::Player(p) => {
                if poison {
                    put_player_counters(cx, p, 1, 0);
                }
                if energy {
                    change_energy(cx, p, 1);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn tap(cx: &mut Cx, id: ObjectId) {
    cx.st().update_obj(id, |o| o.tapped = true);
    cx.emit(Event::Tapped { permanent: id });
}

/// Marks the game as one where static abilities may apply, once an object whose card has any is made, or one that has
/// hone counters (CR 122.1j) (the layers' guard; `GameState::statics_seen`).
fn note_statics(cx: &mut Cx, id: ObjectId) {
    if cx.s().statics_seen {
        return;
    }
    let obj = cx.s().obj(id);
    let card = obj.copy.map(|c| c.card).unwrap_or_else(|| cx.cards().face(obj.card, obj.transformed));
    if cx.cards().statics[card.0 as usize] || obj.counters(CounterKind::Hone) > 0 {
        cx.st().see_statics();
    }
}

/// Untaps a permanent (CR 701.26a). A permanent with a stun counter on it that would become untapped has a stun
/// counter removed from it instead (CR 122.1d): one counter for each untap, whatever their number.
pub fn untap(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) || !cx.s().obj(id).tapped {
        return;
    }
    if cx.s().obj(id).counters(CounterKind::Stun) > 0 {
        cx.st().update_obj(id, |o| o.counters[CounterKind::Stun as usize] -= 1);
        return;
    }
    cx.st().update_obj(id, |o| o.tapped = false);
}

/// Exerts a permanent (CR 701.43a): it won't untap during its controller's next untap step.
pub fn exert(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let controller = cx.s().obj(id).controller;
    cx.st().update_obj(id, |o| o.marks.exerted_by = Some(controller));
}

/// Monstrosity N (CR 701.37a): a permanent that is not monstrous gets N +1/+1 counters and becomes monstrous, and what
/// watches for it triggers (CR 701.37c).
pub fn monstrosity(cx: &mut Cx, id: ObjectId, counters: i16) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) || cx.s().obj(id).marks.monstrous {
        return;
    }
    cx.st().update_obj(id, |o| o.marks.monstrous = true);
    add_counters(cx, id, CounterKind::PlusOne, counters);
    cx.emit(Event::Monstrous { permanent: id });
}

/// A permanent becomes saddled until end of turn (CR 702.171b).
pub fn saddle(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let turn = cx.s().turn.number;
    cx.st().update_obj(id, |o| o.marks.saddled_turn = turn);
}

/// Two creatures fight (CR 701.14a): each deals damage equal to its power to the other, at once, as one event (CR 120.2;
/// not combat damage, CR 701.14d). A creature that fights itself deals damage to itself equal to twice its power (CR
/// 701.14c). Nothing happens unless both are creatures on the battlefield (CR 701.14b).
pub fn fight(cx: &mut Cx, first: ObjectId, second: ObjectId) {
    let fights = |cx: &Cx, id: ObjectId| {
        live(cx.s(), id)
            && cx.s().obj(id).zone.is(ZoneKind::Battlefield)
            && layers::chars(cx.cards(), cx.s(), id).is(CardType::Creature)
    };
    if !fights(cx, first) || !fights(cx, second) {
        return;
    }
    let power = |cx: &Cx, id: ObjectId| layers::chars(cx.cards(), cx.s(), id).power.unwrap_or(0).max(0) as u16;
    let (a, b) = (power(cx, first), power(cx, second));
    if first == second {
        damage(cx, first, Target::Object(first), a.saturating_mul(2), false);
        return;
    }
    damage_together(cx, &[(first, Target::Object(second), a), (second, Target::Object(first), b)]);
}

/// Creates tokens tapped and attacking (CR 508.4): they enter together, then are attacking `defender` without having
/// been declared as attackers, so no "whenever attacks" ability sees them. Outside combat they are only tapped tokens.
pub fn create_tokens_attacking(cx: &mut Cx, tokens: &[(CardId, PlayerId)], defender: Target) -> R<Vec<ObjectId>> {
    let created = create_tokens_batch(cx, tokens, true, None, &[])?;
    attack_with_tokens(cx, &created, defender);
    Ok(created)
}

/// Makes created tokens attack `defender` (CR 508.4), when there is a combat to attack in.
pub fn attack_with_tokens(cx: &mut Cx, created: &[ObjectId], defender: Target) {
    if cx.s().turn.attacked {
        let mut attackers = cx.s().attackers.clone();
        for &id in created {
            if live(cx.s(), id) && cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
                attackers.push(Attacker { id, target: defender, blocked: false });
            }
        }
        cx.st().set_attackers(attackers);
    }
}

/// Puts a card onto the battlefield tapped and attacking `defender` (CR 508.4): ninjutsu's "put this card onto the
/// battlefield from your hand tapped and attacking". Outside combat it is only tapped.
pub fn put_onto_battlefield_attacking(cx: &mut Cx, object: ObjectId, controller: PlayerId, defender: Target) -> R<ObjectId> {
    let new = put_onto_battlefield(cx, &[(object, controller, true)], ZoneReason::Other)?.first().copied().unwrap_or(object);
    if cx.s().turn.attacked && live(cx.s(), new) && cx.s().obj(new).zone.is(ZoneKind::Battlefield) {
        let mut attackers = cx.s().attackers.clone();
        attackers.push(Attacker { id: new, target: defender, blocked: false });
        cx.st().set_attackers(attackers);
    }
    Ok(new)
}

/// Settles who controls a permanent from the control-changing effects that apply to it (CR 613.1b, layer 2): the one
/// with the latest timestamp, else whoever controlled it before any of them did. Control that is new is new for
/// summoning sickness (CR 302.6) and takes the permanent out of combat (CR 506.4).
pub fn refresh_control(cx: &mut Cx, id: ObjectId) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let obj = *cx.s().obj(id);
    let latest = cx
        .s()
        .effects
        .iter()
        .filter(|e| e.affected == id)
        .filter_map(|e| match e.kind {
            EffectKind::Control(p) => Some((e.timestamp, p)),
            _ => None,
        })
        .max_by_key(|&(timestamp, _)| timestamp)
        .map(|(_, p)| p);
    let base = obj.marks.control_base.unwrap_or(obj.controller);
    let controller = latest.unwrap_or(base);
    let control_base = latest.map(|_| base);
    if controller == obj.controller && control_base == obj.marks.control_base {
        return;
    }
    let turn = cx.s().turn.number;
    cx.st().update_obj(id, |o| {
        o.marks.control_base = control_base;
        if o.controller != controller {
            o.controller = controller;
            o.controlled_since = turn;
        }
    });
    if controller != obj.controller {
        if cx.s().attackers.iter().any(|a| a.id == id) {
            let attackers = cx.s().attackers.iter().copied().filter(|a| a.id != id).collect();
            cx.st().set_attackers(attackers);
        }
        if cx.s().blocks.iter().any(|b| b.blocker == id || b.attacker == id) {
            let blocks = cx.s().blocks.iter().copied().filter(|b| b.blocker != id && b.attacker != id).collect();
            cx.st().set_blocks(blocks);
        }
    }
}

/// A permanent becomes a copy of something until end of turn (CR 707.2, 611.2b): its copiable values are `values` (exceptions
/// the effect states included) from now on, and are what they were again when the turn ends. It stays the same object, with
/// its counters, damage, status and attachments (CR 707.2). Nothing if it is not on the battlefield.
pub fn become_copy(cx: &mut Cx, source: ObjectId, id: ObjectId, values: Copiable) {
    if !live(cx.s(), id) || !cx.s().obj(id).zone.is(ZoneKind::Battlefield) {
        return;
    }
    let prior = cx.s().obj(id).copy;
    cx.st().update_obj(id, |o| o.copy = Some(values));
    let controller = cx.s().obj(source).controller;
    let timestamp = cx.st().timestamp();
    cx.st().effect_push(cantrip_state::ContinuousEffect {
        source,
        controller,
        timestamp,
        affected: id,
        kind: EffectKind::RestoreCopy(prior),
        duration: Duration::EndOfTurn,
    });
    note_statics(cx, id);
}

/// An effect states that a player loses the game (CR 104.3e): they leave it at once, and if one player is left, that player
/// has won (CR 104.2a).
pub fn lose_game(cx: &mut Cx, player: PlayerId) {
    if cx.s().player(player).lost || cx.s().is_over() {
        return;
    }
    cx.st().update_player(player, |pl| pl.lost = true);
    crate::designations::monarch_left(cx);
    leave_game(cx, player);
    let left: Vec<PlayerId> = cx.s().apnap().collect();
    match left.len() {
        0 => cx.st().set_outcome(Outcome::Draw),
        1 => cx.st().set_outcome(Outcome::Won(left[0])),
        _ => {}
    }
}

/// A player gets poison counters (CR 122.1f); ten lose the game as a state-based action (CR 704.5c).
pub fn give_poison(cx: &mut Cx, player: PlayerId, amount: u32) {
    put_player_counters(cx, player, amount.min(u16::MAX as u32) as u16, 0);
}

/// An effect states that a player wins the game (CR 104.2b): every opponent has lost, which ends the game with them
/// the winner (CR 104.2a). A player who has lost wins nothing; one who is about to lose to a state-based action
/// (CR 704.5a-c) both wins and loses at once, and loses (CR 104.3f).
pub fn win_game(cx: &mut Cx, player: PlayerId) {
    let p = *cx.s().player(player);
    if p.lost || p.life <= 0 || p.drew_from_empty || p.poison >= 10 || cx.s().is_over() {
        return;
    }
    for o in cx.s().opponents(player).collect::<Vec<_>>() {
        if !cx.s().player(o).lost {
            cx.st().update_player(o, |pl| pl.lost = true);
            crate::designations::monarch_left(cx);
        }
    }
    cx.st().set_outcome(Outcome::Won(player));
}

/// A player has left a game that goes on without them (CR 800.4a): the spells and abilities they control, and every object
/// they own, leave the game, and what they control but another owns is exiled. The last player standing wins at once
/// instead (CR 104.2a), and there is nothing left to clear.
pub fn leave_game(cx: &mut Cx, player: PlayerId) {
    if cx.s().apnap().count() < 2 {
        return;
    }
    // On the stack: a card they own leaves with them, an object not represented by a card they control (an ability, a
    // copy) ceases to exist, and a card they control but another owns is exiled with the other permanents below.
    let mut exiled: Vec<ObjectId> = Vec::new();
    for at in (0..cx.s().stack.len()).rev() {
        let item = cx.s().stack[at];
        if item.kind != StackKind::Spell {
            if item.controller == player {
                cx.st().stack_remove(at);
            }
            continue;
        }
        let obj = *cx.s().obj(item.object);
        if obj.owner == player || (item.controller == player && obj.card_copy) {
            cx.st().stack_remove(at);
            cx.st().zone_remove(Zone::STACK, item.object);
            cx.st().update_obj(item.object, |o| o.zone = Zone::GONE);
        } else if item.controller == player {
            cx.st().stack_remove(at);
            exiled.push(item.object);
        }
    }
    // Triggers they control that have not yet been put on the stack have nothing to go on it.
    for at in (0..cx.s().triggers.len()).rev() {
        if cx.s().triggers[at].controller == player {
            cx.st().trigger_remove(at);
        }
    }
    for at in (0..cx.s().delayed.len()).rev() {
        if cx.s().delayed[at].controller == player {
            cx.st().delayed_remove(at);
        }
    }
    // Everything they own, wherever it is.
    let owned: Vec<(Zone, ObjectId)> = cx
        .s()
        .zones
        .iter()
        .flatten()
        .copied()
        .filter(|&id| cx.s().obj(id).owner == player)
        .map(|id| (cx.s().obj(id).zone, id))
        .collect();
    for (zone, id) in owned {
        cx.st().zone_remove(zone, id);
        cx.st().update_obj(id, |o| o.zone = Zone::GONE);
    }
    let gone = |cx: &Cx, id: ObjectId| cx.s().obj(id).zone.is(ZoneKind::Gone);
    if cx.s().attackers.iter().any(|a| gone(cx, a.id)) {
        let attackers = cx.s().attackers.iter().copied().filter(|a| !gone(cx, a.id)).collect();
        cx.st().set_attackers(attackers);
    }
    if cx.s().blocks.iter().any(|b| gone(cx, b.blocker) || gone(cx, b.attacker)) {
        let blocks = cx.s().blocks.iter().copied().filter(|b| !gone(cx, b.blocker) && !gone(cx, b.attacker)).collect();
        cx.st().set_blocks(blocks);
    }
    // What they still control is exiled (CR 800.4a).
    exiled.extend(cx.s().battlefield().iter().copied().filter(|&id| cx.s().obj(id).controller == player));
    let moves: Vec<Moving> = exiled.into_iter().map(|id| Moving::to(id, Zone::exile(cx.s().obj(id).owner))).collect();
    if !moves.is_empty() {
        move_together(cx, &moves, ZoneReason::Exile);
    }
}
