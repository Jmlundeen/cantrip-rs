//! Replacement effects (CR 614, 616): finding the ones that apply to a proposed event, having the affected player
//! order them when there are several, and applying them one at a time until none is left.
//!
//! The effects belong to cards (`CardDef::replacements`): the core knows no card. An event that nothing on the board
//! could replace costs one scan of the battlefield for a flag per card, and nothing is built.

use cantrip_api::*;

use crate::layers::live;
use crate::Cx;

/// One effect that applies to the event: its source, and which of the source's replacements it is.
#[derive(Clone, Copy)]
struct Candidate {
    source: ObjectId,
    index: u8,
    /// For a replacement effect a spell or ability made (CR 611.2, `EffectKind::Replacement`): the timestamp of that
    /// effect, which names it. `index` is then one of its source card's `granted_replacements`.
    effect: Option<u32>,
}

/// An effect already applied to the event (CR 614.5): its source, its index, and the effect that made it if it has one.
type Applied = (ObjectId, u8, Option<u32>);

/// The card whose definition a permanent's abilities come from: the card it copies, if it copies one.
fn card_of(cx: &Cx, id: ObjectId) -> CardId {
    let obj = cx.s().obj(id);
    obj.copy.map_or(obj.card, |c| c.card)
}

/// Whether any effect could apply to an event about `subject`: a permanent with replacements, or the subject's own
/// card, which has some that function from every zone. The guard that keeps the hot path free of replacement work.
fn possible(cx: &Cx, subject: Option<ObjectId>) -> bool {
    // An effect a spell or ability made, which lasts (CR 611.2).
    if cx.s().effects.iter().any(|e| matches!(e.kind, EffectKind::Replacement(_) | EffectKind::ReplacementRule(_))) {
        return true;
    }
    let cards = cx.cards();
    if !cards.any_replaces {
        return false;
    }
    subject.is_some_and(|id| cards.replaces[card_of(cx, id).0 as usize])
        || cx.s().battlefield().iter().any(|&id| cards.replaces[card_of(cx, id).0 as usize])
}

fn subject_of(proposal: &Proposal) -> Option<ObjectId> {
    match proposal {
        Proposal::ZoneChange(z) => Some(z.object),
        Proposal::Counters(c) => Some(c.object),
        Proposal::DrawInstruction(_) | Proposal::Draw(_) | Proposal::TokenCreation(_) | Proposal::PlayerCounters(_) => None,
        Proposal::Damage(d) => match d.target {
            Target::Object(id) => Some(id),
            _ => None,
        },
    }
}

/// "You" for an effect: its controller; for an effect of the very permanent entering, the player it enters under
/// (CR 110.2a, 614.12).
fn controller_of(cx: &Cx, source: ObjectId, proposal: &Proposal) -> PlayerId {
    match proposal {
        Proposal::ZoneChange(z) if z.object == source => z.controller,
        Proposal::Counters(c) if c.object == source && c.entering => c.controller,
        _ => cx.s().obj(source).controller,
    }
}

/// The player CR 616.1 gives the choice to: whose permanent it is (the player it enters under), or who would draw.
fn affected(cx: &Cx, proposal: &Proposal) -> PlayerId {
    match proposal {
        Proposal::Damage(d) => match d.target {
            Target::Player(p) => p,
            Target::Object(id) => cx.s().obj(id).controller,
            _ => cx.s().turn.active,
        },
        Proposal::ZoneChange(z) => z.controller,
        Proposal::Counters(c) => c.controller,
        Proposal::PlayerCounters(c) => c.player,
        Proposal::DrawInstruction(d) => d.player,
        Proposal::Draw(d) => d.player,
        Proposal::TokenCreation(t) => t.controller,
    }
}

