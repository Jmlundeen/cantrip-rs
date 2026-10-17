//! Replacement effects (CR 614, 616): finding the ones that apply to a proposed event, having the affected player
//! order them when there are several, and applying them one at a time until none is left.
//!
//! The effects belong to cards (`CardDef::replacements`): the core knows no card. An event that nothing on the board
//! could replace costs one scan of the battlefield for a flag per card, and nothing is built. One keyword's replacement
//! is the core's own and is ordered with the cards' like any other: madness (CR 702.35a), of a card being discarded.
//!
//! Prevention effects (CR 615) are ordered with them (CR 616.1): a static ability's (`CardDef::preventions`), one a
//! resolved spell or ability made (`EffectKind::Prevention`) and protection's (CR 702.16e). Each that prevents damage
//! records it (`Event::DamagePrevented`, CR 615.6), and none applies to damage that can't be prevented (CR 615.12): such
//! a prevention effect would prevent nothing, and a shield is not reduced by it.

use cantrip_api::*;

use crate::layers::live;
use crate::Cx;

/// Where a replacement effect is defined.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Origin {
    /// One of its source card's `replacements`.
    Printed,
    /// One a spell or ability made (CR 611.2, `EffectKind::Replacement`), named by that effect's timestamp: one of its
    /// source card's `granted_replacements`.
    Granted(u32),
    /// Madness's static ability (CR 702.35a), the core's own keyword replacement, of the discarded card itself.
    Madness,
    /// Protection's prevention of damage (CR 702.16e, 702.16j), a prevention effect ordered with the replacement effects that
    /// would change the same damage (CR 616.1): of the protected permanent, or the source of a player's protection.
    Protection,
    /// One of its source card's `preventions`, a static ability's prevention effect (CR 615, 604).
    Prevention,
    /// A prevention effect a spell or ability made (CR 611.2, 615, `EffectKind::Prevention`), named by that effect's
    /// timestamp: one of its source card's `granted_preventions`.
    GrantedPrevention(u32),
    /// "Each land played this way enters tapped" (CR 305.1, 614.1c, `PlayGrant::enters_tapped`): the replacement the effect
    /// that gave a permission to play the land made, of the land played by it.
    PlayedTapped,
}

/// One effect that applies to the event: its source, and which of the source's replacements it is.
#[derive(Clone, Copy)]
struct Candidate {
    source: ObjectId,
    index: u8,
    origin: Origin,
}

/// An effect already applied to the event (CR 614.5): its source, its index and where it is defined.
type Applied = (ObjectId, u8, Origin);

/// What madness's replacement says it does, for the player who orders it with others (CR 616.1).
const MADNESS_HINT: &str = "madness discards it into exile, then it may be cast for its madness cost";

/// What protection's prevention says it does, for the player who orders it with others (CR 616.1).
const PROTECTION_HINT: &str = "protection prevents the damage";

/// What a permission's "enters tapped" says it does, for the player who orders it with others (CR 616.1).
const PLAYED_TAPPED_HINT: &str = "a land played this way enters tapped";

/// "Each land played this way enters tapped" (CR 305.1, 614.1c): a land played by a permission that says so
/// (`actions::land_play_permission`), entering untapped as the proposal stands. Judged before the land moves, while it is
/// still where it was played from and the permission is there to read.
fn played_tapped_applies(cx: &Cx, z: &ZoneChange) -> bool {
    z.reason == ZoneReason::PlayLand
        && z.to.is(ZoneKind::Battlefield)
        && !z.tapped
        && cx.s().permissions.iter().any(|p| p.card == z.object && p.grant.enters_tapped)
        && crate::actions::land_play_permission(cx, z.object, z.controller).is_some_and(|p| p.grant.enters_tapped)
}

/// Madness (CR 702.35a): "If a player would discard this card, that player discards it, but exiles it instead of putting
/// it into their graveyard." A static ability of the card itself, functioning while it is in a hand; once another
/// effect has sent the card elsewhere than the graveyard it no longer applies (CR 616.1f).
fn madness_applies(cx: &Cx, z: &ZoneChange) -> bool {
    z.reason == ZoneReason::Discard
        && z.from.is(ZoneKind::Hand)
        && z.to.is(ZoneKind::Graveyard)
        && cx.cards().get(card_of(cx, z.object)).madness.is_some()
        && !cx.lost_all_abilities(z.object)
}