/// Every effect that applies to the event and has not been applied to it yet (CR 614.5), in a deterministic order:
/// by source, then by the order the card lists them.
fn candidates(cx: &Cx, proposal: &Proposal, applied: &[Applied]) -> Vec<Candidate> {
    let cards = cx.cards();
    let kind = proposal.kind();
    let subject = subject_of(proposal);
    let mut found = Vec::new();
    let mut consider = |id: ObjectId, own_only: bool| {
        let card = card_of(cx, id);
        if !cards.replaces[card.0 as usize] {
            return;
        }
        let asker = Asker { source: id, controller: controller_of(cx, id, proposal) };
        for (index, def) in cards.get(card).replacements.iter().enumerate() {
            let index = index as u8;
            if def.kind != kind || (own_only && !def.own_everywhere) || applied.contains(&(id, index, None)) {
                continue;
            }
            if (def.applies)(cx, &asker, proposal) {
                found.push(Candidate { source: id, index, effect: None });
            }
        }
    };
    for &id in cx.s().battlefield() {
        if live(cx.s(), id) {
            consider(id, false);
        }
    }
    // The subject's own effects that function in every zone (CR 113.6b), when it is not on the battlefield already.
    if let Some(id) = subject
        && !cx.s().obj(id).zone.is(ZoneKind::Battlefield)
    {
        consider(id, true);
    }
    // The replacement effects spells and abilities made, still lasting (CR 611.2): one about an object ends with it.
    for e in &cx.s().effects {
        let (index, about_player) = match e.kind {
            EffectKind::Replacement(i) => (i, false),
            EffectKind::ReplacementRule(i) => (i, true),
            _ => continue,
        };
        if !about_player && !live(cx.s(), e.affected) {
            continue;
        }
        let def = &cards.get(card_of(cx, e.source)).granted_replacements[index as usize];
        if def.kind != kind || applied.contains(&(e.source, index, Some(e.timestamp))) {
            continue;
        }
        let asker = Asker { source: e.source, controller: e.controller };
        if (def.applies)(cx, &asker, e.affected, proposal) {
            found.push(Candidate { source: e.source, index, effect: Some(e.timestamp) });
        }
    }
    found.sort_by_key(|c| (c.source, c.index, c.effect));
    found
}

/// What an effect says it does, for the player who orders several (CR 616.1).
fn hint_of(cx: &Cx, c: &Candidate) -> &'static str {
    let card = cx.cards().get(card_of(cx, c.source));
    match c.effect {
        None => card.replacements[c.index as usize].hint,
        Some(_) => card.granted_replacements[c.index as usize].hint,
    }
}

/// Asks the affected player which of several effects applies first (CR 616.1). When the question has no answer yet
/// the run is abandoned and replayed with it (see `Cx::ask_core`), so the first is taken to carry on to the end.
fn choose(cx: &mut Cx, proposal: &Proposal, found: &[Candidate]) -> usize {
    let ask = RequestKind::ChooseReplacement {
        subject: subject_of(proposal),
        sources: found.iter().map(|c| c.source).collect(),
        hints: found.iter().map(|c| hint_of(cx, c)).collect(),
    };
    match cx.ask_core(affected(cx, proposal), ask, false) {
        Ok(Answer::Index(i)) if (i as usize) < found.len() => i as usize,
        _ => 0,
    }
}

/// Applies the effects to the proposal, one at a time: each is judged again after the one before it has changed the
/// event (CR 616.1f), and none applies twice (CR 614.5).
fn apply(cx: &mut Cx, proposal: &mut Proposal) {
    let mut applied: Vec<Applied> = Vec::new();
    loop {
        let found = candidates(cx, proposal, &applied);
        if found.is_empty() {
            return;
        }
        let pick = if found.len() > 1 { choose(cx, proposal, &found) } else { 0 };
        let Candidate { source, index, effect } = found[pick];
        applied.push((source, index, effect));
        match effect {
            None => {
                let def = &cx.cards().get(card_of(cx, source)).replacements[index as usize];
                let (apply, then) = (def.apply, def.then);
                let asker = Asker { source, controller: controller_of(cx, source, proposal) };
                let before = *proposal;
                apply(&*cx, &asker, proposal);
                if let Some(then) = then {
                    then(cx, &asker, &before, proposal);
                }
            }
            Some(timestamp) => {
                let e = *cx.s().effects.iter().find(|e| e.timestamp == timestamp).expect("the effect that applies");
                let def = &cx.cards().get(card_of(cx, source)).granted_replacements[index as usize];
                let asker = Asker { source, controller: e.controller };
                (def.apply)(&*cx, &asker, e.affected, proposal);
            }
        }
    }
}