/// The card whose definition a permanent's abilities come from: the card it copies, if it copies one, or the face that is up
/// (CR 712.8d-f: a double-faced permanent with its back face up has only that face's abilities).
fn card_of(cx: &Cx, id: ObjectId) -> CardId {
    let obj = cx.s().obj(id);
    obj.copy.map_or_else(|| cx.cards().face(obj.card, obj.transformed), |c| c.card)
}

/// Whether any effect could apply to an event about `subject`: a permanent with replacements, or the subject's own
/// card, which has some that function from every zone. The guard that keeps the hot path free of replacement work.
pub(crate) fn possible(cx: &Cx, subject: Option<ObjectId>) -> bool {
    // An effect a spell or ability made, which lasts (CR 611.2).
    if cx.s().effects.iter().any(|e| matches!(e.kind, EffectKind::Replacement(_) | EffectKind::ReplacementRule(_) | EffectKind::Prevention { .. })) {
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
        Proposal::DrawInstruction(_)
        | Proposal::Draw(_)
        | Proposal::TokenCreation(_)
        | Proposal::PlayerCounters(_)
        | Proposal::LifeChange(_) => None,
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
        Proposal::LifeChange(l) => l.player,
    }
}

/// Every effect that applies to the event and has not been applied to it yet (CR 614.5), in a deterministic order:
/// by source, then by the order the card lists them.
fn candidates(cx: &Cx, proposal: &Proposal, applied: &[Applied]) -> Vec<Candidate> {
    // Damage that has been prevented in full is no event any more: nothing is left to replace (CR 615.1).
    if let Proposal::Damage(d) = proposal
        && d.amount == 0
    {
        return Vec::new();
    }
    let cards = cx.cards();
    let kind = proposal.kind();
    let subject = subject_of(proposal);
    // Damage a prevention effect may prevent (CR 615.12): where it can't be, none of them applies.
    let preventable = match proposal {
        Proposal::Damage(d) => crate::ops::damage_can_be_prevented(cx, d.source, d.combat),
        _ => false,
    };
    let mut found = Vec::new();
    let mut consider = |id: ObjectId, own_only: bool| {
        let card = card_of(cx, id);
        // A permanent that has lost all its abilities has no replacement effects (CR 613.1f).
        if !cards.replaces[card.0 as usize] || cx.lost_all_abilities(id) {
            return;
        }
        let asker = Asker { source: id, controller: controller_of(cx, id, proposal) };
        for (index, def) in cards.get(card).replacements.iter().enumerate() {
            let index = index as u8;
            if def.kind != kind || (own_only && !def.own_everywhere) || applied.contains(&(id, index, Origin::Printed)) {
                continue;
            }
            if (def.applies)(cx, &asker, proposal) {
                found.push(Candidate { source: id, index, origin: Origin::Printed });
            }
        }
        // Its static abilities' prevention effects function only while it is on the battlefield (CR 604.1).
        if let Proposal::Damage(d) = proposal
            && preventable
            && !own_only
        {
            for (index, def) in cards.get(card).preventions.iter().enumerate() {
                let index = index as u8;
                if !applied.contains(&(id, index, Origin::Prevention)) && (def.applies)(cx, &asker, d) {
                    found.push(Candidate { source: id, index, origin: Origin::Prevention });
                }
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
            // A prevention effect a spell or ability made (CR 615.3): of damage to whom it names, if it names anyone, and of
            // the damage its definition describes. One about an object ends with it (CR 400.7).
            EffectKind::Prevention { card, index, to, .. } => {
                if let Proposal::Damage(d) = proposal
                    && preventable
                    && (to == Target::None || to == d.target)
                    && (to.object().is_none() || live(cx.s(), e.affected))
                    && !applied.contains(&(e.source, index, Origin::GrantedPrevention(e.timestamp)))
                {
                    let def = &cards.get(card).granted_preventions[index as usize];
                    let asker = Asker { source: e.source, controller: e.controller };
                    if (def.applies)(cx, &asker, e.affected, d) {
                        found.push(Candidate { source: e.source, index, origin: Origin::GrantedPrevention(e.timestamp) });
                    }
                }
                continue;
            }
            _ => continue,
        };
        if !about_player && !live(cx.s(), e.affected) {
            continue;
        }
        let def = &cards.get(card_of(cx, e.source)).granted_replacements[index as usize];
        if def.kind != kind || applied.contains(&(e.source, index, Origin::Granted(e.timestamp))) {
            continue;
        }
        let asker = Asker { source: e.source, controller: e.controller };
        if (def.applies)(cx, &asker, e.affected, proposal) {
            found.push(Candidate { source: e.source, index, origin: Origin::Granted(e.timestamp) });
        }
    }
    found.sort_by_key(|c| (c.source, c.index, c.origin));
    // The discarded card's own madness (CR 702.35a), ordered with the rest (CR 616.1). It is offered first, as the
    // order its owner almost always wants: declining the cast still puts the card into the graveyard, where the other
    // effects meet it again. A player or bot who takes the first offer keeps the madness.
    if let Proposal::ZoneChange(z) = proposal
        && !applied.contains(&(z.object, 0, Origin::Madness))
        && madness_applies(cx, z)
    {
        found.insert(0, Candidate { source: z.object, index: 0, origin: Origin::Madness });
    }
    // A permission's "each land played this way enters tapped" (CR 614.1c), ordered with the rest (CR 616.1): it is of the
    // land itself, which is what the player ordering them is shown.
    if let Proposal::ZoneChange(z) = proposal
        && !applied.contains(&(z.object, 0, Origin::PlayedTapped))
        && played_tapped_applies(cx, z)
    {
        found.push(Candidate { source: z.object, index: 0, origin: Origin::PlayedTapped });
    }
    // Protection of whoever the damage would now be dealt to (CR 702.16e), ordered with the effects that would change it
    // (CR 616.1): prevented first, nothing is redirected; redirected first, it is judged again at its new recipient. It is
    // offered first, as the order the protected player almost always wants. Only asked about where some other effect
    // applies too: alone, `ops::deal_damage_with` prevents the damage itself.
    if let Proposal::Damage(d) = proposal
        && !found.is_empty()
        && let Some(by) = crate::ops::protection_prevents(cx, d.source, d.target, d.combat)
        && !applied.contains(&(by, 0, Origin::Protection))
    {
        found.insert(0, Candidate { source: by, index: 0, origin: Origin::Protection });
    }
    found
}

/// What an effect says it does, for the player who orders several (CR 616.1).
fn hint_of(cx: &Cx, c: &Candidate) -> &'static str {
    let card = cx.cards().get(card_of(cx, c.source));
    match c.origin {
        Origin::Printed => card.replacements[c.index as usize].hint,
        Origin::Granted(_) => card.granted_replacements[c.index as usize].hint,
        Origin::Madness => MADNESS_HINT,
        Origin::Protection => PROTECTION_HINT,
        Origin::Prevention => card.preventions[c.index as usize].hint,
        Origin::GrantedPrevention(timestamp) => match cx.s().effects.iter().find(|e| e.timestamp == timestamp).map(|e| e.kind) {
            Some(EffectKind::Prevention { card, index, .. }) => cx.cards().get(card).granted_preventions[index as usize].hint,
            _ => "",
        },
        Origin::PlayedTapped => PLAYED_TAPPED_HINT,
    }
}