/// An object about to change zones (CR 614.1a, 614.1c): what remains of the move once every effect has applied.
pub fn zone_change(cx: &mut Cx, change: ZoneChange) -> ZoneChange {
    if !possible(cx, Some(change.object)) {
        return change;
    }
    let mut proposal = Proposal::ZoneChange(change);
    apply(cx, &mut proposal);
    match proposal {
        Proposal::ZoneChange(change) => change,
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}

/// Counters that would be put on a permanent, or that one enters with (CR 122.6, 614.1a): how many of each kind
/// remain once every effect has applied. The permanent is judged as it is now, or as it would exist on the
/// battlefield if it is entering (CR 614.12).
pub fn counters(cx: &mut Cx, object: ObjectId, controller: PlayerId, entering: bool, counts: [i16; CounterKind::COUNT]) -> [i16; CounterKind::COUNT] {
    if !possible(cx, Some(object)) || counts.iter().all(|&n| n <= 0) {
        return counts;
    }
    // CR 122.6a: counters an object enters with are put on it by its controller, unless the effect names another player.
    let putter = if entering { Some(controller) } else { cx.actor() };
    let mut proposal = Proposal::Counters(CounterPlacement { object, controller, entering, counts, putter });
    apply(cx, &mut proposal);
    match proposal {
        Proposal::Counters(placement) => placement.counts,
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}

/// Counters that would be put on a player (CR 122.6): how many poison and energy counters remain once every effect has
/// applied.
pub fn player_counters(cx: &mut Cx, player: PlayerId, poison: u16, energy: u16) -> (u16, u16) {
    if !possible(cx, None) || (poison == 0 && energy == 0) {
        return (poison, energy);
    }
    let putter = cx.actor();
    let mut proposal = Proposal::PlayerCounters(PlayerCounters { player, putter, poison, energy });
    apply(cx, &mut proposal);
    match proposal {
        Proposal::PlayerCounters(c) => (c.poison, c.energy),
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}

/// An instruction to draw cards (CR 121.2a): how many are drawn once every effect that changes the number has
/// applied, before any individual draw.
pub fn draw_count(cx: &mut Cx, player: PlayerId, count: u32) -> u32 {
    if !possible(cx, None) {
        return count;
    }
    let mut proposal = Proposal::DrawInstruction(DrawInstruction { player, count });
    apply(cx, &mut proposal);
    match proposal {
        Proposal::DrawInstruction(d) => d.count,
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}

/// One card about to be drawn (CR 121.1, 121.6): whether an effect has replaced the draw, so that nothing is drawn.
pub fn draw_replaced(cx: &mut Cx, player: PlayerId) -> bool {
    if !possible(cx, None) {
        return false;
    }
    let mut proposal = Proposal::Draw(CardDraw { player, replaced: false });
    apply(cx, &mut proposal);
    matches!(proposal, Proposal::Draw(CardDraw { replaced: true, .. }))
}

/// Tokens that would be created (CR 111.1, 614.1a): how many remain once every effect has applied. Several definitions
/// created as one instruction are asked about each on its own, under each player.
pub fn token_batch(cx: &mut Cx, tokens: &[(CardId, PlayerId)]) -> Option<Vec<(CardId, PlayerId)>> {
    if !possible(cx, None) {
        return None;
    }
    let mut out: Vec<(CardId, PlayerId)> = Vec::with_capacity(tokens.len());
    let mut changed = false;
    let mut i = 0;
    while i < tokens.len() {
        // The run of one definition under one player, in the order given.
        let (token, controller) = tokens[i];
        let mut count = 0u32;
        while i < tokens.len() && tokens[i] == (token, controller) {
            count += 1;
            i += 1;
        }
        let mut proposal = Proposal::TokenCreation(TokenCreation { controller, token, count });
        apply(cx, &mut proposal);
        let Proposal::TokenCreation(after) = proposal else { unreachable!("an effect cannot change the kind of the event") };
        changed |= after.count != count;
        out.extend(std::iter::repeat((token, controller)).take(after.count as usize));
    }
    changed.then_some(out)
}

/// Damage about to be dealt (CR 614.1a, 120.3): what remains of it once every effect has applied.
pub fn damage(cx: &mut Cx, event: DamageEvent) -> DamageEvent {
    if !possible(cx, None) {
        return event;
    }
    let mut proposal = Proposal::Damage(event);
    apply(cx, &mut proposal);
    match proposal {
        Proposal::Damage(event) => event,
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}