/// Whether the effect prevents all the damage it applies to and uses nothing up: a static ability's prevention,
/// protection's, or a resolved effect that prevents all of it. Which of several such effects applies first changes nothing
/// but which one is recorded as having prevented it, so the affected player is not asked to order only these (CR 616.1).
fn prevents_all(cx: &Cx, c: &Candidate) -> bool {
    match c.origin {
        Origin::Protection | Origin::Prevention => true,
        Origin::GrantedPrevention(timestamp) => cx
            .s()
            .effects
            .iter()
            .any(|e| e.timestamp == timestamp && matches!(e.kind, EffectKind::Prevention { shield: Shield::All, .. })),
        _ => false,
    }
}

/// Asks the affected player which of several effects applies first (CR 616.1). When the question has no answer yet
/// the run is abandoned and replayed with it (see `Cx::ask_core`), so the first is taken to carry on to the end.
fn choose(cx: &mut Cx, proposal: &Proposal, found: &[Candidate]) -> usize {
    let ask = RequestKind::ChooseReplacement {
        subject: subject_of(proposal),
        life: match proposal {
            Proposal::LifeChange(l) => Some(l.amount),
            _ => None,
        },
        sources: found.iter().map(|c| c.source).collect(),
        hints: found.iter().map(|c| hint_of(cx, c)).collect(),
        damage: match proposal {
            Proposal::Damage(d) => Some(DamageAbout { source: d.source, to: d.target, amount: d.amount }),
            _ => None,
        },
        turn: false,
        counters: match proposal {
            Proposal::Counters(c) => CounterKind::ALL.into_iter().zip(c.counts).filter(|&(_, n)| n > 0).collect(),
            _ => Vec::new(),
        },
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
        let pick = if found.len() > 1 && !found.iter().all(|c| prevents_all(cx, c)) { choose(cx, proposal, &found) } else { 0 };
        let Candidate { source, index, origin } = found[pick];
        applied.push((source, index, origin));
        match origin {
            Origin::Printed => {
                let def = &cx.cards().get(card_of(cx, source)).replacements[index as usize];
                let (apply, then) = (def.apply, def.then);
                let asker = Asker { source, controller: controller_of(cx, source, proposal) };
                let before = *proposal;
                apply(&*cx, &asker, proposal);
                if let Some(then) = then {
                    then(cx, &asker, &before, proposal);
                }
            }
            Origin::Granted(timestamp) => {
                let e = *cx.s().effects.iter().find(|e| e.timestamp == timestamp).expect("the effect that applies");
                let def = &cx.cards().get(card_of(cx, source)).granted_replacements[index as usize];
                let asker = Asker { source, controller: e.controller };
                (def.apply)(&*cx, &asker, e.affected, proposal);
            }
            // "That player discards it, but exiles it instead of putting it into their graveyard" (CR 702.35a): the
            // graveyard is its owner's (CR 400.3), and so is the exile it goes to.
            Origin::Madness => {
                if let Proposal::ZoneChange(z) = proposal {
                    z.to = Zone::exile(z.to.player);
                    z.madness = true;
                }
            }
            // CR 614.1c: it enters tapped.
            Origin::PlayedTapped => {
                if let Proposal::ZoneChange(z) = proposal {
                    z.tapped = true;
                }
            }
            // CR 702.16e, 615.1: the damage is prevented, all of it.
            Origin::Protection | Origin::Prevention => {
                if let Proposal::Damage(d) = proposal {
                    prevent(cx, d, d.amount, source);
                }
            }
            Origin::GrantedPrevention(timestamp) => {
                if let Proposal::Damage(d) = proposal {
                    let at = cx.s().effects.iter().position(|e| e.timestamp == timestamp).expect("the effect that applies");
                    let mut e = cx.s().effects[at];
                    let EffectKind::Prevention { card, index, to, shield } = e.kind else { unreachable!("a prevention effect") };
                    match shield {
                        Shield::All => prevent(cx, d, d.amount, source),
                        // CR 615.7: each 1 damage prevented reduces the shield by 1; once it is 0 the rest is dealt, and
                        // the effect is used up.
                        Shield::Next(left) => {
                            let prevented = left.min(d.amount);
                            prevent(cx, d, prevented, source);
                            if prevented == left {
                                cx.st().effects_retain(|f| f.timestamp != timestamp);
                            } else {
                                e.kind = EffectKind::Prevention { card, index, to, shield: Shield::Next(left - prevented) };
                                cx.st().effect_set(at, e);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Whether a replacement effect might change damage about to be dealt ("it deals that much damage plus 1 instead"): one a
/// spell or ability made, or a permanent's on the battlefield with a damage replacement. Asked before the damage is
/// proposed, so it only says the amounts may not stay as they are.
pub(crate) fn damage_may_change(cx: &Cx) -> bool {
    let cards = cx.cards();
    cx.s().effects.iter().any(|e| match e.kind {
        EffectKind::Replacement(i) | EffectKind::ReplacementRule(i) => {
            cards.get(card_of(cx, e.source)).granted_replacements[i as usize].kind == ReplacementKind::Damage
        }
        _ => false,
    }) || cx.s().battlefield().iter().any(|&id| {
        live(cx.s(), id)
            && !cx.lost_all_abilities(id)
            && cards.get(card_of(cx, id)).replacements.iter().any(|r| r.kind == ReplacementKind::Damage)
    })
}

/// The prevention shields that would apply to this damage (CR 615.7), taken together: the source of the first, and how much
/// they have left to prevent between them. None where no shield applies, or the damage can't be prevented (CR 615.12).
pub(crate) fn shields(cx: &Cx, d: &DamageEvent) -> Option<(ObjectId, u16)> {
    if !cx.s().effects.iter().any(|e| matches!(e.kind, EffectKind::Prevention { shield: Shield::Next(_), .. })) {
        return None;
    }
    if !crate::ops::damage_can_be_prevented(cx, d.source, d.combat) {
        return None;
    }
    let mut found: Option<(ObjectId, u16)> = None;
    for e in &cx.s().effects {
        let EffectKind::Prevention { card, index, to, shield: Shield::Next(left) } = e.kind else { continue };
        if !(to == Target::None || to == d.target) || (to.object().is_some() && !live(cx.s(), e.affected)) {
            continue;
        }
        let def = &cx.cards().get(card).granted_preventions[index as usize];
        if (def.applies)(cx, &Asker { source: e.source, controller: e.controller }, e.affected, d) {
            found = Some(found.map_or((e.source, left), |(s, n)| (s, n.saturating_add(left))));
        }
    }
    found
}

/// Prevents `amount` of the damage (CR 615.1) by the effect of `by`, and records it (CR 615.6, 615.13).
fn prevent(cx: &mut Cx, d: &mut DamageEvent, amount: u16, by: ObjectId) {
    if amount == 0 {
        return;
    }
    d.amount -= amount;
    cx.emit(Event::DamagePrevented { source: d.source, target: d.target, amount, combat: d.combat, by });
}

/// An object about to change zones (CR 614.1a, 614.1c): what remains of the move once every effect has applied.
pub fn zone_change(cx: &mut Cx, change: ZoneChange) -> ZoneChange {
    if !possible(cx, Some(change.object)) && !madness_applies(cx, &change) && !played_tapped_applies(cx, &change) {
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
/// battlefield if it is entering (CR 614.12). `effect` says an effect puts them, not a cost or a turn-based action
/// (`CounterPlacement::effect`).
pub fn counters(
    cx: &mut Cx,
    object: ObjectId,
    controller: PlayerId,
    entering: bool,
    effect: bool,
    counts: [i16; CounterKind::COUNT],
) -> [i16; CounterKind::COUNT] {
    if counts.iter().all(|&n| n <= 0) {
        return counts;
    }
    // CR 101.2, 122.6: counters of a kind the permanent can't have put on it are not put on it (Melira, Sylvok Outcast's
    // "creatures you control can't have -1/-1 counters put on them"), entering with them included; no replacement effect sees
    // them, and none adds them back.
    let counts = without_banned(cx, object, controller, counts);
    if !possible(cx, Some(object)) || counts.iter().all(|&n| n <= 0) {
        return counts;
    }
    // CR 122.6a: counters an object enters with are put on it by its controller, unless the effect names another player.
    let putter = if entering { Some(controller) } else { cx.actor() };
    let mut proposal = Proposal::Counters(CounterPlacement { object, controller, entering, counts, putter, effect });
    apply(cx, &mut proposal);
    match proposal {
        Proposal::Counters(placement) => without_banned(cx, object, controller, placement.counts),
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}

/// `counts` less every kind `object`, controlled by `controller`, can't have put on it (CR 101.2, 122.6).
fn without_banned(cx: &Cx, object: ObjectId, controller: PlayerId, mut counts: [i16; CounterKind::COUNT]) -> [i16; CounterKind::COUNT] {
    for (i, n) in counts.iter_mut().enumerate() {
        if *n > 0 && crate::static_rules::counters_banned(cx, cx.cards(), object, controller, CounterKind::ALL[i]) {
            *n = 0;
        }
    }
    counts
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

/// Damage about to be dealt (CR 614.1a, 120.3): what remains of it once every effect has applied - how much, and whom it
/// is dealt to, which a redirection ("is dealt to this creature instead") changes, after which the effects not yet
/// applied are judged again against the new recipient and its controller orders them (CR 614.6, 616.1f).
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

/// Life `player` is about to gain (`amount` above zero) or lose (below zero) (CR 119.3, 614.1a): what remains of it once
/// every effect has applied, in the order the player chooses (CR 616.1), each judged again on the event as the one
/// before it left it (CR 616.1f), so a gain turned into a loss meets the effects on losing life. Zero is no event
/// (CR 119.10).
pub fn life_change(cx: &mut Cx, player: PlayerId, amount: i32) -> i32 {
    if !possible(cx, None) {
        return amount;
    }
    let mut proposal = Proposal::LifeChange(LifeChange { player, amount });
    apply(cx, &mut proposal);
    match proposal {
        Proposal::LifeChange(l) => l.amount,
        _ => unreachable!("an effect cannot change the kind of the event"),
    }
}
