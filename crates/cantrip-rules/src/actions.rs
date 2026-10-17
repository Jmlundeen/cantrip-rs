//! What a player can do with priority, and doing it: playing a land (CR 305), casting a spell (CR 601), activating
//! an ability (CR 602, 605), and paying for them (CR 601.2g-h, 118.12).
//!
//! The options are listed in the C# engine's order: mana abilities, other activated abilities and instant-speed
//! casts, then (at sorcery timing) land plays and the remaining casts. A scenario that names "the Forest" means the
//! first one offered.

use cantrip_api::*;
use cantrip_state::Span;

use crate::layers::{self, live, summoning_sick};
use crate::{Cx, Flow, R, Stop, ops, static_rules};

/// What a static ability that offers another cost for an activated ability asks (CR 118.9, 602.2b): Forge Anew.
const ALTERNATIVE_ACTIVATION_COST: Question = Question::new(
    "AlternativeCost",
    "pay its alternative cost instead of the ability's mana cost?",
    "",
    "Use the alternative cost",
    "Pay the mana cost",
);

pub fn sorcery_timing(cx: &Cx, player: PlayerId) -> bool {
    let s = cx.s();
    s.turn.active == player && s.turn.step().is_main() && s.stack.is_empty()
}

/// Everything `player` may do now beyond passing.
pub fn priority_options(cx: &Cx, player: PlayerId) -> Vec<ActionOption> {
    let mut options = Vec::new();
    mana_ability_options(cx, player, &mut options);
    // Split second (CR 702.61a): with such a spell on the stack, mana abilities are all anyone may do.
    if split_second(cx) {
        return options;
    }
    activated_options(cx, player, &mut options);
    let available = std::cell::OnceCell::new();
    castable(cx, player, &available, true, &mut options);
    if sorcery_timing(cx, player) {
        land_plays(cx, player, &mut options);
        castable(cx, player, &available, false, &mut options);
    }
    options
}

/// Whether a spell with split second is on the stack (CR 702.61a): read from the printed cards of what is on the stack,
/// which is nothing at all for nearly every priority pass.
fn split_second(cx: &Cx) -> bool {
    cx.s().stack.iter().any(|i| {
        i.kind == StackKind::Spell && {
            let obj = cx.s().obj(i.object);
            cx.card(obj.copy.map_or(obj.card, |c| c.card)).keywords.has(Keyword::SplitSecond)
        }
    })
}

fn activated(cx: &Cx, ability: AbilityRef) -> Option<&'static ActivatedDef> {
    match cx.cards().ability(ability) {
        AbilityDef::Activated(a) => Some(a),
        AbilityDef::Triggered(_) | AbilityDef::Static(_) => None,
    }
}

/// Whether the non-mana parts of a cost could be paid (CR 602.2b, 118.3): an activation that cannot be is not
/// offered at all.
fn can_pay_other(cx: &Cx, source: ObjectId, controller: PlayerId, cost: &Cost) -> bool {
    let obj = cx.s().obj(source);
    // A creature's {T} ability needs it under its controller's control since their turn began, unless it has haste or the
    // ability may be activated as though it had (CR 302.6).
    if cost.tap
        && (obj.tapped
            || summoning_sick(cx.cards(), cx.s(), &cx.g.log, source) && !static_rules::activates_as_though_haste(cx, controller, source))
    {
        return false;
    }
    if let Some(loyalty) = cost.loyalty
        && loyalty < 0
        && loyalty != LOYALTY_MINUS_X
        && obj.counters(CounterKind::Loyalty) < -(loyalty as i16)
    {
        return false;
    }
    if !can_pay_life(cx, controller, cost.life) {
        return false;
    }
    if cost.return_self && !obj.zone.is(ZoneKind::Battlefield) {
        return false;
    }
    if let Some(variable) = cost.variable
        && !variable_payable(cx, source, controller, variable)
    {
        return false;
    }
    if cost.unattach_self && !obj.attached_to.is_some_and(|h| cx.s().obj(h).zone.is(ZoneKind::Battlefield) && cx.s().obj(h).became.is_none()) {
        return false;
    }
    if let Some((kind, n)) = cost.remove_counters
        && obj.counters(kind) < n as i16
    {
        return false;
    }
    if cost.remove_any_counter && !obj.counters.iter().any(|&n| n > 0) {
        return false;
    }
    if let Some((n, filter)) = cost.discard {
        let asker = Asker { source, controller };
        let hand = cx.s().zone(Zone::hand(controller)).iter().filter(|&&c| c != source && filter(cx, &asker, c)).count();
        if hand < n as usize {
            return false;
        }
    }
    if let Some((n, filter)) = cost.exile_from_hand {
        let asker = Asker { source, controller };
        let hand = cx.s().zone(Zone::hand(controller)).iter().filter(|&&c| c != source && filter(cx, &asker, c)).count();
        if hand < n as usize {
            return false;
        }
    }
    if cost.exile_from_graveyard > 0 && graveyard_exiles(cx, controller, source, cost).len() < cost.exile_from_graveyard as usize {
        return false;
    }
    // Collect evidence N (CR 701.59a): payable only if the graveyard's cards, the source apart, add up to N.
    if cost.collect_evidence > 0 {
        let total: u32 = cx.s().zone(Zone::graveyard(controller)).iter().filter(|&&c| c != source).map(|&c| cx.mana_value(c)).sum();
        if total < cost.collect_evidence as u32 {
            return false;
        }
    }
    if let Some(p) = cost.permanents {
        let asker = Asker { source, controller };
        let mut candidates = cx.permanents_of(controller).into_iter().filter(|&id| pays_permanents(cx, cost, &p, &asker, id)).count();
        // Tapped permanents are judged once the mana abilities have been activated (CR 601.2g-h): an untapped one that
        // passes the filter and one of its controller's mana abilities could tap will do.
        if p.tapped && candidates < p.count as usize {
            let tappable = tappable_for_mana(cx, controller);
            candidates += cx
                .permanents_of(controller)
                .into_iter()
                .filter(|&id| !cx.s().obj(id).tapped && (p.filter)(cx, &asker, id) && tappable.contains(&id))
                .count();
        }
        if candidates < p.count as usize {
            return false;
        }
    }
    if let Some((n, _, _)) = cost.behold
        && behold_candidates(cx, source, controller, cost).len() < n as usize
    {
        return false;
    }
    // CR 701.68b: a player who controls no creature can't choose to blight.
    if cost.blight > 0 && blight_candidates(cx, controller).is_empty() {
        return false;
    }
    if !counters_among_payable(cx, source, controller, cost) {
        return false;
    }
    if let Some(t) = cost.tap_creatures {
        let (_, powers) = tap_candidates(cx, source, controller, &t);
        if powers.iter().map(|&p| p as u32).sum::<u32>() < t.power as u32 {
            return false;
        }
    }
    if let Some(m) = cost.materials {
        let candidates = material_candidates(cx, source, controller, m);
        if candidates.len() < m.count as usize || !parts_can_be_matched(cx, source, controller, m, &candidates) {
            return false;
        }
    }
    true
}

/// Whether materials named one by one can each be a separate object of the candidates (the Throne of the Grim Captain
/// ruling): a matching of every part to a distinct candidate passing its filter, found by trying each in turn. True for
/// materials of one kind.
fn parts_can_be_matched(cx: &Cx, source: ObjectId, controller: PlayerId, m: &Materials, candidates: &[ObjectId]) -> bool {
    m.parts.is_empty() || assign_parts(cx, &Asker { source, controller }, m.parts, candidates, &mut Vec::new())
}

/// Whether `parts` can each take a distinct candidate passing its filter, none of `used` among them.
fn assign_parts(cx: &Cx, asker: &Asker, parts: &[MaterialPart], candidates: &[ObjectId], used: &mut Vec<ObjectId>) -> bool {
    let Some((part, rest)) = parts.split_first() else { return true };
    for &c in candidates {
        if !used.contains(&c) && (part.filter)(cx, asker, c) {
            used.push(c);
            let found = assign_parts(cx, asker, rest, candidates, used);
            used.pop();
            if found {
                return true;
            }
        }
    }
    false
}

/// What craft may exile as its materials (CR 702.167a): the permanents the payer controls, then the cards in their
/// graveyard, that pass the materials' filter, the source apart (it is exiled as itself, not as a material).
fn material_candidates(cx: &Cx, source: ObjectId, controller: PlayerId, m: &Materials) -> Vec<ObjectId> {
    let asker = Asker { source, controller };
    let mut out = cx.permanents_of(controller);
    // Cards in the graveyard: a token put there (a Treasure sacrificed for the mana) is no card, and ceases to exist there
    // (CR 111.7, 704.5d).
    out.extend(cx.s().zone(Zone::graveyard(controller)).iter().copied().filter(|&id| !cx.s().obj(id).token));
    out.retain(|&id| id != source && (m.filter)(cx, &asker, id));
    out
}

/// Chooses craft's materials (CR 702.167a, 601.2h), always asked, even when the choice is forced: exactly `count` of
/// them, or `count` or more. Too few left to choose among (one gone since the ability was offered) is a cost that can't
/// be paid, and the activation is taken back.
fn choose_materials(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some(m) = cost.materials else { return Ok(Vec::new()) };
    // The source is exiled with them: one a mana ability moved while paying can't be (CR 601.2h).
    if !live(cx.s(), source) {
        return Err(Stop::Cancel);
    }
    let cards = material_candidates(cx, source, player, m);
    if cards.len() < m.count as usize {
        return Err(Stop::Cancel);
    }
    // Materials named one by one: each asked in turn among what is left that passes its filter and still leaves each later
    // one a separate object, so no choice offered is one that can't be finished. With none (one gone since the ability was
    // offered) the cost can't be paid, and the activation is taken back (CR 602.2b, 601.2h).
    if !m.parts.is_empty() {
        let asker = Asker { source, controller: player };
        let mut chosen: Vec<ObjectId> = Vec::new();
        for (i, part) in m.parts.iter().enumerate() {
            let left: Vec<ObjectId> = cards
                .iter()
                .copied()
                .filter(|&c| {
                    !chosen.contains(&c) && (part.filter)(cx, &asker, c) && {
                        let mut used: Vec<ObjectId> = chosen.iter().copied().chain(std::iter::once(c)).collect();
                        assign_parts(cx, &asker, &m.parts[i + 1..], &cards, &mut used)
                    }
                })
                .collect();
            if left.is_empty() {
                return Err(Stop::Cancel);
            }
            let ask = RequestKind::Craft { source, cards: left, minimum: 1, maximum: 1, quality: part.quality };
            match cx.ask_core(player, ask, true)? {
                Answer::Cards(one) => chosen.extend(one),
                _ => return Err(Stop::Cancel),
            }
        }
        return Ok(chosen);
    }
    let maximum = if m.or_more { cards.len().min(255) as u8 } else { m.count };
    let ask = RequestKind::Craft { source, cards, minimum: m.count, maximum, quality: m.quality };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Pays craft's exile (CR 702.167a): the source and the materials chosen, from the battlefield and the graveyard, all at
/// once (CR 601.2h). The source goes first, so the history holds it and then the materials, just before the activation:
/// what `cantrip_mechanics::craft` reads to find the exiled cards used to craft a permanent (CR 702.167c).
fn pay_materials(cx: &mut Cx, source: ObjectId, chosen: &[ObjectId]) {
    // Nothing that is no longer the object it was is moved (CR 400.7).
    let moves: Vec<ops::Moving> = std::iter::once(source)
        .chain(chosen.iter().copied())
        .filter(|&id| live(cx.s(), id))
        .map(|id| ops::Moving::to(id, Zone::exile(cx.s().obj(id).owner)))
        .collect();
    ops::move_together(cx, &moves, ZoneReason::Exile);
}

/// What a cost that beholds may choose among (CR 701.4a): the permanents its payer controls and the cards in their hand,
/// the source apart, that pass the filter.
fn behold_candidates(cx: &Cx, source: ObjectId, controller: PlayerId, cost: &Cost) -> Vec<ObjectId> {
    let Some((_, filter, _)) = cost.behold else { return Vec::new() };
    let asker = Asker { source, controller };
    let mut out = cx.permanents_of(controller);
    out.extend(cx.s().zone(Zone::hand(controller)).iter().copied());
    out.retain(|&id| id != source && filter(cx, &asker, id));
    out
}

/// Chooses what a cost beholds (CR 701.4a, 601.2h), always asked, even when the choice is forced.
fn choose_behold(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some((n, _, _)) = cost.behold else { return Ok(Vec::new()) };
    let eligible = behold_candidates(cx, source, player, cost);
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Behold, cards: eligible.clone(), eligible, count: n, up_to: false, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Pays the behold part of a cost (CR 701.4a): each card chosen from the hand is revealed; a permanent is only chosen.
fn pay_behold(cx: &mut Cx, player: PlayerId, chosen: &[ObjectId]) {
    let revealed: Vec<ObjectId> = chosen.iter().copied().filter(|&id| cx.s().obj(id).zone.is(ZoneKind::Hand)).collect();
    if !revealed.is_empty() {
        Game::reveal(cx, player, &revealed);
    }
}

/// The creatures a cost that blights may put its counters on (CR 701.68a).
fn blight_candidates(cx: &Cx, controller: PlayerId) -> Vec<ObjectId> {
    cx.permanents_of(controller).into_iter().filter(|&id| cx.chars(id).is(CardType::Creature)).collect()
}

/// Chooses the creature a cost blights (CR 701.68a, 601.2h), always asked, even when the choice is forced.
fn choose_blight(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Option<ObjectId>> {
    if cost.blight == 0 {
        return Ok(None);
    }
    let eligible = blight_candidates(cx, player);
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Blight, cards: eligible.clone(), eligible, count: 1, up_to: false, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => cards.first().copied().map(Some).ok_or(Stop::Cancel),
        _ => Err(Stop::Cancel),
    }
}

/// Pays the blight part of a cost (CR 701.68a): the -1/-1 counters go on the creature chosen.
fn pay_blight(cx: &mut Cx, cost: &Cost, chosen: Option<ObjectId>) {
    if let Some(creature) = chosen {
        ops::add_counters(cx, creature, CounterKind::MinusOne, cost.blight as i16);
    }
}

/// What a cost that removes counters from among permanents may take them from (CR 118.3, 122.1): each kind of counter on
/// each permanent the payer controls that passes the filter, with how many there are, in battlefield order.
fn counter_pools(cx: &Cx, source: ObjectId, controller: PlayerId, filter: ObjectFilter) -> Vec<CounterPool> {
    let asker = Asker { source, controller };
    let mut pools = Vec::new();
    for id in cx.permanents_of(controller) {
        if !filter(cx, &asker, id) {
            continue;
        }
        let obj = cx.s().obj(id);
        for kind in CounterKind::ALL {
            let count = obj.counters(kind);
            if count > 0 {
                pools.push(CounterPool { object: id, kind, count: count as u16 });
            }
        }
    }
    pools
}

/// Whether there are counters enough among the permanents a cost removes them from (CR 118.3).
fn counters_among_payable(cx: &Cx, source: ObjectId, controller: PlayerId, cost: &Cost) -> bool {
    cost.remove_counters_among
        .is_none_or(|(n, filter)| counter_pools(cx, source, controller, filter).iter().map(|p| p.count as u32).sum::<u32>() >= n as u32)
}

/// Chooses the counters a cost removes from among permanents (CR 601.2h), always asked, even when the choice is forced. Too
/// few left (some gone since the action was offered) is a cost that can't be paid, and the spell or ability is taken back.
fn choose_counters_among(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<(ObjectId, CounterKind, u16)>> {
    let Some((total, filter)) = cost.remove_counters_among else { return Ok(Vec::new()) };
    let pools = counter_pools(cx, source, player, filter);
    if pools.iter().map(|p| p.count as u32).sum::<u32>() < total as u32 {
        return Err(Stop::Cancel);
    }
    match cx.ask_core(player, RequestKind::RemoveCounters { source, pools, total }, true)? {
        Answer::Counters(taken) => Ok(taken),
        _ => Err(Stop::Cancel),
    }
}

/// Pays a removal of counters from among permanents (CR 118.3): each chosen number of each kind comes off its permanent.
fn pay_counters_among(cx: &mut Cx, taken: &[(ObjectId, CounterKind, u16)]) {
    for &(object, kind, n) in taken {
        ops::add_counters(cx, object, kind, -(n as i16));
    }
}

/// The creatures that could be tapped for crew or saddle (CR 702.122a, 702.171a): the controller's other untapped
/// creatures, summoning sick or not, each with what it counts as towards the total power (CR 702.122b).
fn tap_candidates(cx: &Cx, source: ObjectId, controller: PlayerId, t: &TapCreatures) -> (Vec<ObjectId>, Vec<u8>) {
    let mut cards = Vec::new();
    let mut powers = Vec::new();
    for id in cx.permanents_of(controller) {
        if id == source || cx.s().obj(id).tapped {
            continue;
        }
        let c = cx.chars(id);
        if !c.is(CardType::Creature) {
            continue;
        }
        let bonus = if t.crew && c.has(Keyword::CrewsAsThoughPowerTwoGreater) { 2 } else { 0 };
        cards.push(id);
        powers.push((c.power.unwrap_or(0).max(0) + bonus).min(255) as u8);
    }
    (cards, powers)
}

/// Chooses the creatures a crew or saddle cost taps (CR 601.2h), always asked, even when the choice is forced.
fn choose_tappers(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some(t) = cost.tap_creatures else { return Ok(Vec::new()) };
    let (cards, powers) = tap_candidates(cx, source, player, &t);
    let ask = RequestKind::TapForPower { source, purpose: Purpose::TapToPay, cards, powers, minimum: t.power };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Pays the exert and tap-creatures parts of a cost.
fn pay_exert_and_tappers(cx: &mut Cx, source: ObjectId, cost: &Cost, tappers: &[ObjectId]) {
    if cost.exert {
        ops::exert(cx, source);
    }
    if cost.unattach_self {
        ops::unattach(cx, source);
    }
    for &id in tappers {
        ops::tap(cx, id);
    }
}

/// "Activate only once each turn" (CR 602.5b-c): whether the `index`th ability of this object has been activated this
/// turn, by anyone. Counted for the object (a new object is a new count) and for each instance of an identically
/// worded ability on its own.
fn limit_reached(cx: &Cx, id: ObjectId, chars: &Chars, index: usize, def: &ActivatedDef) -> bool {
    // "Activate only once" (CR 602.5b; exhaust, 702.177a; power-up, 702.193a): this permanent has activated this
    // ability before, in any turn. A permanent that left and came back is another object, with the ability unused.
    if def.once
        && let Some(ability) = chars.abilities().nth(index)
        && cx
            .history()
            .select(EventQuery::kind(EventMask::ABILITY_ACTIVATED).object(id))
            .any(|r| matches!(r.event, Event::AbilityActivated { ability: a, .. } if a == ability))
    {
        return true;
    }
    if granted_limit_reached(cx, id, chars, index) {
        return true;
    }
    if !def.once_each_turn {
        return false;
    }
    let obj = cx.s().obj(id);
    obj.activated_turn == cx.s().turn.number && obj.activated_mask & (1 << index) != 0
}

/// "You may activate each of those abilities only once each turn" (CR 602.5b, `CardDef::granted_once_each_turn`): the
/// `index`th ability of `id` is an instance of an ability its permanent's static ability gave it with that limit, and as
/// many of them have been activated this turn as it has limited instances, asked of the history. An instance it has with
/// no limit (printed, or given by something else) is the one activated first: identical abilities are interchangeable,
/// so the limit binds only once every instance it has is a limited one.
fn granted_limit_reached(cx: &Cx, id: ObjectId, chars: &Chars, index: usize) -> bool {
    let Some(limited) = cx.card(chars.card).granted_once_each_turn else { return false };
    let Some(ability) = chars.abilities().nth(index) else { return false };
    let limited = limited(cx, id, ability);
    if limited == 0 || chars.abilities().filter(|&a| a == ability).count() > limited {
        return false;
    }
    let used = cx
        .history()
        .select(EventQuery::kind(EventMask::ABILITY_ACTIVATED).object(id).this_turn())
        .filter(|r| matches!(r.event, Event::AbilityActivated { ability: a, .. } if a == ability))
        .count();
    used >= limited
}

/// Notes that the `index`th ability of an object was activated this turn.
fn note_activation(cx: &mut Cx, id: ObjectId, index: usize, def: &ActivatedDef) {
    if !def.once_each_turn {
        return;
    }
    let turn = cx.s().turn.number;
    cx.st().update_obj(id, |o| {
        if o.activated_turn != turn {
            o.activated_turn = turn;
            o.activated_mask = 0;
        }
        o.activated_mask |= 1 << index;
    });
}

/// Chooses the cards a cost discards (CR 601.2h), always asked, even when the choice is forced.
fn choose_discards(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    // "Discard your hand" (CR 701.9) chooses nothing: every card in it, the source (on the stack by now) not among them.
    if cost.discard_hand {
        return Ok(cx.s().zone(Zone::hand(player)).iter().copied().filter(|&c| c != source).collect());
    }
    let Some((n, filter)) = cost.discard else { return Ok(Vec::new()) };
    let asker = Asker { source, controller: player };
    let hand: Vec<ObjectId> = cx.s().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| c != source && filter(cx, &asker, c)).collect();
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Discard, cards: hand, eligible, count: n, up_to: false, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Chooses the cards a cost exiles from the hand (CR 601.2h, 118.9), always asked, even when the choice is forced: the
/// hand is shown, and the cards that pass the filter are the ones that may be chosen.
fn choose_hand_exiles(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some((n, filter)) = cost.exile_from_hand else { return Ok(Vec::new()) };
    let asker = Asker { source, controller: player };
    let hand: Vec<ObjectId> = cx.s().zone(Zone::hand(player)).to_vec();
    let eligible: Vec<ObjectId> = hand.iter().copied().filter(|&c| c != source && filter(cx, &asker, c)).collect();
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::ExileToPay, cards: hand, eligible, count: n, up_to: false, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Collect evidence N (CR 701.59a): the cards, with total mana value N or more, that a cost exiles from the graveyard;
/// always asked.
fn choose_evidence(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    if cost.collect_evidence == 0 {
        return Ok(Vec::new());
    }
    let cards: Vec<ObjectId> = cx.s().zone(Zone::graveyard(player)).iter().copied().filter(|&c| c != source).collect();
    let values: Vec<u8> = cards.iter().map(|&c| cx.mana_value(c).min(255) as u8).collect();
    let ask = RequestKind::CollectEvidence { source, cards, values, minimum: cost.collect_evidence };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// The cards of `player`'s graveyard a cost may exile: any but the source, or those of the quality it names (CR 601.2h).
fn graveyard_exiles(cx: &Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> Vec<ObjectId> {
    let asker = Asker { source, controller: player };
    cx.s()
        .zone(Zone::graveyard(player))
        .iter()
        .copied()
        .filter(|&c| c != source && cost.exile_from_graveyard_of.is_none_or(|f| f(cx, &asker, c)))
        .collect()
}

/// Chooses the cards a cost exiles from the graveyard (CR 601.2h), always asked, even when the choice is forced.
fn choose_exiles(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    if cost.exile_from_graveyard == 0 {
        return choose_evidence(cx, player, source, cost);
    }
    let eligible = graveyard_exiles(cx, player, source, cost);
    let ask = RequestKind::ChooseCards {
        source,
        purpose: Purpose::ExileToPay,
        cards: cx.s().zone(Zone::graveyard(player)).iter().copied().filter(|&c| c != source).collect(),
        eligible,
        count: cost.exile_from_graveyard,
        up_to: false, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Pays the exile part of a cost: the chosen cards, together.
fn pay_exiles(cx: &mut Cx, chosen: &[ObjectId]) {
    if chosen.is_empty() {
        return;
    }
    let moves: Vec<ops::Moving> = chosen.iter().map(|&id| ops::Moving::to(id, Zone::exile(cx.s().obj(id).owner))).collect();
    ops::move_together(cx, &moves, ZoneReason::Exile);
}

/// Whether `id` may pay a permanents cost now: it passes the cost's filter, and is tapped if the cost asks for tapped
/// permanents ("return a tapped creature you control", CR 702.188a). When the cost taps permanents and also has {T}
/// ("{T}, Tap an untapped creature you control"), the source is not one of them: it is tapped by the {T} part of the same
/// cost, and one permanent can't be tapped to pay both (CR 118.3, 602.2b).
fn pays_permanents(cx: &Cx, cost: &Cost, p: &PermanentCost, asker: &Asker, id: ObjectId) -> bool {
    !(cost.tap && cost.tap_permanents && id == asker.source) && (!p.tapped || cx.s().obj(id).tapped) && (p.filter)(cx, asker, id)
}

/// The untapped permanents `player`'s mana abilities could tap as they are activated now (CR 601.2g, 605.3a): each one
/// whose own mana ability has {T} in its cost, and each one a mana ability's "tap an untapped [permanent] you control"
/// could tap.
fn tappable_for_mana(cx: &Cx, player: PlayerId) -> Vec<ObjectId> {
    let mut options = Vec::new();
    mana_ability_options(cx, player, &mut options);
    let mut out = Vec::new();
    for o in &options {
        let Some(source) = o.source else { continue };
        if o.index >= PLAYER_MANA_ABILITY {
            continue;
        }
        let Some(def) = cx.chars(source).abilities().nth(o.index as usize).and_then(|a| activated(cx, a)) else { continue };
        if def.cost.tap && !cx.s().obj(source).tapped {
            out.push(source);
        }
        if def.cost.tap_permanents {
            out.extend(o.target_candidates.iter().filter_map(|t| t.object()));
        }
    }
    out
}

/// Whether a cost's permanents are chosen once its mana is paid (CR 601.2h): those that must be tapped, which a mana
/// ability activated while paying (CR 601.2g) may have tapped. The others are chosen with the rest of the cost, before.
fn permanents_after_mana(cost: &Cost) -> bool {
    cost.permanents.is_some_and(|p| p.tapped)
}

/// Chooses the permanents a cost names (CR 601.2h), always asked, even when the choice is forced. Too few to choose
/// among (tapped ones, judged once the mana is paid) is a cost that can't be paid: the announcement is taken back.
fn choose_permanents(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some(p) = cost.permanents else { return Ok(Vec::new()) };
    let asker = Asker { source, controller: player };
    let eligible: Vec<ObjectId> = cx.permanents_of(player).into_iter().filter(|&id| pays_permanents(cx, cost, &p, &asker, id)).collect();
    if p.tapped && eligible.len() < p.count as usize {
        return Err(Stop::Cancel);
    }
    let purpose = if cost.tap_permanents {
        Purpose::TapToPay
    } else if p.to_hand {
        Purpose::ReturnToHand
    } else {
        Purpose::SacrificeCost
    };
    // A cost of tapped permanents says so, and names creatures when those are all it can take (web-slinging's).
    let kind = match p.tapped {
        true if eligible.iter().all(|&id| cx.chars(id).is(CardType::Creature)) => "tapped creature",
        true => "tapped permanent",
        false => "",
    };
    let ask = RequestKind::ChooseCards { source, purpose, cards: eligible.clone(), eligible, count: p.count, up_to: false, kind, total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// Pays the discard part of a cost: the chosen cards, together (CR 701.9).
fn pay_discards(cx: &mut Cx, chosen: &[ObjectId]) {
    if chosen.is_empty() {
        return;
    }
    ops::discard_together(cx, chosen);
}

/// Pays the life and counters parts of a cost.
fn pay_life_and_counters(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) {
    if cost.life > 0 {
        ops::change_life(cx, player, -(cost.life as i32));
    }
    if let Some((kind, n)) = cost.remove_counters {
        cx.st().update_obj(source, |o| {
            let c = &mut o.counters[kind as usize];
            *c = (*c - n as i16).max(0);
        });
        ops::end_lapsed_durations(cx, source);
    }
    // Counters a cost puts on its source are put by the player paying it (CR 118.3): what replaces counters being put applies.
    if let Some((kind, n)) = cost.put_counters {
        ops::add_counters(cx, source, kind, n as i16);
    }
}

/// The kind of counter a cost that removes a counter of any kind takes from the source (CR 118.3): the only kind it has,
/// or the payer's choice among several, made as the cost is chosen.
fn choose_counter_kind(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Option<CounterKind>> {
    if !cost.remove_any_counter {
        return Ok(None);
    }
    let kinds: Vec<CounterKind> =
        CounterKind::ALL.into_iter().filter(|&k| cx.s().obj(source).counters[k as usize] > 0).collect();
    if kinds.len() <= 1 {
        return Ok(kinds.first().copied());
    }
    let ask = RequestKind::ChooseModes {
        source,
        modes: kinds.iter().map(|&k| Label::Counter(k)).collect(),
        eligible: (0..kinds.len() as u8).collect(),
        count: 1,
        at_most: 0,
        affordable: Vec::new(),
        additional_cost: true,
        repeat: false,
    };
    match cx.ask_core(player, ask, true)? {
        Answer::Modes(chosen) => Ok(chosen.first().map(|&i| kinds[i as usize])),
        _ => Err(Stop::Cancel),
    }
}

/// The permanents a variable sacrifice may take, or the other cards in the graveyard an exile may take (CR 601.2h).
fn variable_candidates(cx: &Cx, source: ObjectId, player: PlayerId, variable: VariableCost) -> Vec<ObjectId> {
    match variable {
        VariableCost::SacrificeX(filter) | VariableCost::SacrificeOneOrMore(filter) => {
            let asker = Asker { source, controller: player };
            cx.permanents_of(player).into_iter().filter(|&id| filter(cx, &asker, id)).collect()
        }
        VariableCost::ExileFromGraveyardWithTypes(_) => {
            cx.s().zone(Zone::graveyard(player)).iter().copied().filter(|&c| c != source).collect()
        }
    }
}

/// How many card types appear among these cards (CR 205.2a).
fn types_among(cx: &Cx, cards: &[ObjectId]) -> usize {
    let mut seen = TypeSet::NONE;
    for &c in cards {
        for t in cx.chars(c).types.iter() {
            seen = seen.with(t);
        }
    }
    seen.iter().count()
}

/// Whether a variable cost could be paid at all: X may be zero, "one or more" needs one, and the exile needs enough types.
fn variable_payable(cx: &Cx, source: ObjectId, player: PlayerId, variable: VariableCost) -> bool {
    match variable {
        VariableCost::SacrificeX(_) => true,
        VariableCost::SacrificeOneOrMore(_) => !variable_candidates(cx, source, player, variable).is_empty(),
        VariableCost::ExileFromGraveyardWithTypes(n) => {
            types_among(cx, &variable_candidates(cx, source, player, variable)) >= n.max(1) as usize
        }
    }
}

fn ask_cards(cx: &mut Cx, player: PlayerId, source: ObjectId, purpose: Purpose, cards: Vec<ObjectId>, count: usize, up_to: bool) -> R<Vec<ObjectId>> {
    let ask = RequestKind::ChooseCards { source, purpose, cards: cards.clone(), eligible: cards, count: count.min(255) as u8, up_to, kind: "", total: None };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards),
        _ => Err(Stop::Cancel),
    }
}

/// What a variable cost takes, chosen as the cost is chosen (CR 601.2h): the permanents to sacrifice, or the cards to
/// exile. `x` is the X announced for `SacrificeX`.
fn choose_variable(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost, x: u16) -> R<(Vec<ObjectId>, Vec<ObjectId>)> {
    let Some(variable) = cost.variable else { return Ok((Vec::new(), Vec::new())) };
    let candidates = variable_candidates(cx, source, player, variable);
    match variable {
        VariableCost::SacrificeX(_) => {
            if x == 0 {
                return Ok((Vec::new(), Vec::new()));
            }
            // A creature that would have paid has gone since X was announced: the announcement is taken back.
            if candidates.len() < x as usize {
                return Err(Stop::Cancel);
            }
            Ok((ask_cards(cx, player, source, Purpose::SacrificeCost, candidates, x as usize, false)?, Vec::new()))
        }
        VariableCost::SacrificeOneOrMore(_) => {
            if candidates.is_empty() {
                return Err(Stop::Cancel);
            }
            // As the C# engine asked it: one mandatory permanent (unless it is the only one), then up to all the rest.
            let first = if candidates.len() > 1 {
                match ask_cards(cx, player, source, Purpose::SacrificeCost, candidates.clone(), 1, false)?.first() {
                    Some(&f) => f,
                    None => return Err(Stop::Cancel),
                }
            } else {
                candidates[0]
            };
            let mut chosen = vec![first];
            let rest: Vec<ObjectId> = candidates.into_iter().filter(|&c| c != first).collect();
            if !rest.is_empty() {
                let n = rest.len();
                let more = ask_cards(cx, player, source, Purpose::SacrificeCost, rest.clone(), n, true)?;
                chosen.extend(rest.into_iter().filter(|c| more.contains(c)));
            }
            Ok((chosen, Vec::new()))
        }
        VariableCost::ExileFromGraveyardWithTypes(n) => {
            if types_among(cx, &candidates) < n.max(1) as usize {
                return Err(Stop::Cancel);
            }
            let count = candidates.len();
            let chosen = ask_cards(cx, player, source, Purpose::ExileToPay, candidates, count, true)?;
            // A choice covering too few card types does not pay: the announcement is rewound.
            if types_among(cx, &chosen) < n.max(1) as usize {
                return Err(Stop::Cancel);
            }
            Ok((Vec::new(), chosen))
        }
    }
}

/// Pays the permanents part of a cost, once the rest is paid.
fn pay_permanents(cx: &mut Cx, cost: &Cost, chosen: &[ObjectId]) {
    if cost.tap_permanents {
        for &id in chosen {
            ops::tap(cx, id);
        }
        return;
    }
    match cost.permanents {
        Some(PermanentCost { to_hand: true, .. }) => {
            ops::return_to_hand(cx, chosen);
        }
        Some(_) => {
            ops::sacrifice(cx, chosen);
        }
        None => {}
    }
}

pub fn mana_ability_options(cx: &Cx, player: PlayerId, options: &mut Vec<ActionOption>) {
    let forbidders = static_rules::activation_forbidders(cx);
    for &id in cx.s().battlefield() {
        if cx.s().obj(id).controller != player {
            continue;
        }
        let chars = cx.chars(id);
        for (index, ability) in chars.abilities().enumerate() {
            let Some(def) = activated(cx, ability) else { continue };
            if !def.is_mana_ability()
                || def.from != ZoneKind::Battlefield
                || limit_reached(cx, id, &chars, index, def)
                || !can_pay_other(cx, id, player, &def.cost)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            // "Activate only if ..." (CR 602.5b), as Fanatic of Rhonas's ferocious mana.
            if let Some(condition) = def.condition
                && !condition(cx, &Asker { source: id, controller: player })
            {
                continue;
            }
            // A mana ability with a mana cost of its own (Sunken Ruins' {U/B}) is not offered when that cost cannot be
            // paid by the other sources, its own {T} not among them (CR 605.3a, 602.2b, 609.4b).
            if def.cost.mana.x == 0 && def.cost.mana.mana_value() > 0 {
                let a = available_without(cx, player, def.cost.tap.then_some(id));
                let mut cost = def.cost.mana;
                if static_rules::spend_any_color_for(cx, player, id) {
                    cost = ManaCost { generic: cost.mana_value().min(255) as u8, ..ManaCost::ZERO };
                }
                if !a.covers_for(&cost, &ManaPayment::Ability { source: chars.types, subtypes: chars.subtypes }) {
                    continue;
                }
            }
            let mut option = ActionOption::on(ActionKind::ActivateAbility, id, index as u8);
            option.restriction = def.restricted.map(|r| restriction_from(cx, id, r));
            if let Some(p) = def.cost.permanents {
                let asker = Asker { source: id, controller: player };
                option.target_candidates = cx
                    .permanents_of(player)
                    .into_iter()
                    .filter(|&o| pays_permanents(cx, &def.cost, &p, &asker, o))
                    .map(Target::Object)
                    .collect();
                option.targets_required = p.count;
            }
            if def.colors_from_sacrifice {
                // The types any land it could sacrifice could produce; the activation then names one of that land's.
                let (mut colors, mut colorless) = (ColorSet::NONE, false);
                for t in &option.target_candidates {
                    if let Some(land) = t.object() {
                        let (c, n) = producible_types(cx, land);
                        colors = ColorSet(colors.0 | c.0);
                        colorless |= n;
                        option.names_nothing |= c.is_empty() && !n;
                    }
                }
                option.color_choices = colors;
                if colorless {
                    option.mana = Some(Mana::C);
                }
            } else if def.any_color {
                option.color_choices = match def.colors_from {
                    Some(from) => from(cx, &Asker { source: id, controller: player }),
                    None if def.colors.is_empty() => ColorSet(0b11111),
                    None => def.colors,
                };
                // With no color to choose it is still offered, and adds no mana (CR 106.5).
            }
            options.push(option);
        }
    }
    // Mana abilities the player has of their own, from a resolved spell or ability (CR 605.1a, 611.2a): offered whenever they
    // could activate a mana ability. Numbered in the order of the effects that gave them.
    for (n, effect) in cx.s().effects.iter().filter(|e| matches!(e.kind, EffectKind::PlayerManaAbility(_)) && e.controller == player).enumerate() {
        let EffectKind::PlayerManaAbility(ability) = effect.kind else { continue };
        if let Some(def) = activated(cx, ability)
            && can_pay_other(cx, effect.source, player, &def.cost)
        {
            options.push(ActionOption::on(ActionKind::ActivateAbility, effect.source, PLAYER_MANA_ABILITY + n as u8));
        }
    }
    // Mana abilities that function from the hand (CR 113.6, 605): Elvish Spirit Guide's exile-this-card.
    for &id in cx.s().zone(Zone::hand(player)) {
        let def = cx.card(cx.s().obj(id).card);
        for (index, ability) in def.abilities.iter().enumerate() {
            let AbilityDef::Activated(a) = ability else { continue };
            if !a.is_mana_ability()
                || a.from != ZoneKind::Hand
                || !can_pay_other(cx, id, player, &a.cost)
                || static_rules::activation_barred(cx, &forbidders, id, a)
            {
                continue;
            }
            if let Some(condition) = a.condition
                && !condition(cx, &Asker { source: id, controller: player })
            {
                continue;
            }
            options.push(ActionOption {
                restriction: a.restricted.map(|r| restriction_from(cx, id, r)),
                ..ActionOption::on(ActionKind::ActivateAbility, id, index as u8)
            });
        }
    }
}

/// The candidates for one target slot (CR 601.2c), in a stable order: players in turn order, then objects on the
/// battlefield, the stack and in graveyards.
pub fn target_candidates(cx: &Cx, asker: &Asker, spec: &TargetSpec) -> Vec<Target> {
    target_candidates_given(cx, asker, spec, &[], Target::None, None)
}

/// The candidates for a slot of a spell as if its gift were promised (`Query::gift_assumed`), without promising it.
fn target_candidates_if_promised(cx: &Cx, asker: &Asker, spec: &TargetSpec) -> Vec<Target> {
    candidates(cx, asker, spec, &[], Target::None, None, true)
}

/// The candidates for a slot after the slots before it chose `earlier` (CR 601.2c); `slot_player` is the opponent the slot
/// is for when it is one of a `TargetSpec::EACH_OPPONENT`.
pub fn target_candidates_given(
    cx: &Cx,
    asker: &Asker,
    spec: &TargetSpec,
    earlier: &[Target],
    subject: Target,
    slot_player: Option<PlayerId>,
) -> Vec<Target> {
    candidates(cx, asker, spec, earlier, subject, slot_player, false)
}

fn candidates(cx: &Cx, asker: &Asker, spec: &TargetSpec, earlier: &[Target], subject: Target, slot_player: Option<PlayerId>, gift: bool) -> Vec<Target> {
    let s = cx.s();
    let cx = &crate::resolve::Given { q: cx, earlier, subject, slot_player, gift };
    let mut out = Vec::new();
    for p in s.seats() {
        let t = Target::Player(p);
        if (spec.filter)(cx, asker, t) && cx.can_target(asker, t) {
            out.push(t);
        }
    }
    let mut zones = vec![Zone::BATTLEFIELD, Zone::STACK];
    zones.extend(s.seats().map(Zone::graveyard));
    for zone in zones {
        for &id in s.zone(zone) {
            // A spell cannot target itself (CR 115.5); an ability may target its own source.
            if id == asker.source && zone.is(ZoneKind::Stack) {
                continue;
            }
            let t = Target::Object(id);
            if (spec.filter)(cx, asker, t) && cx.can_target(asker, t) {
                out.push(t);
            }
        }
    }
    // Abilities on the stack (CR 115.1), oldest first.
    for item in s.stack.iter().filter(|i| i.kind != StackKind::Spell) {
        let t = Target::Ability(item.serial);
        if (spec.filter)(cx, asker, t) && cx.can_target(asker, t) {
            out.push(t);
        }
    }
    out
}

/// Whether the targets are announced with the action (a single slot for one target), rather than asked slot by
/// slot once the spell or ability has begun to be put on the stack, as the C# engine does for anything else.
pub fn targets_with_action(specs: &[TargetSpec]) -> bool {
    specs.len() == 1 && specs[0].max == 1
}

/// Whether an activated ability's targets are announced with the action: one slot of one target, unless they wait for X
/// (`ActivatedDef::late_targets`, CR 601.2b-c by way of 602.2b).
fn targets_with_activation(def: &ActivatedDef) -> bool {
    !def.late_targets && targets_with_action(def.targets)
}

/// Whether every required slot has at least its minimum number of legal targets (CR 601.2c): fewer is no legal announcement, so the
/// spell or ability is not offered.
pub(crate) fn slots_satisfiable(cx: &Cx, asker: &Asker, specs: &[TargetSpec]) -> bool {
    specs.iter().all(|spec| !spec.required() || target_candidates(cx, asker, spec).len() >= spec.min as usize)
}

/// Fills in an option's targets; false when a required slot has no candidate, so the option is not offered.
fn with_targets(cx: &Cx, asker: &Asker, specs: &[TargetSpec], option: &mut ActionOption) -> bool {
    if !targets_with_action(specs) {
        return slots_satisfiable(cx, asker, specs);
    }
    for spec in specs {
        let candidates = target_candidates(cx, asker, spec);
        if candidates.is_empty() && spec.required() {
            return false;
        }
        option.targets_required += spec.max;
        option.targets_optional |= spec.min == 0;
        for c in candidates {
            if !option.target_candidates.contains(&c) {
                option.target_candidates.push(c);
            }
        }
    }
    true
}

/// The modes of `modal` that could be chosen now (CR 700.2a): those every one of whose target slots has a legal
/// target, or is optional.
pub(crate) fn eligible_modes(cx: &Cx, asker: &Asker, modal: &ModalDef) -> Vec<u8> {
    (0..modal.modes.len() as u8)
        .filter(|&i| {
            slots_satisfiable(cx, asker, modal.modes[i as usize].targets)
                && modal.allowed.is_none_or(|allowed| allowed(cx, asker, i))
        })
        .collect()
}

/// Whether enough modes could be chosen for the spell or ability to be announced at all (CR 700.2a).
fn modes_possible(cx: &Cx, asker: &Asker, modal: &ModalDef) -> bool {
    // A mode that may be chosen more than once is enough for any number (CR 700.2d).
    eligible_modes(cx, asker, modal).len() >= if modal.repeat { 1 } else { modal.min.max(1) as usize }
}

/// Whether a spell's targets are announced with the action: one slot of one target, on a spell that is not modal and
/// does not choose its target after its kicker or X (CR 601.2b-c), nor how many by its kicker.
fn targets_with_cast(def: &'static CardDef) -> bool {
    def.modal.is_none()
        && !def.late_targets
        && def.gift.is_none()
        && def.divide.is_none()
        && targets_with_action(def.spell_targets())
        && def.spell_targets()[0].if_kicked.is_none()
}

/// Fills in a spell option's targets, or checks that its modes and targets could be announced: a modal spell needs
/// enough modes, one whose target waits on its kicker is offered if a target exists either way.
fn offer_spell(cx: &Cx, asker: &Asker, def: &'static CardDef, option: &mut ActionOption) -> bool {
    if let Some(modal) = def.modal {
        return modes_possible(cx, asker, modal);
    }
    let specs = def.spell_targets();
    // A gift is promised before the targets (CR 601.2b-c, 702.174m), so what may be targeted can depend on it: it is
    // offered if every required slot has a target without the gift, or with it.
    if def.late_targets || def.gift.is_some() || def.divide.is_some() {
        let met = |if_promised: bool| {
            specs.iter().all(|spec| {
                !spec.required()
                    || (if if_promised { target_candidates_if_promised(cx, asker, spec) } else { target_candidates(cx, asker, spec) }).len() >= spec.min as usize
            })
        };
        return def.kicker.is_some() || met(false) || (def.gift.is_some() && met(true));
    }
    // "If this spell was kicked, instead choose ..." (CR 702.33a, 601.2c): the targets wait for the kicker, and it may be
    // cast if it could be announced either unkicked or, with a kicker to pay, kicked.
    if def.kicker.is_some() && specs.iter().any(|spec| spec.if_kicked.is_some()) {
        return slots_satisfiable(cx, asker, specs) || slots_satisfiable(cx, asker, &specs.iter().map(|spec| spec.when_kicked(1)).collect::<Vec<_>>());
    }
    with_targets(cx, asker, specs, option)
}

/// Whether a cast is worth offering, its target candidates kept to those it could be paid for. The total cost is determined
/// once the target is announced (CR 601.2c, 601.2f): where it is announced with the cast and something could change the
/// cost, the cast is judged for each candidate (`covered`), and only those it could be paid for are offered; otherwise it is
/// judged once, with no target.
fn offer_costed(cx: &Cx, a: &Available, asker: &Asker, def: &'static CardDef, option: &mut ActionOption, covered: impl Fn(&[Target]) -> bool) -> bool {
    if targets_with_cast(def) && (!a.changers.is_empty() || !def.own_cost_changes.is_empty()) {
        if !offer_spell(cx, asker, def, option) {
            return false;
        }
        let none_covered = option.targets_optional && covered(&[]);
        option.target_candidates.retain(|&t| covered(&[t]));
        !option.target_candidates.is_empty() || none_covered
    } else {
        covered(&[]) && offer_spell(cx, asker, def, option)
    }
}

/// Whether the life the cost changes add to casting `card` as `def` with these targets (CR 118.8, 601.2f: "cost an
/// additional 3 life") could be paid, with the life its own additional cost asks, from its caster's life total (CR 119.4,
/// 118.3). Advice, as `least_cost` is.
fn added_life_payable(cx: &Cx, a: &Available, player: PlayerId, card: ObjectId, def: &CardDef, targets: &[Target]) -> bool {
    if a.changers.is_empty() && def.own_cost_changes.is_empty() {
        return true;
    }
    let (_, added) = change_total(cx, &a.changers, player, Costed::Spell(card), ManaCost::ZERO, face_of(cx, card, def), targets);
    added == 0 || cx.s().player(player).life >= added as i32 + def.additional_cost.map_or(0, |c| c.life as i32)
}

/// The face `def` is of the card `card`, where it is one of its faces rather than the card itself (an Adventure, an Omen, a
/// back face, a bestowed Aura, a half): what the cost changes judge it as (CR 715.4, 720.3a, 709.3, 712.8a).
fn face_of(cx: &Cx, card: ObjectId, def: &CardDef) -> Option<CardId> {
    let whole = cx.card(cx.s().obj(card).card);
    let faces = [whole.adventure, whole.omen, whole.back, whole.bestow, whole.split.map(|s| s.0), whole.split.map(|s| s.1)];
    faces.into_iter().flatten().find(|&f| std::ptr::eq(f, def)).map(|f| cx.card_id(f))
}

/// A cost with the static abilities that change it applied (CR 601.2f, 118.7): increases first, then reductions, a
/// reduction taking nothing below zero generic mana (CR 118.7c). `targets` are those announced for the spell or ability
/// (CR 601.2c), which the total cost is determined after.
pub fn apply_cost_changes(cx: &Cx, player: PlayerId, costed: Costed, cost: ManaCost, targets: &[Target]) -> ManaCost {
    change_with(cx, &cost_change_sources(cx), player, costed, cost, None, targets)
}

/// `apply_cost_changes`, and the life the changes add to the total cost as additional costs (CR 118.8, 601.2f: "cost an
/// additional 3 life to cast").
pub fn apply_cost_changes_total(cx: &Cx, player: PlayerId, costed: Costed, cost: ManaCost, targets: &[Target]) -> (ManaCost, u32) {
    change_total(cx, &cost_change_sources(cx), player, costed, cost, None, targets)
}

/// The permanents on the battlefield whose cards change costs: found from the board, not from the card table (which
/// holds every card), and empty, so unallocated, in nearly every game.
fn cost_change_sources(cx: &Cx) -> Vec<ObjectId> {
    cx.s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            cx.cards().cost_changes[cx.cards().current(obj).0 as usize] && !cx.lost_all_abilities(id)
        })
        .collect()
}

/// The game as the cost changes of a spell read it when the spell is judged before it is cast: the card in hand has the
/// characteristics of the face it would be cast as, the only spell it would be on the stack (CR 715.4, 709.3, 712.8a).
struct AsFace<'a> {
    cx: &'a Cx<'a>,
    spell: ObjectId,
    face: CardId,
}

impl Query for AsFace<'_> {
    fn state(&self) -> &GameState {
        self.cx.s()
    }

    fn card(&self, id: CardId) -> &'static CardDef {
        self.cx.card(id)
    }

    fn chars(&self, id: ObjectId) -> Chars {
        if id == self.spell { layers::chars_as_face(self.cx.cards(), self.cx.s(), &self.cx.g.log, id, self.face) } else { self.cx.chars(id) }
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.cx.card_id(def)
    }

    fn lost_all_abilities(&self, id: ObjectId) -> bool {
        self.cx.lost_all_abilities(id)
    }

    fn events(&self) -> &[Record] {
        self.cx.events()
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        self.cx.can_target(asker, target)
    }
}

/// What activating the ability `index` of `source` (numbered as its options are) would cost `player` with each of
/// `candidates` as its one target, the total as it is determined once the target is announced (CR 601.2c, 601.2f,
/// 602.2b): for the menu row of an action whose cost depends on the target it would be announced with. Empty when
/// nothing could change that cost (no cost changer on the battlefield, none of the ability's or card's own), which is
/// nearly always, or when it is no activated ability with a mana cost.
pub fn activation_costs(game: &crate::Game, player: PlayerId, source: ObjectId, index: u8, candidates: &[Target]) -> Vec<(ManaCost, u32)> {
    let Some(ability) = game.chars(source).abilities().nth(index as usize) else { return Vec::new() };
    let AbilityDef::Activated(def) = game.cards.ability(ability) else { return Vec::new() };
    let changers = game.state.battlefield().iter().any(|&id| {
        game.cards.cost_changes[game.cards.current(game.state.obj(id)).0 as usize] && !game.lost_all_abilities(id)
    });
    let obj = game.state.obj(source);
    let own = !game.cards.get(obj.copy.map_or(obj.card, |c| c.card)).own_cost_changes.is_empty();
    if def.special_action || def.cost.mana.x > 0 || !(changers || own || def.cost_change.is_some()) {
        return Vec::new();
    }
    // The rules read the game through a frame's view; a copy of the position (no tree) is enough to ask it.
    let mut copy = crate::Game { state: game.state.clone(), log: game.log.clone(), tree: crate::tree::Tree::new(), at: crate::tree::NodeId::ROOT, cards: game.cards, verify: false };
    let cx = Cx { g: &mut copy, frame: usize::MAX, cursor: 0 };
    candidates.iter().map(|&t| apply_cost_changes_total(&cx, player, Costed::Ability(source, ability), def.cost.mana, &[t])).collect()
}

/// What casting `card` by the cast option `index` (its mana cost, 0, or a way a permanent gives) would cost `player` with
/// each of `candidates` as its one target, the cost changes applied as they are once the target is announced (CR 601.2c,
/// 601.2f): its mana, and the life the changes add (CR 118.8). For the menu row of a cast whose cost depends on its target.
/// Empty when nothing could change it (no cost changer on the battlefield, none of the card's own), which is nearly always,
/// or for any other way of casting it.
pub fn cast_costs(game: &crate::Game, player: PlayerId, card: ObjectId, index: u8, candidates: &[Target]) -> Vec<(ManaCost, u32)> {
    let def = game.cards.get(game.state.obj(card).card);
    let changers = game.state.battlefield().iter().any(|&id| {
        game.cards.cost_changes[game.cards.current(game.state.obj(id)).0 as usize] && !game.lost_all_abilities(id)
    });
    if !(changers || !def.own_cost_changes.is_empty()) {
        return Vec::new();
    }
    let base = match (index, granted_slot(index)) {
        (0, _) => def.cost,
        (_, Some(_)) => match granted_cast_cost(game, player, card, index) {
            Some(cost) => cost.mana,
            None => return Vec::new(),
        },
        _ => return Vec::new(),
    };
    let mut copy = crate::Game { state: game.state.clone(), log: game.log.clone(), tree: crate::tree::Tree::new(), at: crate::tree::NodeId::ROOT, cards: game.cards, verify: false };
    let cx = Cx { g: &mut copy, frame: usize::MAX, cursor: 0 };
    let face = granted_slot(index).and_then(|(_, face)| faces_to_cast(def).into_iter().find(|&(f, _)| f == face)).filter(|&(f, _)| f != GRANTED_FACE_WHOLE).map(|(_, d)| cx.card_id(d));
    candidates.iter().map(|&t| change_total(&cx, &cost_change_sources(&cx), player, Costed::Spell(card), base, face, &[t])).collect()
}

/// What a cast a permanent grants (`index`, a granted way's number) costs `player` for `card` (`granted_cast_cost`), and its
/// mana as the cost changes in play leave it before any target is announced (CR 601.2f): for the menu row of a cast without
/// paying the mana cost, which says what it still costs ({1} more under Thalia).
pub fn granted_cast_total(game: &crate::Game, player: PlayerId, card: ObjectId, index: u8) -> Option<(Cost, ManaCost)> {
    let cost = granted_cast_cost(game, player, card, index)?;
    let (_, face) = granted_slot(index)?;
    let (_, face_def) = faces_to_cast(game.cards.get(game.state.obj(card).card)).into_iter().find(|&(f, _)| f == face)?;
    let mut copy = crate::Game { state: game.state.clone(), log: game.log.clone(), tree: crate::tree::Tree::new(), at: crate::tree::NodeId::ROOT, cards: game.cards, verify: false };
    let cx = Cx { g: &mut copy, frame: usize::MAX, cursor: 0 };
    let face_id = (face != GRANTED_FACE_WHOLE).then(|| cx.card_id(face_def));
    let mana = change_with(&cx, &cost_change_sources(&cx), player, Costed::Spell(card), cost.mana, face_id, &[]);
    Some((cost, mana))
}

/// What a cast a permanent grants (CR 604, 118.9) costs `player` for `card`, by the option's way number: the way's own
/// cost, read for the face it casts as the offer and the cast read it (CR 709.3a), before cost changes. For the menu row,
/// since a way's text need not say it (Iroh's "the flashback cost is equal to that card's mana cost"). `None` for any other
/// way, or one the permanent no longer gives.
pub fn granted_cast_cost(game: &crate::Game, player: PlayerId, card: ObjectId, way: u8) -> Option<Cost> {
    let (k, face) = granted_slot(way)?;
    let (granter, controller, rule) = *granted_ways_of(game.cards, &game.state, &game.log).get(k)?;
    let (_, face_def) = faces_to_cast(game.cards.get(game.state.obj(card).card)).into_iter().find(|&(f, _)| f == face)?;
    let mut copy = crate::Game { state: game.state.clone(), log: game.log.clone(), tree: crate::tree::Tree::new(), at: crate::tree::NodeId::ROOT, cards: game.cards, verify: false };
    let cx = Cx { g: &mut copy, frame: usize::MAX, cursor: 0 };
    let view = AsFace { cx: &cx, spell: card, face: cx.card_id(face_def) };
    let q: &dyn Query = if face == GRANTED_FACE_WHOLE { &cx } else { &view };
    (rule.cost)(q, &Asker { source: granter, controller }, card, player)
}

/// The cost changes the object `id` carries itself (CR 601.2f): its card's, or a copy's.
fn own_cost_changes(cx: &Cx, id: ObjectId) -> &'static [CostChange] {
    let obj = cx.s().obj(id);
    cx.card(obj.copy.map_or(obj.card, |c| c.card)).own_cost_changes
}

fn change_with(
    cx: &Cx,
    sources: &[ObjectId],
    player: PlayerId,
    costed: Costed,
    cost: ManaCost,
    face: Option<CardId>,
    targets: &[Target],
) -> ManaCost {
    change_total(cx, sources, player, costed, cost, face, targets).0
}

/// The cost changes applied (`change_with`), and the life they add (CR 118.8, 601.2f).
fn change_total(
    cx: &Cx,
    sources: &[ObjectId],
    player: PlayerId,
    costed: Costed,
    mut cost: ManaCost,
    face: Option<CardId>,
    targets: &[Target],
) -> (ManaCost, u32) {
    let view;
    let q: &dyn Query = match (face, costed) {
        (Some(face), Costed::Spell(spell)) => {
            view = AsFace { cx, spell, face };
            &view
        }
        _ => cx,
    };
    // The thing costed is its own source for the changes it carries itself (affinity, channel's reduction).
    let (id, ability) = match costed {
        Costed::Spell(id) => (id, None),
        // A special action is no activated ability (CR 116.1): no discount on abilities, and no ability's own change.
        Costed::SpecialAction(id, _) => (id, None),
        Costed::Ability(id, ability) => (id, Some(ability)),
    };
    let own: &[CostChange] = match face {
        Some(face) => cx.card(face).own_cost_changes,
        None => own_cost_changes(cx, id),
    };
    // The ability's own change ("this ability costs ... less", power-up), asked for this ability alone.
    let ability_own = ability.and_then(|a| activated(cx, a)).and_then(|d| d.cost_change);
    if sources.is_empty() && own.is_empty() && ability_own.is_none() {
        return (cost, 0);
    }
    let mut more = 0u32;
    let mut less = ManaCost::ZERO;
    let mut life = 0u32;
    let mut add = |delta: CostDelta| {
        more += delta.more as u32;
        less = less.plus(&delta.less);
        life += delta.life as u32;
    };
    for change in own {
        add(change(q, id, player, costed, targets));
    }
    if let Some(change) = ability_own {
        add(change(q, id, player, costed, targets));
    }
    let mut discount = 0u32;
    for &source in sources {
        let obj = cx.s().obj(source);
        let def = cx.card(cx.cards().current(obj));
        for change in def.cost_changes {
            add(change(q, source, player, costed, targets));
        }
        if let Some(ability) = ability {
            for d in def.ability_discounts {
                discount += d(q, source, player, id, ability) as u32;
            }
        }
    }
    // Increases, then reductions (CR 601.2f). Reductions of generic and colored mana commute, so the order the player
    // could apply them in changes nothing.
    cost.generic = (cost.generic as u32 + more).min(255) as u8;
    cost = reduce(cost, &less);
    // "Can't reduce the mana in that cost to less than one mana": applied to the cost the other changes left (CR 601.2f).
    if discount > 0 {
        let room = cost.mana_value().saturating_sub(1);
        cost.generic -= discount.min(room).min(cost.generic as u32) as u8;
    }
    (cost, life)
}

/// A cost reduced by an amount of mana (CR 118.7): its generic part takes generic mana off only (118.7a); each colored
/// or colorless symbol takes off one mana of that type, and one the cost has none of (118.7b), or has fewer of (118.7c-d),
/// takes off generic mana instead; a Phyrexian symbol takes off mana of its color (118.7f). Nothing goes below zero
/// (CR 601.2f). The reduced symbols are the cost's plain ones of that type: no cost a colored reduction meets has a
/// hybrid or Phyrexian symbol of its color yet.
fn reduce(mut cost: ManaCost, by: &ManaCost) -> ManaCost {
    // A hybrid symbol in a reduction is split by the payer as the reduction is applied (CR 118.7e), a choice not built:
    // refused in every build rather than reduced some other way.
    assert!(
        by.hybrid.iter().all(Option::is_none),
        "a cost reduction by hybrid mana needs the payer's choice of half (CR 118.7e), which is not built"
    );
    let mut excess = by.generic as u32;
    for i in 0..6 {
        let mut n = by.symbols[i] as u32;
        if i < 5 {
            n += by.phyrexian[i] as u32;
        }
        let off = n.min(cost.symbols[i] as u32);
        cost.symbols[i] -= off as u8;
        excess += n - off;
    }
    cost.generic = (cost.generic as u32).saturating_sub(excess) as u8;
    cost
}

/// The least mana a spell could be cast for with this base cost, for the advice that a cast that cannot be paid is
/// not offered (CR 601.2, 118.3): X at 0, no kicker, the cheapest modes it must choose, Phyrexian symbols paid with
/// life where the life total allows. `targets` are the candidate it would be announced with, where its target is announced
/// with the cast, or none (CR 601.2c, 601.2f).
fn least_cost(cx: &Cx, available: &Available, player: PlayerId, card: ObjectId, def: &CardDef, base: ManaCost, targets: &[Target]) -> ManaCost {
    let mut cost = base;
    if let Some(extra) = def.additional_cost {
        cost = add_cost(cost, extra.mana);
    }
    // CR 601.2b: an additional cost that is one of several ("behold a Jace or pay {1}") costs at least the mana of the
    // cheapest of them that can be paid otherwise.
    if let Some(cheapest) = def
        .additional_choice
        .iter()
        .filter(|c| can_pay_other(cx, card, player, &c.cost))
        .map(|c| c.cost.mana)
        .min_by_key(|m| m.mana_value())
    {
        cost = add_cost(cost, cheapest);
    }
    if let Some(modal) = def.modal {
        let mut costs: Vec<ManaCost> = modal.modes.iter().map(|m| m.cost).collect();
        costs.sort_by_key(|c| c.mana_value());
        // One mode to choose: the cheapest the player could pay for - Three Steps Ahead's {3} when its {1}{U} would need
        // a second blue.
        if modal.min <= 1
            && let Some(i) = costs.iter().position(|&c| available.covers_for(&finish(cx, available, player, card, def, add_cost(cost, c), targets), &spell_kind(cx, available, card, def)))
        {
            costs.swap(0, i);
        }
        for c in costs.into_iter().take(modal.min.max(1) as usize) {
            cost = add_cost(cost, c);
        }
    }
    finish(cx, available, player, card, def, cost, targets)
}

/// `least_cost`'s Phyrexian symbols paid with life where the life total allows, and the cost changes applied.
fn finish(cx: &Cx, available: &Available, player: PlayerId, card: ObjectId, def: &CardDef, mut cost: ManaCost, targets: &[Target]) -> ManaCost {
    let mut life = cx.s().player(player).life;
    for (i, n) in cost.phyrexian.into_iter().enumerate() {
        for _ in 0..n {
            if life > 2 {
                life -= 2;
            } else {
                cost.symbols[Color::ALL[i].mana() as usize] += 1;
            }
        }
    }
    cost.phyrexian = [0; 5];
    // A face cast from the hand (an Adventure, an Omen, a half, a back face) is judged as that face (CR 715.4, 720.3a,
    // 709.3, 712.8a).
    change_with(cx, &available.changers, player, Costed::Spell(card), cost, face_of(cx, card, def), targets)
}

/// Whether `player` could pay `cost` from the mana pool and what the mana abilities they could activate add (advice,
/// CR 601.2g): a mana ability whose yield is not stated is taken to be able to pay anything.
///
/// `source` is what the cost is for: a spell on the stack may be paid with the restricted mana its kind allows, anything
/// else with the restricted mana an ability may spend (CR 106.6).
pub fn can_afford(cx: &Cx, player: PlayerId, source: ObjectId, cost: &ManaCost) -> bool {
    if cost.mana_value() == 0 {
        return true;
    }
    let a = available(cx, player);
    let chars = cx.chars(source);
    if cx.s().obj(source).zone.is(ZoneKind::Stack) {
        a.covers_for(cost, &spell_payment(cx, source, a.names_subtypes()))
    } else {
        a.covers_for(cost, &ManaPayment::Ability { source: chars.types, subtypes: chars.subtypes })
    }
}

/// The mana a player has to pay with: the pool and what their mana abilities add, summed once for every cost judged.
pub struct Available {
    fixed: [u8; 6],
    /// Mana of a type to be chosen, by the mana types it may be (a bit for each `Mana`, W U B R G C): a land with
    /// "{T}: Add {W}" and "{T}: Add {R}" is one mana that is white or red, never blue.
    flex: [u32; 64],
    /// Some mana ability's yield is not stated: anything may be payable.
    unknown: bool,
    /// Mana that the mana abilities with a mana cost of their own (Chromatic Star's {1}) must be paid first: it is
    /// spent on them, so it is not there to pay for the cost they are for.
    consume: u32,
    /// The permanents that change what things cost.
    changers: Vec<ObjectId>,
    /// Restricted mana the pool and the mana sources hold, one entry per restriction (CR 106.6): its mana of stated kinds,
    /// and its mana of any one color a mana ability adds (Great Hall of the Biblioplex, Delighted Halfling). Each pays
    /// only what `Available::covers_for` says it may. Mana that pays anything (`GainsHaste`) is in `fixed`.
    restricted: Vec<(ManaRestriction, [u8; 6], u32)>,
}

/// Adds restricted mana to the entry of its restriction in `Available::restricted`.
fn add_available(list: &mut Vec<(ManaRestriction, [u8; 6], u32)>, restriction: ManaRestriction, fixed: [u8; 6], flex: u32) {
    let at = match list.iter().position(|(r, ..)| *r == restriction) {
        Some(at) => at,
        None => {
            list.push((restriction, [0; 6], 0));
            list.len() - 1
        }
    };
    for m in 0..6 {
        list[at].1[m] = list[at].1[m].saturating_add(fixed[m]);
    }
    list[at].2 += flex;
}

pub fn available(cx: &Cx, player: PlayerId) -> Available {
    available_without(cx, player, None)
}

/// What `player` has to pay with, `skip` apart: a permanent whose own {T} is in the cost being paid cannot also be
/// tapped for mana to pay it (CR 602.2b, 605.3b).
fn available_without(cx: &Cx, player: PlayerId, skip: Option<ObjectId>) -> Available {
    let mut fixed = cx.s().player(player).pool.0;
    let mut restricted: Vec<(ManaRestriction, [u8; 6], u32)> = Vec::new();
    for (restriction, pool) in cx.s().restricted_of(player) {
        if restriction == ManaRestriction::GainsHaste {
            // Mana that pays anything is not a limit (CR 106.6): it is spent as ordinary mana is.
            for m in 0..6 {
                fixed[m] += pool.0[m];
            }
        } else {
            add_available(&mut restricted, restriction, pool.0, 0);
        }
    }
    let mut flex = [0u32; 64];
    let mut consume: u32 = 0;
    // Mana abilities of the player's own (CR 605.1a): paid with life, as often as the life total allows (CR 119.4).
    for effect in cx.s().effects.iter().filter(|e| matches!(e.kind, EffectKind::PlayerManaAbility(_)) && e.controller == player) {
        let EffectKind::PlayerManaAbility(ability) = effect.kind else { continue };
        if let Some(def) = activated(cx, ability)
            && def.cost.life > 0
        {
            let times = (cx.s().player(player).life.max(0) as u32 / def.cost.life as u32).min(100);
            for m in 0..6 {
                fixed[m] = fixed[m].saturating_add((def.adds.0[m] as u32 * times).min(255) as u8);
            }
        }
    }
    let forbidders = static_rules::activation_forbidders(cx);
    for &id in cx.s().battlefield() {
        if cx.s().obj(id).controller != player || skip == Some(id) {
            continue;
        }
        let chars = cx.chars(id);
        let mut only: Option<&ActivatedDef> = None;
        let mut count = 0;
        // The most any one ability adds, for the abilities that are not restricted and for each kind of restriction: they
        // are alternatives (each taps the permanent), so only the widest of each counts (CR 605.3b).
        let mut widest_free = 0u32;
        let mut widest: Vec<(ManaRestriction, u32)> = Vec::new();
        let mut spent = 0u32;
        // The one ability that is not restricted, when there is exactly one: its yield is stated, not any color.
        let mut unrestricted: Option<&ActivatedDef> = None;
        let mut unrestricted_count = 0;
        // The mana types the abilities that are not restricted could add between them.
        let mut types = 0u8;
        for (index, ability) in chars.abilities().enumerate() {
            let Some(def) = activated(cx, ability) else { continue };
            if !def.is_mana_ability()
                || def.from != ZoneKind::Battlefield
                || limit_reached(cx, id, &chars, index, def)
                || !can_pay_other(cx, id, player, &def.cost)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            // "Activate only if ..." (CR 602.5b): a Verge's second color is not there without its land type.
            if let Some(condition) = def.condition
                && !condition(cx, &Asker { source: id, controller: player })
            {
                continue;
            }
            count += 1;
            only = Some(def);
            spent = spent.max(def.cost.mana.mana_value());
            let adds = def.adds.0.iter().map(|&n| n as u32).sum::<u32>() + def.adds_any as u32;
            match def.restricted {
                Some(ManaRestriction::GainsHaste) | None => {
                    unrestricted = Some(def);
                    unrestricted_count += 1;
                    widest_free = widest_free.max(adds);
                    types |= yield_types(cx, id, player, def);
                }
                Some(kind) => {
                    let kind = restriction_from(cx, id, kind);
                    match widest.iter_mut().find(|(r, _)| *r == kind) {
                        Some((_, n)) => *n = (*n).max(adds),
                        None => widest.push((kind, adds)),
                    }
                }
            }
            if def.adds.0.iter().all(|&n| n == 0) && def.adds_any == 0 {
                return Available { fixed, flex, consume: 0, unknown: true, changers: cost_change_sources(cx), restricted };
            }
        }
        if count > 0 {
            consume += spent;
        }
        match (count, only) {
            (0, _) => {}
            (1, Some(def)) if def.restricted == Some(ManaRestriction::GainsHaste) => {
                for m in 0..6 {
                    fixed[m] += def.adds.0[m];
                }
                flex[any_types(cx, id, player, def) as usize] += def.adds_any as u32;
            }
            (1, Some(def)) if let Some(kind) = def.restricted => {
                add_available(&mut restricted, restriction_from(cx, id, kind), def.adds.0, def.adds_any as u32);
            }
            (1, Some(def)) => {
                for m in 0..6 {
                    fixed[m] += def.adds.0[m];
                }
                flex[any_types(cx, id, player, def) as usize] += def.adds_any as u32;
            }
            _ => {
                match unrestricted {
                    Some(def) if unrestricted_count == 1 => {
                        for m in 0..6 {
                            fixed[m] += def.adds.0[m];
                        }
                        flex[any_types(cx, id, player, def) as usize] += def.adds_any as u32;
                    }
                    _ => flex[types as usize] += widest_free,
                }
                for (kind, n) in widest {
                    add_available(&mut restricted, kind, [0; 6], n);
                }
            }
        }
    }
    Available { fixed, flex, consume, unknown: false, changers: cost_change_sources(cx), restricted }
}

/// The spell a card or face is when cast, as far as the mana limited to some spells is concerned (CR 106.6): advice,
/// which the payment judges again on the stack. Its subtypes are the face's, a changeling face's every creature type
/// (CR 702.73a), and for the whole card those effects give it where it is. They are only worked out when some mana at hand
/// is restricted by subtype.
///
/// Its mana value is the face's with X at 0 (CR 202.3e), or for the back face of a nonmodal double-faced card its front
/// face's (CR 712.8c); `x`, whether its mana cost has {X} in it, says it could be more (`Available::covers_for`).
fn spell_kind(cx: &Cx, a: &Available, card: ObjectId, def: &CardDef) -> ManaPayment {
    let whole = cx.card(cx.s().obj(card).card);
    let back = !def.has_cost && whole.back.is_some_and(|b| std::ptr::eq(b, def));
    let mana_value = if back { whole.cost.mana_value() } else { def.cost.mana_value() };
    let (types, legendary, x) = (def.types, def.supertypes.legendary, def.cost.x > 0);
    // Cast from where the card is (CR 601.2a).
    let from = cx.s().obj(card).zone.kind();
    if !a.names_subtypes() {
        return ManaPayment::Spell { types, subtypes: SubtypeSet::NONE, legendary, mana_value, x, from };
    }
    let mut subtypes = def.subtypes;
    if std::ptr::eq(def, cx.card(cx.s().obj(card).card)) {
        subtypes = subtypes.union(cx.chars(card).subtypes);
    }
    if def.keywords.has(Keyword::Changeling) && (def.types.has(CardType::Creature) || def.types.has(CardType::Kindred)) {
        subtypes = Subtype::CREATURE_TYPES.into_iter().fold(subtypes, SubtypeSet::with);
    }
    ManaPayment::Spell { types, subtypes, legendary, mana_value, x, from }
}

impl Available {
    /// Whether `cost` could be paid for `payment` (casting a spell of a kind, activating an ability of a source): the
    /// restricted mana that may be spent on it counts (CR 106.6, 601.2g, 602.2b).
    /// Whether some of its restricted mana is limited by the spell's subtypes.
    fn names_subtypes(&self) -> bool {
        self.restricted.iter().any(|(r, ..)| r.names_subtypes())
    }

    pub fn covers_for(&self, cost: &ManaCost, payment: &ManaPayment) -> bool {
        if self.covers_as(cost, payment) {
            return true;
        }
        // A spell whose X is still to be announced (CR 601.2b) has the larger mana value the larger X is (CR 202.3e):
        // mana for spells of a mana value or greater may pay for it at an X that brings it there (Ashling, Rimebound's
        // mana and a {X}{R} spell), each X costing its {X} symbols more.
        let ManaPayment::Spell { types, subtypes, legendary, mana_value, x, from } = *payment else { return false };
        let Some(most) = self.restricted.iter().filter_map(|(r, ..)| r.least_mana_value()).max() else { return false };
        if cost.x == 0 || mana_value >= most as u32 {
            return false;
        }
        let per = cost.x as u32;
        (1..=(most as u32 - mana_value).div_ceil(per)).any(|n| {
            let mut more = *cost;
            more.generic = (more.generic as u32 + n * per).min(255) as u8;
            self.covers_as(&more, &ManaPayment::Spell { types, subtypes, legendary, mana_value: mana_value + n * per, x, from })
        })
    }

    /// `covers_for` at the cost as it is.
    fn covers_as(&self, cost: &ManaCost, payment: &ManaPayment) -> bool {
        if !self.restricted.iter().any(|(r, ..)| r.may_pay(payment)) {
            return self.covers(cost);
        }
        let mut with = Available {
            fixed: self.fixed,
            flex: self.flex,
            consume: self.consume,
            unknown: self.unknown,
            changers: Vec::new(),
            restricted: Vec::new(),
        };
        for (r, fixed, flex) in &self.restricted {
            if r.may_pay(payment) {
                for m in 0..6 {
                    with.fixed[m] = with.fixed[m].saturating_add(fixed[m]);
                }
                with.flex[ANY_COLOR as usize] += flex;
            }
        }
        with.covers(cost)
    }

    /// How much mana of no restriction it could pay at most (advice): the pool's and what the mana abilities add, less
    /// what those abilities cost themselves; `u16::MAX` when some ability's yield is not stated.
    pub fn most_unrestricted(&self) -> u16 {
        if self.unknown {
            return u16::MAX;
        }
        let total = self.fixed.iter().map(|&n| n as u32).sum::<u32>() + self.flex.iter().sum::<u32>();
        total.saturating_sub(self.consume).min(u16::MAX as u32) as u16
    }

    /// Whether `cost` could be paid with it (advice, CR 601.2g).
    pub fn covers(&self, cost: &ManaCost) -> bool {
        if self.unknown || cost.mana_value() == 0 {
            return true;
        }
        let fixed = self.fixed;
        let flex: u32 = self.flex.iter().sum();
        // What a mana ability costs is paid from what is already there.
        if flex > 0 && (fixed.iter().map(|&n| n as u32).sum::<u32>()) < self.consume {
            return false;
        }
        let mut left = fixed;
        // The symbols the fixed mana leaves unpaid, by the mana types that could pay each.
        let mut unpaid = [0u32; 64];
        let mut flex_needed: u32 = 0;
        for m in 0..6 {
            let need = cost.symbols[m];
            let have = left[m];
            let used = need.min(have);
            left[m] -= used;
            if need > used {
                unpaid[1 << m] += (need - used) as u32;
                flex_needed += (need - used) as u32;
            }
        }
        for (a, b) in cost.hybrid.iter().flatten() {
            if left[*a as usize] > 0 {
                left[*a as usize] -= 1;
            } else if left[*b as usize] > 0 {
                left[*b as usize] -= 1;
            } else {
                unpaid[(1 << *a as usize) | (1 << *b as usize)] += 1;
                flex_needed += 1;
            }
        }
        if flex_needed > 0 && !flex_pays(&unpaid, &self.flex) {
            return false;
        }
        let spare: u32 = (left.iter().map(|&n| n as u32).sum::<u32>() + (flex - flex_needed)).saturating_sub(self.consume);
        spare >= cost.generic as u32
    }
}

/// `ColorSet`'s five colors as mana types: what "mana of any color" may be.
const ANY_COLOR: u8 = 0b11111;

/// Whether the mana of a type to be chosen in `flex` could pay the symbols in `unpaid`, both counted by the mana types
/// each may be. Hall's condition (each symbol matched to a mana that may pay it): for every set of types, the symbols
/// payable only within it are no more than the mana that can be one of them.
fn flex_pays(unpaid: &[u32; 64], flex: &[u32; 64]) -> bool {
    let needs: Vec<usize> = (1..64).filter(|&m| unpaid[m] > 0).collect();
    let sources: Vec<usize> = (1..64).filter(|&m| flex[m] > 0).collect();
    for set in 1..64usize {
        let need: u32 = needs.iter().filter(|&&m| m & !set == 0).map(|&m| unpaid[m]).sum();
        if need > 0 && need > sources.iter().filter(|&&m| m & set != 0).map(|&m| flex[m]).sum::<u32>() {
            return false;
        }
    }
    true
}

/// The mana types a mana ability could add (a bit for each `Mana`).
fn yield_types(cx: &Cx, id: ObjectId, player: PlayerId, def: &ActivatedDef) -> u8 {
    let mut types = 0u8;
    for m in 0..6 {
        if def.adds.0[m] > 0 {
            types |= 1 << m;
        }
    }
    if def.adds_any > 0 {
        types |= any_types(cx, id, player, def);
    }
    types
}

/// The colors a mana ability's "mana of any color" may be: those it names, or all five.
fn any_types(cx: &Cx, id: ObjectId, player: PlayerId, def: &ActivatedDef) -> u8 {
    match def.colors_from {
        Some(from) => from(cx, &Asker { source: id, controller: player }).0,
        None if def.colors.is_empty() => ANY_COLOR,
        None => def.colors.0,
    }
}

fn activated_options(cx: &Cx, player: PlayerId, options: &mut Vec<ActionOption>) {
    // Abilities of permanents, then abilities that function from the hand (cycling) and the graveyard (unearth).
    let sources: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| cx.s().obj(id).controller == player)
        .chain(cx.s().zone(Zone::hand(player)).iter().copied())
        .chain(cx.s().zone(Zone::graveyard(player)).iter().copied())
        .collect();
    let forbidders = static_rules::activation_forbidders(cx);
    let available = std::cell::OnceCell::new();
    for id in sources {
        let zone = cx.s().obj(id).zone.kind();
        let chars = cx.chars(id);
        for (index, ability) in chars.abilities().enumerate() {
            let Some(def) = activated(cx, ability) else { continue };
            if def.is_mana_ability()
                || def.from != zone
                || limit_reached(cx, id, &chars, index, def)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            match def.timing {
                // Foretell (CR 702.143a): during your own turn.
                Timing::OwnTurn if cx.s().turn.active != player => continue,
                // Suspend (CR 702.62a): whenever the card could be cast.
                // "Could begin to cast this card by putting it onto the stack from your hand" (CR 702.62a): from the zone
                // the ability works from, which is the zone the card would be cast from.
                Timing::AsCast
                    if !(chars.is(CardType::Instant) || chars.has(Keyword::Flash) || sorcery_timing(cx, player))
                        || cast_barred(cx, player).contains(zone) =>
                {
                    continue;
                }
                _ => {}
            }
            let sorcery = def.timing == Timing::Sorcery || def.cost.loyalty.is_some();
            // A static ability, or an effect about the player, may lift the timing restriction of a loyalty ability (CR 606.3).
            if sorcery
                && !sorcery_timing(cx, player)
                && !(def.cost.loyalty.is_some() && static_rules::instant_loyalty(cx, player, id))
                && !static_rules::instant_activation(cx, player, id, index as u8)
            {
                continue;
            }
            // A loyalty ability once per turn per permanent (CR 606.3).
            if def.cost.loyalty.is_some() && !loyalty_available(cx, id) {
                continue;
            }
            let asker = Asker { source: id, controller: player };
            if let Some(condition) = def.condition
                && !condition(cx, &asker)
            {
                continue;
            }
            if !can_pay_other(cx, id, player, &def.cost) {
                continue;
            }
            // An activation whose mana cost the player cannot meet is not offered, as a spell they cannot afford is not
            // (CR 602.2b): a seat handed it would cancel at the payment and be handed it again, forever.
            let owed = if def.special_action { special_action_cost(cx, player, id, ability, def) } else { def.cost.mana };
            let judged = owed.x == 0 && owed.mana_value() > 0 && !def.tap_creatures_for_mana && !def.waterbend;
            // The source's own mana abilities cannot pay for an ability that taps it (CR 602.2b, 605.3b).
            let own = (judged && def.cost.tap && has_mana_source(cx, id, player)).then(|| available_without(cx, player, Some(id)));
            let a = judged.then(|| own.as_ref().unwrap_or_else(|| available.get_or_init(|| self::available(cx, player))));
            // A special action is no ability, so mana limited to activating abilities can't pay for it (CR 116.2, 106.6).
            let payment = if def.special_action { ManaPayment::Other } else { ManaPayment::Ability { source: chars.types, subtypes: chars.subtypes } };
            let covered = |a: &Available, targets: &[Target]| {
                // A special action is no activated ability (CR 116.2): what changes the cost of activating abilities does not
                // touch it; what changes its own cost is already in `owed` (`special_action_cost`), which `special_action`
                // pays.
                let mut cost =
                    if def.special_action { owed } else { change_with(cx, &a.changers, player, Costed::Ability(id, ability), owed, None, targets) };
                // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
                if static_rules::spend_any_color_for(cx, player, id) {
                    cost = ManaCost { generic: cost.mana_value().min(255) as u8, ..ManaCost::ZERO };
                }
                a.covers_for(&cost, &payment)
            };
            // The total cost is determined once the targets are announced (CR 601.2c, 601.2f, 602.2b), so a change that reads
            // them ("equip abilities that target this creature cost {2} less") is judged for each candidate a target announced
            // with the action could be: only those the cost could then be paid for are offered. Only where something could
            // change the cost - a cost changer on the battlefield, or the ability's or card's own - since nearly nothing does;
            // otherwise the cost is judged once, before any target is looked for.
            let each_target = a.is_some_and(|a| {
                targets_with_activation(def)
                    && !def.special_action
                    && def.modal.is_none()
                    && (!a.changers.is_empty() || def.cost_change.is_some() || !own_cost_changes(cx, id).is_empty())
            });
            if let Some(a) = a
                && !each_target
                && !covered(a, &[])
                && static_rules::activation_alternative(cx, player, id, def).is_none()
            {
                continue;
            }
            let mut option = ActionOption::on(ActionKind::ActivateAbility, id, index as u8);
            let possible = match def.modal {
                Some(modal) => modes_possible(cx, &asker, modal),
                None if def.late_targets => slots_satisfiable(cx, &asker, def.targets),
                None => with_targets(cx, &asker, def.targets, &mut option),
            };
            if !possible {
                continue;
            }
            // A static ability that lets the player pay another cost instead (CR 118.9) leaves every target on offer.
            if let Some(a) = a.filter(|_| each_target) {
                let kept: Vec<Target> = option.target_candidates.iter().copied().filter(|&t| covered(a, &[t])).collect();
                if kept.len() != option.target_candidates.len() && static_rules::activation_alternative(cx, player, id, def).is_none() {
                    if kept.is_empty() && !(option.targets_optional && covered(a, &[])) {
                        continue;
                    }
                    option.target_candidates = kept;
                }
            }
            options.push(option);
        }
    }
}

/// The marker in the history of an effect having allowed one more loyalty activation this turn (CR 606.3): an
/// activation of this ability index, past any a card prints.
pub const EXTRA_LOYALTY_ALLOWED: u8 = AbilityRef::EXTRA_LOYALTY_ALLOWED;

/// Whether a loyalty ability of `id` may be activated now (CR 606.3): none has been this turn, or effects have allowed
/// more than have been. Counted in the history, where the activations and the allowances are both recorded.
fn loyalty_available(cx: &Cx, id: ObjectId) -> bool {
    if cx.s().obj(id).loyalty_turn != cx.s().turn.number {
        return true;
    }
    let (mut allowed, mut used) = (1usize, 0usize);
    for r in cx.history().select(EventQuery::kind(EventMask::ABILITY_ACTIVATED).object(id).this_turn()) {
        if let Event::AbilityActivated { ability, .. } = r.event {
            if ability.index == EXTRA_LOYALTY_ALLOWED {
                allowed += 1;
            } else if matches!(cx.cards().ability(ability), AbilityDef::Activated(d) if d.cost.loyalty.is_some()) {
                used += 1;
            }
        }
    }
    used < allowed
}

/// The permission `player` holds to play `card` from where it is, if any (CR 601.3, 305.1).
pub fn permission_for(cx: &Cx, card: ObjectId, player: PlayerId) -> Option<PlayPermission> {
    cx.s().permissions.iter().find(|p| p.card == card && p.player == player && permission_stands(cx, p, player)).copied()
}

/// Whether a permission lets `player` play its card now, as far as its own terms say (CR 601.3, 305.1): not on the turn it
/// was given if it is for a later turn (CR 702.185a), and only while its condition holds (CR 611.2).
fn permission_stands(cx: &Cx, p: &PlayPermission, player: PlayerId) -> bool {
    !(p.grant.later_turns && p.turn == cx.s().turn.number) && p.grant.when.is_none_or(|when| play_condition_holds(cx, player, when))
}

/// Whether the condition on a permission to play holds now (CR 611.2): asked of the history of this turn, which no later
/// event can change, so it is a creature as it was declared that counts, dead or changed since.
fn play_condition_holds(cx: &Cx, player: PlayerId, when: PlayCondition) -> bool {
    match when {
        PlayCondition::AttackedWithThisTurn(kind) => cx.history().this_turn().iter().any(|r| {
            matches!(r.event, Event::Attacks { attacker, .. }
                if cx.s().obj(attacker).controller == player && cx.chars(attacker).has_subtype(kind))
        }),
        PlayCondition::ControlsSubtype(kind) => {
            cx.s().battlefield().iter().any(|&id| cx.s().obj(id).controller == player && cx.chars(id).has_subtype(kind))
        }
    }
}

/// Whether an effect lets `player` play lands and cast spells from their graveyard now (`EffectKind::PlayFromGraveyard`).
fn graveyard_play(cx: &Cx, player: PlayerId) -> bool {
    cx.s().effects.iter().any(|e| e.kind == EffectKind::PlayFromGraveyard && e.controller == player)
}

/// The play rules that apply to `player` now (`CardDef::play_rules`), each with the permanent whose rule it is: those of the
/// permanents they control, and with `command` of the emblems they have (CR 114.4), that have not lost their abilities (CR
/// 613.1f). A rule under "as long as" (`PlayRule::While`) applies only while its condition holds for its permanent: a
/// Case's "Solved - [static ability]" (CR 702.169b). A Room's are those of its unlocked doors: a locked door has no rules
/// text (CR 709.5).
pub fn each_play_rule(q: &dyn Query, cards: &CardTable, player: PlayerId, command: bool, mut f: impl FnMut(ObjectId, &'static PlayRule)) {
    fn walk(q: &dyn Query, asker: &Asker, rules: &'static [PlayRule], f: &mut impl FnMut(ObjectId, &'static PlayRule)) {
        for rule in rules {
            match rule {
                PlayRule::While(condition, inner) => {
                    if condition(q, asker) {
                        walk(q, asker, inner, f);
                    }
                }
                other => f(asker.source, other),
            }
        }
    }
    if !cards.any_play_rules {
        return;
    }
    let s = q.state();
    let emblems: &[ObjectId] = if command { s.zone(Zone::COMMAND) } else { &[] };
    for &id in s.battlefield().iter().chain(emblems) {
        let obj = s.obj(id);
        let card = cards.current(obj);
        if obj.controller != player || !cards.play_rules[card.0 as usize] || q.lost_all_abilities(id) {
            continue;
        }
        let def = cards.get(card);
        let asker = Asker { source: id, controller: player };
        if def.is_room() {
            for door in def.unlocked_doors(obj.marks.doors) {
                walk(q, &asker, door.play_rules, &mut f);
            }
        } else {
            walk(q, &asker, def.play_rules, &mut f);
        }
    }
}

/// What `player`'s permanents let them do about land plays (CR 305.2) and the top of their library (CR 305.1, 601.3).
#[derive(Clone, Copy, Default)]
struct LandRules {
    /// The land plays beyond the one.
    extra: u32,
    /// No limit to the land plays.
    unlimited: bool,
    /// Lands may be played from the graveyard.
    graveyard: bool,
    /// Lands may be played and spells cast from the top of the library, the spells for life (Bolas's Citadel).
    top_for_life: bool,
    /// Lands may be played from the top of the library (Bolas's Citadel; Case of the Locked Hothouse, solved).
    top_lands: bool,
}

fn land_rules(cx: &Cx, player: PlayerId) -> LandRules {
    // "Until end of turn, you may play lands and cast spells from your graveyard" (CR 305.1, 601.3): Yawgmoth's Will.
    let mut rules = LandRules { graveyard: graveyard_play(cx, player), ..LandRules::default() };
    each_play_rule(cx, cx.cards(), player, false, |_, rule| match rule {
        PlayRule::ExtraLandPlays(n) => rules.extra += *n as u32,
        PlayRule::AnyNumberOfLandPlays => rules.unlimited = true,
        PlayRule::LandsFromGraveyard => rules.graveyard = true,
        PlayRule::LibraryTop => {
            rules.top_for_life = true;
            rules.top_lands = true;
        }
        PlayRule::LibraryTopLands => rules.top_lands = true,
        PlayRule::LibraryTopSpells(_) | PlayRule::NoMaximumHandSize | PlayRule::LookAtLibraryTop | PlayRule::While(..) => {}
    });
    rules
}

/// Whether a permanent `player` controls, an emblem they have (CR 114.4), or an effect a resolved spell made for the rest
/// of the game says they have no maximum hand size (CR 402.2).
pub fn no_maximum_hand_size(cx: &Cx, player: PlayerId) -> bool {
    if cx.s().effects.iter().any(|e| e.kind == EffectKind::NoMaximumHandSize && e.controller == player) {
        return true;
    }
    let mut none = false;
    each_play_rule(cx, cx.cards(), player, true, |_, rule| none |= matches!(rule, PlayRule::NoMaximumHandSize));
    none
}

/// Whether `player` may look at the top card of their library any time (CR 401.2): a permanent they control says so (Bolas's
/// Citadel, Mystic Forge, Case of the Locked Hothouse once solved).
pub fn may_look_at_library_top(q: &dyn Query, cards: &CardTable, player: PlayerId) -> bool {
    let mut may = false;
    each_play_rule(q, cards, player, false, |_, rule| may |= matches!(rule, PlayRule::LookAtLibraryTop));
    may
}

fn land_plays(cx: &Cx, player: PlayerId, options: &mut Vec<ActionOption>) {
    let LandRules { extra, unlimited, graveyard: from_graveyard, top_lands: from_top, .. } = land_rules(cx, player);
    if !unlimited && cx.s().player(player).lands_played as u32 >= 1 + extra {
        return;
    }
    // A land card is played as itself; a modal double-faced card whose back face is a land may be played as that
    // (CR 712.4, 712.14a), from wherever a land could be played.
    let offer = |id: ObjectId, options: &mut Vec<ActionOption>| {
        if cx.chars(id).is(CardType::Land) {
            options.push(ActionOption::on(ActionKind::PlayLand, id, 0));
        }
        if cx.card(cx.s().obj(id).card).back_land {
            options.push(ActionOption::on(ActionKind::PlayLand, id, WAY_BACK_FACE));
        }
    };
    if from_top && let Some(&top) = cx.s().zone(Zone::library(player)).first() {
        offer(top, options);
    }
    for &id in cx.s().zone(Zone::hand(player)) {
        offer(id, options);
    }
    if from_graveyard {
        for &id in cx.s().zone(Zone::graveyard(player)) {
            offer(id, options);
        }
    }
    // A land card its owner exiled on an Adventure (CR 715.3d): "for as long as that card remains exiled, that player may
    // play it" - as a land play, at the time and within the number of land plays the rules allow (CR 305.1, 305.2).
    for &id in cx.s().zone(Zone::exile(player)) {
        if cx.s().obj(id).on_adventure {
            offer(id, options);
        }
    }
    // The land cards in the graveyard that a way a permanent gives lets the player play (CR 305.1, 604): once during each of
    // their turns, say. Looked for only when the graveyard holds a land card.
    let mut granted: Option<Vec<(ObjectId, PlayerId, &'static GrantedWay)>> = None;
    for &id in cx.s().zone(Zone::graveyard(player)) {
        if !cx.chars(id).is(CardType::Land) {
            continue;
        }
        let ways = granted.get_or_insert_with(|| granted_ways_of(cx.cards(), cx.s(), &cx.g.log));
        for (k, &(granter, controller, way)) in ways.iter().enumerate().take(WAY_GRANTED_SLOTS as usize) {
            if way.lands && (way.cost)(cx, &Asker { source: granter, controller }, id, player).is_some() {
                options.push(ActionOption::on(ActionKind::PlayLand, id, granted_way(k, GRANTED_FACE_WHOLE)));
            }
        }
    }
    // A land a card lets this player play from where it is (CR 305.1): "you may play that card".
    for p in &cx.s().permissions {
        if p.player == player
            && p.grant.lands
            && permission_stands(cx, p, player)
            && played_by_permission_from(cx, p.card, player, from_graveyard, from_top)
        {
            offer(p.card, options);
        }
    }
}

/// Whether a land play of `card` by `player` from where it is now would be by a permission (CR 305.1): not from the hand,
/// and not from a graveyard or the top of a library that a rule lets the player play lands from (`LandRules`), which is the
/// way it is played from there.
fn played_by_permission_from(cx: &Cx, card: ObjectId, player: PlayerId, from_graveyard: bool, from_top: bool) -> bool {
    let zone = cx.s().obj(card).zone;
    !zone.is(ZoneKind::Hand)
        && !(from_graveyard && zone == Zone::graveyard(player))
        && !(from_top && cx.s().zone(Zone::library(player)).first() == Some(&card))
}

/// The permission a land play of `card` by `player` is made by (CR 305.1), if it is one (`played_by_permission_from`): what
/// the permission says of a land played this way applies to it (`PlayGrant::enters_tapped`). Of several that would let it be
/// played, it is played by one that lets it enter untapped if there is one, as one permission is taken when several would
/// let a card be cast (`permission_for`): the land play offered is one, and entering tapped is all a permission can add to it.
pub fn land_play_permission(cx: &Cx, card: ObjectId, player: PlayerId) -> Option<PlayPermission> {
    let LandRules { graveyard, top_lands, .. } = land_rules(cx, player);
    if !played_by_permission_from(cx, card, player, graveyard, top_lands) {
        return None;
    }
    let mut by = cx.s().permissions.iter().filter(|p| p.card == card && p.player == player && p.grant.lands && permission_stands(cx, p, player));
    let first = by.clone().next().copied();
    by.find(|p| !p.grant.enters_tapped).copied().or(first)
}

/// The spells `player` could begin to cast: from hand by its mana cost or an alternative cost, from a graveyard by a
/// cost that casts it from there (flashback). `instants` picks the instant-speed pass or the sorcery-speed one.
/// "Each player can't cast more than N spells each turn" (CR 101.2): the smallest limit any permanent on the
/// battlefield imposes. A restriction takes precedence over the permission to cast.
fn cast_limit(cx: &Cx) -> Option<u8> {
    if !cx.cards().any_cast_limit {
        return None;
    }
    cx.s()
        .battlefield()
        .iter()
        .map(|&id| {
            let obj = cx.s().obj(id);
            // A permanent that has lost all its abilities limits nothing (CR 613.1f).
            let limit = cx.card(obj.copy.map_or(obj.card, |c| c.card)).cast_limit;
            if limit > 0 && cx.lost_all_abilities(id) { 0 } else { limit }
        })
        .filter(|&n| n > 0)
        .min()
}

/// The faces a card may be cast as (CR 709.3, 715.3, 720.3, 712.8a), as the face number a granted way names and the face: a
/// split card's two halves (the card itself is never cast), otherwise the card and its Adventure or Omen and its back face,
/// if it has them.
fn faces_to_cast(def: &'static CardDef) -> Vec<(u8, &'static CardDef)> {
    if let Some((left, right)) = def.split {
        return vec![(GRANTED_FACE_LEFT, left), (GRANTED_FACE_RIGHT, right)];
    }
    let mut faces = vec![(GRANTED_FACE_WHOLE, def)];
    for (_, face, inset) in def.inset_faces() {
        faces.push((face, inset));
    }
    if let Some(back) = def.back
        && back.has_cost
    {
        faces.push((GRANTED_FACE_BACK, back));
    }
    faces
}

/// The kinds of zone from which anything forbids `player` to cast a spell now, whatever the spell is (CR 101.2, 601.3): a
/// static ability that bars them from casting (Jennifer Walters) or from casting from some zones (Drannith Magistrate), an
/// effect that does (Avatar's Wrath), or a limit on the spells each player may cast in a turn (Archon of Emeria) already
/// reached, which bars every zone. Asked of every cast alike, whether it is begun with priority or as an effect resolves,
/// with the zone the card is in as it is begun (CR 601.2a); a copy of a card is cast from the zone it is created in (CR
/// 707.12).
fn cast_barred(cx: &Cx, player: PlayerId) -> ZoneKinds {
    if cast_limit(cx).is_some_and(|limit| cx.history().casts_by(player).count() >= limit as usize) {
        return ZoneKinds::ALL;
    }
    static_rules::cast_barred(cx, player)
}

/// Whether the card's own restriction on when it may be cast ("Cast this spell only ...", CR 601.3) allows it now.
fn cast_only_if_met(cx: &Cx, face: &'static CardDef, asker: &Asker) -> bool {
    face.cast_only_if.is_none_or(|only_if| only_if(cx, asker))
}

/// The ways the permanents on the battlefield give cards in graveyards to be cast (CR 604, 118.9): each with the permanent
/// and its controller, in the order a way number counts them. Nothing is allocated while no permanent has one.
fn granted_ways_of(cards: &CardTable, s: &GameState, log: &[Record]) -> Vec<(ObjectId, PlayerId, &'static GrantedWay)> {
    let mut ways = Vec::new();
    for &id in s.battlefield() {
        let obj = s.obj(id);
        let def = cards.get(cards.current(obj));
        if !def.granted_ways.is_empty() && crate::layers::lost_all_abilities(cards, s, log, id) {
            continue;
        }
        for way in def.granted_ways {
            ways.push((id, obj.controller, way));
        }
    }
    ways
}

/// The permissions the permanents on the battlefield give to cast cards in exile (CR 601.3, 604.1), each with the permanent
/// and its controller. Nothing is allocated while no permanent has one; a permanent that has lost its abilities gives none.
fn exile_casts_of(cards: &CardTable, s: &GameState, log: &[Record]) -> Vec<(ObjectId, PlayerId, &'static ExileCast)> {
    let mut casts = Vec::new();
    for &id in s.battlefield() {
        let obj = s.obj(id);
        let def = cards.get(cards.current(obj));
        if def.exile_casts.is_empty() || crate::layers::lost_all_abilities(cards, s, log, id) {
            continue;
        }
        for cast in def.exile_casts {
            casts.push((id, obj.controller, cast));
        }
    }
    casts
}

/// The permissions permanents give now to cast `card`, a card in exile, as itself (CR 601.3): each with the player it lets
/// cast it and what it is called, for the card's face to say.
pub fn exile_cast_texts(game: &crate::Game, card: ObjectId) -> Vec<(PlayerId, &'static str)> {
    let (q, cards, log): (&dyn Query, &CardTable, &[Record]) = (game, game.cards, &game.log);
    let s = q.state();
    if !s.obj(card).zone.is(ZoneKind::Exile) {
        return Vec::new();
    }
    exile_casts_of(cards, s, log)
        .into_iter()
        .filter(|&(granter, controller, cast)| (cast.allows)(q, &Asker { source: granter, controller }, card, controller))
        .map(|(_, controller, cast)| (controller, cast.text))
        .collect()
}

/// The first of `casts` that lets `player` cast `card` from exile now, as the card itself or, with `face`, as that face of
/// it (its Adventure or Omen, judged through a view of that face alone, CR 715.3a, 720.3a): the permanent and the permission.
fn exile_cast_for(
    cx: &Cx,
    casts: &[(ObjectId, PlayerId, &'static ExileCast)],
    card: ObjectId,
    face: Option<CardId>,
    player: PlayerId,
) -> Option<(ObjectId, &'static ExileCast)> {
    let view = face.map(|face| AsFace { cx, spell: card, face });
    let q: &dyn Query = match &view {
        Some(view) => view,
        None => cx,
    };
    casts.iter().find(|&&(granter, controller, c)| (c.allows)(q, &Asker { source: granter, controller }, card, player)).map(|&(granter, _, c)| (granter, c))
}

/// The permission a permanent gives `player` to cast `card`, a card in exile, as the face `face` (the card itself for
/// none), if nothing else lets them cast it from there (CR 601.3): a permission of its own, a prepare spell's copy or a card
/// on an Adventure. What the cast owes on top of its cost (CR 118.8) is the permission's.
fn exile_cast_permission(cx: &Cx, card: ObjectId, face: Option<CardId>, player: PlayerId) -> Option<(ObjectId, &'static ExileCast)> {
    let obj = cx.s().obj(card);
    if !obj.zone.is(ZoneKind::Exile) || obj.on_adventure || obj.card_copy || permission_for(cx, card, player).is_some() {
        return None;
    }
    let casts = exile_casts_of(cx.cards(), cx.s(), &cx.g.log);
    if casts.is_empty() {
        return None;
    }
    exile_cast_for(cx, &casts, card, face, player)
}

/// Whether the additional costs a card has (CR 118.8) could be paid, whatever way it is cast.
fn extra_payable(cx: &Cx, id: ObjectId, player: PlayerId, def: &CardDef) -> bool {
    def.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
        && (def.additional_choice.is_empty() || def.additional_choice.iter().any(|c| can_pay_other(cx, id, player, &c.cost)))
}

/// Whether a card, or the face of it that would be cast, may be cast whenever its caster could cast an instant (CR 307.5,
/// 702.8): an instant, a spell with flash, one of a type an effect lets them cast as though it had flash, or one a static
/// ability lets them (Sigarda's Aid, Valley Floodcaller). A face is judged as the spell it would be on the stack, its own
/// characteristics alone (CR 709.3a, 712.8a, 715.3a, 720.3a, 702.103b), so "noncreature spells" finds a creature's sorcery
/// Adventure and a bestowed Aura is an Aura spell.
struct Speed<'a> {
    cx: &'a Cx<'a>,
    /// The card types an effect lets the player cast as though they had flash (`static_rules::flash_types`).
    flash: TypeSet,
    /// The permanents whose static abilities may let them (`static_rules::flash_granters`).
    granters: &'a [(ObjectId, PlayerId, &'static CardDef)],
    player: PlayerId,
    card: ObjectId,
}

impl Speed<'_> {
    /// The card cast as itself, with the card's own "you may cast this spell as though it had flash".
    fn whole(&self, def: &'static CardDef) -> bool {
        def.cast_as_though_flash
            || own_speed(def, self.flash)
            || (!self.granters.is_empty() && static_rules::grants_flash(self.cx, self.granters, self.player, self.card))
    }

    /// The card cast as `face`: a split card's half or a Room's door, an Adventure or an Omen, a back face, a bestowed Aura.
    fn face(&self, face: &'static CardDef) -> bool {
        own_speed(face, self.flash)
            || (!self.granters.is_empty() && {
                let view = AsFace { cx: self.cx, spell: self.card, face: self.cx.card_id(face) };
                static_rules::grants_flash(&view, self.granters, self.player, self.card)
            })
    }
}

/// An instant, a spell with flash, or one of the `flash` types (CR 307.5, 702.8).
fn own_speed(def: &CardDef, flash: TypeSet) -> bool {
    def.is(CardType::Instant) || def.keywords.has(Keyword::Flash) || def.types.0 & flash.0 != 0
}

fn castable(cx: &Cx, player: PlayerId, available: &std::cell::OnceCell<Available>, instants: bool, options: &mut Vec<ActionOption>) {
    let barred = cast_barred(cx, player);
    if ZoneKinds::CAST_FROM.iter().all(|&from| barred.contains(from)) {
        return;
    }
    // Types of spell the player may cast as though they had flash (CR 307.5).
    let flash = static_rules::flash_types(cx, player);
    // Or by a static ability's say-so about a kind of spell (Sigarda's Aid): looked for once, empty unless one is on the board.
    let flash_granters = static_rules::flash_granters(cx);
    // Any player's exile, for a card a permission lets them cast from there; their own other zones.
    // A zone a rule forbids them to cast from is not looked in (CR 101.2).
    let zones = [Zone::hand(player), Zone::graveyard(player)]
        .into_iter()
        .chain(cx.s().seats().map(Zone::exile))
        .filter(|zone| !barred.contains(zone.kind()));
    let granted: std::cell::OnceCell<Vec<(ObjectId, PlayerId, &'static GrantedWay)>> = std::cell::OnceCell::new();
    let exile_casts: std::cell::OnceCell<Vec<(ObjectId, PlayerId, &'static ExileCast)>> = std::cell::OnceCell::new();
    for zone in zones {
        let own = zone.player == player;
        for &id in cx.s().zone(zone) {
            let obj = cx.s().obj(id);
            // From exile, only a prepare spell's copy, by the prepared permanent's controller (CR 722.3), or a
            // card exiled on an Adventure, by its owner (CR 715.5).
            let prepared_copy = zone.is(ZoneKind::Exile)
                && own
                && obj.card_copy
                && obj.prepared_from.is_some_and(|from| {
                    let f = cx.s().obj(from);
                    live(cx.s(), from) && f.zone.is(ZoneKind::Battlefield) && f.prepared && f.controller == player
                });
            let adventured = zone.is(ZoneKind::Exile) && own && obj.on_adventure;
            // Or by a permission to cast it from where it is (CR 601.3), from any zone it was granted for.
            let permitted = !zone.is(ZoneKind::Hand) && permission_for(cx, id, player).is_some();
            // Or by a static ability of a permanent that lets cards in exile be cast (CR 601.3, 604.1), judged on the card
            // as itself here and on its Adventure or Omen below: one that is not otherwise castable from there.
            let def = cx.card(obj.card);
            let by_exile_cast = zone.is(ZoneKind::Exile) && !prepared_copy && !adventured && !permitted;
            let casts: &[(ObjectId, PlayerId, &'static ExileCast)] =
                if by_exile_cast { exile_casts.get_or_init(|| exile_casts_of(cx.cards(), cx.s(), &cx.g.log)) } else { &[] };
            let exile_cast = if casts.is_empty() { None } else { exile_cast_for(cx, casts, id, None, player) };
            if by_exile_cast
                && exile_cast.is_none()
                && (casts.is_empty() || def.inset_faces().all(|(_, _, inset)| exile_cast_for(cx, casts, id, Some(cx.card_id(inset)), player).is_none()))
            {
                continue;
            }
            // The additional cost the permission adds to every way it casts the card by (CR 118.8, 601.2f), and whether it
            // could be paid.
            let additional = exile_cast.map(|(_, c)| c.additional);
            let additional_payable = additional.is_none_or(|a| can_pay_other(cx, id, player, &a));
            let permitted = permitted || exile_cast.is_some();
            // A land card is played, never cast (CR 305.1, 305.9), but one with an Adventure may be cast as that (CR 715.3:
            // Lindblum, Industrial Regency's Mage Siege), and only as that.
            let land = def.is(CardType::Land);
            if land && def.inset_faces().next().is_none() {
                continue;
            }
            let asker = Asker { source: id, controller: player };
            if !cast_only_if_met(cx, def, &asker) {
                continue;
            }
            // What a permission to cast it from here says to pay instead of its mana cost (CR 118.9).
            let grant = if permitted { permission_for(cx, id, player).map(|p| p.grant) } else { None };
            // Or life equal to its mana value (CR 118.9, 119.4): nothing in mana, and the life payable.
            let life_grant = grant.is_some_and(|g| g.life_for_mana_value);
            let instead = if life_grant { Some(ManaCost::ZERO) } else { grant.and_then(|g| g.instead) };
            let any_type = grant.is_some_and(|g| g.any_type);
            let more = grant.map_or(0, |g| g.more);
            // A permission to cast it only as its Adventure (CR 715.3) casts neither the card itself nor another face of it.
            let permitted_whole = permitted && !grant.is_some_and(|g| g.adventure_only);
            // A cost paid "rather than" the mana cost is itself an alternative cost: no other may be used with it (CR 118.9a).
            let alternatives = permitted_whole && !life_grant;
            let speed = Speed { cx, flash, granters: &flash_granters, player, card: id };
            let instant = speed.whole(def) && !grant.is_some_and(|g| g.sorcery_speed);
            // Or by the card's own say-so, for its ordinary cost (CR 113.6b): "You may cast this card from your graveyard";
            // or by a rule that lets its owner cast spells from their graveyard (Yawgmoth's Will).
            let from_graveyard = zone.is(ZoneKind::Graveyard) && own && (def.keywords.has(Keyword::CastFromYourGraveyard) || graveyard_play(cx, player));
            // Or by retrace (CR 702.81a), printed or granted: for its ordinary cost and a land card discarded. Asked only on
            // the pass that may cast it, at its own speed or by an instant-speed way.
            let retrace = zone.is(ZoneKind::Graveyard)
                && own
                && !from_graveyard
                && (instant == instants || (instants && def.ways.iter().any(|w| w.instant_speed)))
                && cx.chars(id).has(Keyword::Retrace)
                && can_pay_other(cx, id, player, &RETRACE);
            let elsewhere = alternatives || from_graveyard || retrace || adventured;
            // Whether a way of casting it from the hand is open to it here (CR 118.9, 601.3): one that names no zone is an
            // alternative cost that may be paid whatever zone a rule or permission has the spell cast from, as Yawgmoth's
            // Will does for Force of Will's, Daze's or web-slinging's; one whose text names the hand is not (warp, CR
            // 702.185a). `elsewhere` is a rule or permission that has it cast from this other zone.
            let from_elsewhere = |way: &WayDef, elsewhere: bool| way.from == ZoneKind::Hand && !way.only_from && !zone.is(ZoneKind::Hand) && elsewhere;
            if instant == instants && !land {
                if def.split.is_none()
                    && (zone.is(ZoneKind::Hand) || prepared_copy || adventured || permitted_whole || from_graveyard || retrace)
                    // A card with no mana cost is cast only for an alternative cost (CR 118.6a): one the permission names, life
                    // equal to its mana value (0) included.
                    && (def.has_cost || instead.is_some())
                    && (!life_grant || can_pay_other(cx, id, player, &life_for_mana_value(def)))
                    && additional_payable
                    && def.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
                    && (def.additional_choice.is_empty()
                        || def.additional_choice.iter().any(|c| can_pay_other(cx, id, player, &c.cost)))
                {
                    let a = available.get_or_init(|| self::available(cx, player));
                    let covered = |targets: &[Target]| {
                        let mut least = least_cost(cx, a, player, id, def, instead.unwrap_or(def.cost), targets);
                        least.generic = least.generic.saturating_add(more);
                        if let Some(additional) = additional {
                            least = add_cost(least, additional.mana);
                        }
                        if any_type {
                            // Mana of any type pays every symbol of it (CR 609.4b).
                            least = ManaCost { generic: least.mana_value().min(255) as u8, ..ManaCost::ZERO };
                        }
                        covers_total(cx, a, player, id, def, least, 0) && added_life_payable(cx, a, player, id, def, targets)
                    };
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
                    if offer_costed(cx, a, &asker, def, &mut option, covered) {
                        options.push(option);
                    }
                }
                for (n, way) in def.ways.iter().enumerate() {
                    // Sneak and the like are offered below, at instant speed whatever the card is (CR 702.190a).
                    if way.instant_speed {
                        continue;
                    }
                    if (way.from != zone.kind() && !from_elsewhere(way, elsewhere)) || !own {
                        continue;
                    }
                    if let Some(condition) = way.condition
                        && !condition(cx, &asker)
                    {
                        continue;
                    }
                    // An additional cost is owed with every way of casting it (CR 118.8, 601.2f).
                    if !can_pay_other(cx, id, player, &way.cost) || !extra_payable(cx, id, player, def) || !additional_payable {
                        continue;
                    }
                    let a = available.get_or_init(|| self::available(cx, player));
                    if !way_covered(cx, a, player, id, def, &additional.map_or(way.cost, |extra| way.cost.and(extra))) {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, (n + 1) as u8);
                    if offer_spell(cx, &asker, def, &mut option) {
                        options.push(option);
                    }
                }
                // Overload (CR 702.96a): from the hand, for its overload cost. It has no targets, so it is offered
                // whether or not there is anything to target.
                if let Some(overload) = def.overload
                    && own
                    && zone.is(ZoneKind::Hand)
                {
                    let a = available.get_or_init(|| self::available(cx, player));
                    let cost = least_cost(cx, a, player, id, def, overload, &[]);
                    if covers_total(cx, a, player, id, def, cost, 0) {
                        options.push(ActionOption::on(ActionKind::CastSpell, id, WAY_OVERLOAD));
                    }
                }
            }
            // An alternative cost that may be used whenever an instant could be cast, whatever the card is (sneak, CR 702.190a).
            // From another zone by a permission (sneak names no zone), unless the permission says when (CR 601.3).
            if instants && !land {
                for (n, way) in def.ways.iter().enumerate() {
                    if !way.instant_speed || !own {
                        continue;
                    }
                    if way.from != zone.kind() && !from_elsewhere(way, elsewhere && !grant.is_some_and(|g| g.sorcery_speed)) {
                        continue;
                    }
                    if let Some(condition) = way.condition
                        && !condition(cx, &asker)
                    {
                        continue;
                    }
                    if !can_pay_other(cx, id, player, &way.cost) || !extra_payable(cx, id, player, def) || !additional_payable {
                        continue;
                    }
                    let a = available.get_or_init(|| self::available(cx, player));
                    if !way_covered(cx, a, player, id, def, &additional.map_or(way.cost, |extra| way.cost.and(extra))) {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, (n + 1) as u8);
                    if offer_spell(cx, &asker, def, &mut option) {
                        options.push(option);
                    }
                }
            }
            // The ways a permanent gives a card in its owner's graveyard or hand to be cast (CR 604, 118.9): each judged on
            // the face that would be cast, the only spell it would be on the stack (CR 709.3, 715.4, 712.8a), so a Room's door
            // is chosen here and a restriction on the spell ("mana value 3 or less") reads that door's characteristics.
            if own && matches!(zone.kind(), ZoneKind::Graveyard | ZoneKind::Hand) {
                let ways = granted.get_or_init(|| granted_ways_of(cx.cards(), cx.s(), &cx.g.log));
                if ways.iter().any(|&(_, _, way)| way.from == zone.kind()) {
                    for (face_way, face_def) in faces_to_cast(def).into_iter().filter(|&(f, _)| !(land && f == GRANTED_FACE_WHOLE)) {
                        let face_instant = if face_way == GRANTED_FACE_WHOLE { instant } else { speed.face(face_def) };
                        if face_instant != instants {
                            continue;
                        }
                        let face_id = cx.card_id(face_def);
                        let view = AsFace { cx, spell: id, face: face_id };
                        let q: &dyn Query = if face_way == GRANTED_FACE_WHOLE { cx } else { &view };
                        for (k, &(granter, controller, way)) in ways.iter().enumerate().take(WAY_GRANTED_SLOTS as usize) {
                            if way.from != zone.kind() {
                                continue;
                            }
                            let Some(cost) = (way.cost)(q, &Asker { source: granter, controller }, id, player) else { continue };
                            if !can_pay_other(cx, id, player, &cost) || !extra_payable(cx, id, player, face_def) {
                                continue;
                            }
                            let a = available.get_or_init(|| self::available(cx, player));
                            let covered = |targets: &[Target]| {
                                let least = least_cost(cx, a, player, id, face_def, cost.mana, targets);
                                covers_total(cx, a, player, id, face_def, least, cost.exile_from_graveyard)
                                    && added_life_payable(cx, a, player, id, face_def, targets)
                            };
                            let mut option = ActionOption::on(ActionKind::CastSpell, id, granted_way(k, face_way));
                            if offer_costed(cx, a, &asker, face_def, &mut option, covered) {
                                options.push(option);
                            }
                        }
                    }
                }
            }
            // What a face cast by the permission costs (CR 118.9, 601.2f): what it names instead, or life equal to the face's
            // mana value, and what it adds; the face's own mana cost from the hand.
            let face_covered = |face: &CardDef, by_permission: Option<Option<Cost>>| {
                let a = available.get_or_init(|| self::available(cx, player));
                let Some(additional) = by_permission else {
                    return a.covers_for(&least_cost(cx, a, player, id, face, face.cost, &[]), &spell_kind(cx, a, id, face));
                };
                if life_grant && !can_pay_other(cx, id, player, &life_for_mana_value(face)) {
                    return false;
                }
                let mut least = least_cost(cx, a, player, id, face, instead.unwrap_or(face.cost), &[]);
                least.generic = least.generic.saturating_add(more);
                if let Some(additional) = additional {
                    least = add_cost(least, additional.mana);
                }
                if any_type {
                    least = ManaCost { generic: least.mana_value().min(255) as u8, ..ManaCost::ZERO };
                }
                a.covers_for(&least, &spell_kind(cx, a, id, face))
            };
            // Whether a permission casts this face from here, and what it adds (CR 118.8): a permission of the card's own for
            // the card, or a permanent's to cast cards from exile, judged on the face (CR 709.3a, 712.8a), whose additional
            // cost has to be payable.
            let face_permission = |face: &'static CardDef| -> Option<Option<Cost>> {
                if zone.is(ZoneKind::Hand) {
                    None
                } else if by_exile_cast {
                    match exile_cast_for(cx, casts, id, Some(cx.card_id(face)), player) {
                        Some((_, c)) if can_pay_other(cx, id, player, &c.additional) => Some(Some(c.additional)),
                        _ => None,
                    }
                } else if permitted_whole {
                    Some(None)
                } else {
                    None
                }
            };
            let face_speed = |face: &'static CardDef| speed.face(face) && !grant.is_some_and(|g| g.sorcery_speed);
            // A split card's halves (CR 709.3): each cast for its own cost at its own speed, from the hand, or for an
            // aftermath half (CR 702.127a) from the graveyard only; or, but for an aftermath half, from wherever a permission
            // lets the card be cast (CR 601.3, 709.3).
            if let Some((left, right)) = def.split {
                for (way, half) in [(WAY_SPLIT_LEFT, left), (WAY_SPLIT_RIGHT, right)] {
                    let from = if half.aftermath { ZoneKind::Graveyard } else { ZoneKind::Hand };
                    let by_permission = if half.aftermath { None } else { face_permission(half) };
                    if (!(zone.is(from) && own) && by_permission.is_none()) || face_speed(half) != instants {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, way);
                    if face_covered(half, by_permission) && offer_spell(cx, &asker, half, &mut option) {
                        options.push(option);
                    }
                }
            }
            // A modal double-faced card's back face, from the hand or wherever a permission lets the card be cast (CR 712.11b,
            // 601.3), at the speed of the back face's own type (CR 712.2, 712.8a): the back face of a transforming card has no
            // mana cost and is not cast.
            if let Some(back) = def.back
                && back.has_cost
                && face_speed(back) == instants
                && let back_by_permission = face_permission(back)
                && (zone.is(ZoneKind::Hand) || back_by_permission.is_some())
            {
                let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_BACK_FACE);
                if face_covered(back, back_by_permission) && offer_spell(cx, &asker, back, &mut option) {
                    options.push(option);
                }
            }
            // Bestow (CR 702.103a): from the hand, as the Aura it becomes, which needs a creature to enchant (CR 601.2c).
            if (zone.is(ZoneKind::Hand) || (zone.is(ZoneKind::Graveyard) && own && def.bestow_from_graveyard))
                && let Some(bestow) = def.bestow
                && speed.face(bestow) == instants
                && bestow.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
            {
                let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_BESTOW);
                if {
                    let a = available.get_or_init(|| self::available(cx, player));
                    a.covers_for(&least_cost(cx, a, player, id, bestow, bestow.cost, &[]), &spell_kind(cx, a, id, bestow))
                } && offer_spell(cx, &asker, bestow, &mut option)
                {
                    options.push(option);
                }
            }
            // Its Adventure or its Omen (CR 715.3, 720.3): wherever the card itself may be cast - from the hand, or from
            // elsewhere by a permission or by the card's or an effect's say-so (CR 601.3), for what that permission says to
            // pay - but not by the permission an Adventure gives a card it exiled (CR 715.3d). At the speed of that face's
            // own type, and judged by that face alone (CR 715.3a, 720.3a).
            let from_graveyard = zone.is(ZoneKind::Graveyard) && own && (def.keywords.has(Keyword::CastFromYourGraveyard) || graveyard_play(cx, player));
            if zone.is(ZoneKind::Hand) || permitted || from_graveyard || by_exile_cast {
                for (way, _, inset) in def.inset_faces() {
                    if permitted && !zone.is(ZoneKind::Hand) && !from_graveyard && grant.is_some_and(|g| g.adventure_only) && way != WAY_ADVENTURE {
                        continue;
                    }
                    if life_grant && !from_graveyard && !can_pay_other(cx, id, player, &life_for_mana_value(inset)) {
                        continue;
                    }
                    let face_instant = speed.face(inset) && !grant.is_some_and(|g| g.sorcery_speed);
                    if face_instant != instants {
                        continue;
                    }
                    // A permanent's permission to cast from exile is judged on the face (CR 715.3a, 720.3a).
                    let face_additional = if by_exile_cast {
                        match exile_cast_for(cx, casts, id, Some(cx.card_id(inset)), player) {
                            Some((_, c)) if can_pay_other(cx, id, player, &c.additional) => Some(c.additional),
                            _ => continue,
                        }
                    } else {
                        None
                    };
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, way);
                    if {
                        let a = available.get_or_init(|| self::available(cx, player));
                        let mut least = least_cost(cx, a, player, id, inset, instead.unwrap_or(inset.cost), &[]);
                        least.generic = least.generic.saturating_add(more);
                        if let Some(additional) = face_additional {
                            least = add_cost(least, additional.mana);
                        }
                        if any_type {
                            // Mana of any type pays every symbol of it (CR 609.4b).
                            least = ManaCost { generic: least.mana_value().min(255) as u8, ..ManaCost::ZERO };
                        }
                        a.covers_for(&least, &spell_kind(cx, a, id, inset))
                    } && offer_spell(cx, &asker, inset, &mut option)
                    {
                        options.push(option);
                    }
                }
            }
        }
    }
    // The top card of the library, for a player a permanent lets cast spells from there for their mana cost (CR 601.3).
    if cx.cards().any_play_rules
        && !barred.contains(ZoneKind::Library)
        && let Some(&id) = cx.s().zone(Zone::library(player)).first()
    {
        let def = cx.card(cx.s().obj(id).card);
        let mut allowed = false;
        each_play_rule(cx, cx.cards(), player, false, |source, rule| {
            allowed |= matches!(rule, PlayRule::LibraryTopSpells(f) if f(cx, &Asker { source, controller: player }, id))
        });
        let instant = Speed { cx, flash, granters: &flash_granters, player, card: id }.whole(def);
        // Where a permanent lets them pay life instead, that cast below is the one offered.
        if allowed && !land_rules(cx, player).top_for_life && !def.is(CardType::Land) && def.has_cost && instant == instants {
            let a = available.get_or_init(|| self::available(cx, player));
            let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
            if covers_total(cx, a, player, id, def, least_cost(cx, a, player, id, def, def.cost, &[]), 0)
                && offer_spell(cx, &Asker { source: id, controller: player }, def, &mut option)
            {
                options.push(option);
            }
        }
    }
    // The top card of the library, for a player a permanent lets cast from there for life (CR 601.3, 118.9).
    if !barred.contains(ZoneKind::Library)
        && land_rules(cx, player).top_for_life
        && let Some(&id) = cx.s().zone(Zone::library(player)).first()
    {
        let def = cx.card(cx.s().obj(id).card);
        let instant = Speed { cx, flash, granters: &flash_granters, player, card: id }.whole(def);
        // Life is an alternative cost, so a card with no mana cost may be cast this way too, for 0 life (CR 118.6a).
        if !def.is(CardType::Land) && instant == instants && can_pay_other(cx, id, player, &life_for_mana_value(def)) {
            let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
            if offer_spell(cx, &Asker { source: id, controller: player }, def, &mut option) {
                options.push(option);
            }
        }
    }
}

/// The life a cast of `card` by `player` by this way pays instead of its mana cost (CR 118.9, 119.4), for the menu row and the
/// seats that weigh it: a permission's life equal to the face's mana value (Inside Information), or Bolas's Citadel's for the
/// top of the library. None for a cast paid in mana.
pub fn life_for_cast(q: &dyn Query, cards: &CardTable, player: PlayerId, card: ObjectId, way: u8) -> Option<u8> {
    let state = q.state();
    let obj = state.obj(card);
    let def = cards.get(obj.card);
    if !obj.zone.is(ZoneKind::Hand)
        && state.permissions.iter().any(|p| p.card == card && p.player == player && p.grant.life_for_mana_value)
    {
        return Some(life_for_mana_value(face_cast(def, way)).life);
    }
    let mut top = false;
    if way == 0 && state.zone(Zone::library(player)).first() == Some(&card) {
        each_play_rule(q, cards, player, false, |_, rule| top |= matches!(rule, PlayRule::LibraryTop));
    }
    top.then(|| life_for_mana_value(def).life)
}

/// The face of a card cast by a way of casting it: the card itself, its Adventure or Omen, a split card's half or a modal
/// double-faced card's back face (CR 715.3, 720.3, 709.3, 712.11b).
fn face_cast(def: &'static CardDef, way: u8) -> &'static CardDef {
    match (way, def.split) {
        (WAY_SPLIT_LEFT, Some((left, _))) => left,
        (WAY_SPLIT_RIGHT, Some((_, right))) => right,
        (WAY_BACK_FACE, _) => def.back.unwrap_or(def),
        (0, _) => def,
        _ => def.inset_face(way).unwrap_or(def),
    }
}

/// "Pay life equal to its mana value rather than pay its mana cost" (CR 118.9, 119.4): the life, as a cost, for a card or the
/// face of it cast, its X counted as 0 (CR 107.3). Bolas's Citadel's rule and Inside Information's permission pay it.
fn life_for_mana_value(def: &CardDef) -> Cost {
    Cost { life: def.cost.mana_value().min(255) as u8, ..Cost::NONE }
}

/// The most an emerge cost could come down by right now (CR 702.119a): the mana values of the richest permanents it could
/// sacrifice. What it comes down by is not known until the player has chosen, so this is only what makes the cast worth
/// offering.
fn emerge_reduction(cx: &Cx, player: PlayerId, card: ObjectId, cost: &Cost) -> u8 {
    let Some(p) = cost.permanents.filter(|_| cost.emerge) else { return 0 };
    let asker = Asker { source: card, controller: player };
    let mut values: Vec<u32> =
        cx.permanents_of(player).into_iter().filter(|&id| pays_permanents(cx, cost, &p, &asker, id)).map(|id| cx.mana_value(id)).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    values.iter().take(p.count as usize).sum::<u32>().min(255) as u8
}

/// The creatures a cost that taps one to reduce its mana may tap (CR 702.180a): the controller's untapped creatures. A
/// creature with summoning sickness may be tapped, since this is not its own {T} ability (CR 302.6, 107.5).
fn reducer_candidates(cx: &Cx, controller: PlayerId) -> Vec<ObjectId> {
    cx.permanents_of(controller).into_iter().filter(|&id| !cx.s().obj(id).tapped && cx.chars(id).is(CardType::Creature)).collect()
}

/// What tapping `creature` takes off the generic mana of the total cost (CR 702.180a): its power, none if that is below 0.
fn reduction_of(cx: &Cx, creature: ObjectId) -> u8 {
    cx.chars(creature).power.unwrap_or(0).clamp(0, 255) as u8
}

/// Whether a way of casting `card` is worth offering (advice, CR 601.2g): the least its mana can come to, with emerge's
/// richest sacrifice, is covered; or, for harmonize (CR 702.180a), is covered once some untapped creature is tapped for
/// its power, that creature then not tapping for mana.
fn way_covered(cx: &Cx, a: &Available, player: PlayerId, card: ObjectId, def: &CardDef, way: &Cost) -> bool {
    let mut cost = least_cost(cx, a, player, card, def, way.mana, &[]);
    cost.generic = cost.generic.saturating_sub(emerge_reduction(cx, player, card, way));
    if covers_total(cx, a, player, card, def, cost, way.exile_from_graveyard) {
        return true;
    }
    way.tap_to_reduce
        && reducer_candidates(cx, player).into_iter().any(|creature| {
            let without = available_without(cx, player, Some(creature));
            let mut reduced = cost;
            reduced.generic = reduced.generic.saturating_sub(reduction_of(cx, creature));
            covers_total(cx, &without, player, card, def, reduced, way.exile_from_graveyard)
        })
}

/// Chooses the creature a cost taps to reduce its mana (CR 702.180b, 601.2b): up to one untapped creature, asked as the cost
/// is chosen, before any target. Nothing is asked when there is no creature to tap.
///
/// The prompt says what tapping each creature takes off: its power, but never more than the generic mana the total cost has as far
/// as it is known now (the cost changes in play applied), since colored mana does not come down (CR 601.2f).
///
/// That total is read here, before X and kicker are announced (CR 601.2b), so a future harmonize card with X or a kicker must
/// recompute it once they are, or the caption's min(power, generic) understates what comes off.
fn choose_reducer(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Option<ObjectId>> {
    if !cost.tap_to_reduce {
        return Ok(None);
    }
    let eligible = reducer_candidates(cx, player);
    if eligible.is_empty() {
        return Ok(None);
    }
    // Before any target is chosen (CR 601.2b), so a change that reads the targets is not in it.
    let generic = apply_cost_changes(cx, player, Costed::Spell(source), cost.mana, &[]).generic;
    let powers = eligible.iter().map(|&c| reduction_of(cx, c).min(generic)).collect();
    let ask = RequestKind::TapForPower { source, purpose: Purpose::TapToReduce, cards: eligible, powers, minimum: 0 };
    match cx.ask_core(player, ask, true)? {
        Answer::Cards(cards) => Ok(cards.first().copied()),
        _ => Err(Stop::Cancel),
    }
}

fn is_land(q: &dyn Query, _: &Asker, id: ObjectId) -> bool {
    q.chars(id).is(CardType::Land)
}

/// The additional cost of retrace (CR 702.81a): discard a land card.
const RETRACE: Cost = Cost { discard: Some((1, is_land)), ..Cost::NONE };

/// Whether a permanent has a mana ability that could be activated now: one more way to spend its tap, which a creature
/// or artifact tapped to pay for convoke or improvise cannot also be used for.
fn has_mana_source(cx: &Cx, id: ObjectId, player: PlayerId) -> bool {
    let forbidders = static_rules::activation_forbidders(cx);
    let chars = cx.chars(id);
    chars.abilities().enumerate().any(|(index, ability)| {
        activated(cx, ability).is_some_and(|def| {
            def.is_mana_ability()
                && !limit_reached(cx, id, &chars, index, def)
                && can_pay_other(cx, id, player, &def.cost)
                && !static_rules::activation_barred(cx, &forbidders, id, def)
        })
    })
}

/// Whether a spell's total cost could be paid (advice, CR 601.2g-h): by the mana at hand, with what else may pay for it
/// (convoke's creatures, delve's graveyard, improvise's artifacts) each paying their share, or by those alone when no
/// mana may be spent on it. A permanent that is a mana source is counted as that, not twice. `exiled` is how many
/// cards another part of the cost exiles from the graveyard, which delve cannot also use.
fn covers_total(cx: &Cx, a: &Available, player: PlayerId, card: ObjectId, def: &CardDef, cost: ManaCost, exiled: u8) -> bool {
    let no_mana = def.keywords.has(Keyword::CantSpendManaToCast);
    if !def.pay_with.any() && !no_mana {
        return a.covers_for(&cost, &spell_kind(cx, a, card, def));
    }
    let mut cost = cost;
    let mut creatures: Vec<ColorSet> = Vec::new();
    let mut generic: u32 = 0;
    if def.pay_with.delve {
        let gy = cx.s().zone(Zone::graveyard(player)).iter().filter(|&&c| c != card).count();
        generic += gy.saturating_sub(exiled as usize) as u32;
    }
    if def.pay_with.improvise || def.pay_with.convoke {
        for id in cx.permanents_of(player) {
            if cx.s().obj(id).tapped || (!no_mana && has_mana_source(cx, id, player)) {
                continue;
            }
            let chars = cx.chars(id);
            if def.pay_with.improvise && chars.is(CardType::Artifact) {
                generic += 1;
            } else if def.pay_with.convoke && chars.is(CardType::Creature) {
                creatures.push(chars.colors);
            }
        }
    }
    cost = convoked(&cost, &creatures);
    cost.generic = cost.generic.saturating_sub(generic.min(255) as u8);
    if no_mana { cost.is_zero() } else { a.covers_for(&cost, &spell_kind(cx, a, card, def)) }
}

/// The text an option is matched by and a menu row shows: an alternative cost's, a granted way's or an ability's. A cast a
/// standing permission lets a card have (CR 601.3) says what the permission changes: "Flashback {2}{R}" for a flashback granted
/// (Snapcaster Mage), "Cast without paying its mana cost", "Cast by paying {1}{U} instead".
pub fn describe(cards: &CardTable, state: &GameState, log: &[Record], option: &ActionOption) -> std::borrow::Cow<'static, str> {
    let Some(source) = option.source else { return "".into() };
    // "Pay life equal to its mana value rather than pay its mana cost" (CR 118.9): the life, for the face cast.
    if option.kind == ActionKind::CastSpell
        && matches!(option.index, 0 | WAY_ADVENTURE | WAY_SPLIT_LEFT | WAY_SPLIT_RIGHT | WAY_BACK_FACE)
        && state.obj(source).zone != Zone::hand(state.obj(source).owner)
        && state.permissions.iter().any(|p| p.card == source && p.grant.life_for_mana_value)
    {
        let def = cards.get(state.obj(source).card);
        let face = face_cast(def, option.index);
        let life = life_for_mana_value(face).life;
        return match face.name {
            _ if option.index == 0 => format!("Cast by paying {life} life instead").into(),
            name => format!("{name}: cast by paying {life} life instead").into(),
        };
    }
    if option.kind == ActionKind::CastSpell
        && option.index == 0
        && let Some(grant) = state.permissions.iter().find(|p| p.card == source && (p.grant.instead.is_some() || p.grant.exile_after)).map(|p| p.grant)
    {
        // A flashback granted costs the card's own mana cost unless the grant names another.
        let cost = grant.instead.unwrap_or_else(|| cards.get(state.obj(source).card).cost);
        return if grant.exile_after {
            format!("Flashback {cost}").into()
        } else if cost == ManaCost::ZERO {
            "Cast without paying its mana cost".into()
        } else {
            format!("Cast by paying {cost} instead").into()
        };
    }
    // A card in exile cast by a permanent's permission (CR 601.3) says what the permission adds: "Cast by removing three counters
    // from among creatures you control". The permanent it was exiled with, if that is one that gives such permission.
    // Cast for an alternative cost or as another face the same way, the way says so too: "Evoke (cast by removing ...)".
    if option.kind == ActionKind::CastSpell
        && let Some(text) = exile_cast_text(cards, state, log, source)
    {
        return if option.index == 0 { text.into() } else { format!("{} ({})", describe_way(cards, state, log, option, source), lowercase_first(text)).into() };
    }
    // A granted cast of a card that can be cast as more than one face (a Room's doors) names the face: "Unholy Annex: Once
    // during each of your turns, ...".
    if option.kind == ActionKind::CastSpell
        && let Some((_, face)) = granted_slot(option.index)
    {
        let def = cards.get(state.obj(source).card);
        if let Some((_, face_def)) = faces_to_cast(def).into_iter().find(|&(f, _)| f == face)
            && (face != GRANTED_FACE_WHOLE || def.inset_faces().next().is_some() || def.back.is_some_and(|b| b.has_cost))
        {
            return format!("{}: {}", face_def.name, describe_way(cards, state, log, option, source)).into();
        }
    }
    describe_way(cards, state, log, option, source).into()
}

/// The text of the permission of the permanent a card in exile was exiled with, if that permanent gives one to cast cards in
/// exile (CR 601.3) and nothing else lets the card be cast from there (a play permission of its own, an Adventure, a prepare
/// copy). Read without asking whether it allows the cast: the option it describes was offered because it did.
fn exile_cast_text(cards: &CardTable, state: &GameState, log: &[Record], card: ObjectId) -> Option<&'static str> {
    let obj = state.obj(card);
    if !obj.zone.is(ZoneKind::Exile) || obj.on_adventure || obj.card_copy || state.permissions.iter().any(|p| p.card == card) {
        return None;
    }
    let by = obj.exiled?.by;
    exile_casts_of(cards, state, log).into_iter().find(|&(granter, _, _)| granter == by).map(|(_, _, cast)| cast.text)
}

fn lowercase_first(text: &str) -> String {
    let mut chars = text.chars();
    chars.next().map_or_else(String::new, |c| c.to_lowercase().chain(chars).collect())
}

fn describe_way(cards: &CardTable, state: &GameState, log: &[Record], option: &ActionOption, source: ObjectId) -> &'static str {
    match option.kind {
        ActionKind::CastSpell if matches!(option.index, WAY_ADVENTURE | WAY_OMEN) => {
            cards.get(state.obj(source).card).inset_face(option.index).map_or("", |a| a.name)
        }
        _ if option.index == WAY_BACK_FACE => cards.get(state.obj(source).card).back.map_or("", |b| b.name),
        ActionKind::CastSpell if option.index == WAY_BESTOW => "Bestow",
        ActionKind::CastSpell | ActionKind::PlayLand if granted_slot(option.index).is_some() => {
            let (k, _) = granted_slot(option.index).expect("a granted way");
            granted_ways_of(cards, state, log).get(k).map_or("", |&(_, _, way)| way.text)
        }
        ActionKind::CastSpell if option.index == WAY_SPLIT_LEFT => cards.get(state.obj(source).card).split.map_or("", |h| h.0.name),
        ActionKind::CastSpell if option.index == WAY_SPLIT_RIGHT => cards.get(state.obj(source).card).split.map_or("", |h| h.1.name),
        ActionKind::CastSpell if option.index > 0 && option.index < WAY_GRANTED => cards.get(state.obj(source).card).ways[option.index as usize - 1].text,
        ActionKind::CastSpell if option.index == WAY_OVERLOAD => "Overload",
        ActionKind::ActivateAbility if option.index >= PLAYER_MANA_ABILITY => state
            .effects
            .iter()
            .filter(|e| matches!(e.kind, EffectKind::PlayerManaAbility(_)))
            .find(|e| e.source == source)
            .map_or("", |e| match e.kind {
                EffectKind::PlayerManaAbility(a) => cards.ability(a).text(),
                _ => "",
            }),
        ActionKind::ActivateAbility => {
            let chars = layers::chars(cards, state, log, source);
            chars.abilities().nth(option.index as usize).map(|a| cards.ability(a).text()).unwrap_or("")
        }
        _ => "",
    }
}

// --- Taking an action ----------------------------------------------------------------------------------------------

/// The `Act` frame: the action chosen with priority is its first answer.
pub fn act(cx: &mut Cx, player: PlayerId) -> R<Flow> {
    let Answer::Act(action) = cx.ask_core(player, RequestKind::Priority { options: Vec::new() }, false)? else {
        unreachable!("an Act frame starts with its action");
    };
    let source = action.source.expect("an action names its source");
    match action.kind {
        ActionKind::PlayLand => {
            // The land play is counted as the land is played (CR 305.1), before anything that watches it triggers:
            // Fastbond's "if it wasn't the first land you played this turn" counts this one.
            cx.st().update_player(player, |p| p.lands_played += 1);
            // A way a permanent gave is noted in the history against it (see the cast's note), and what it grants is the
            // land's from the moment it is on the battlefield.
            let granted = granted_slot(action.index).map(|(k, _)| {
                let (granter, _, way) = granted_ways_of(cx.cards(), cx.s(), &cx.g.log)[k];
                (granter, way)
            });
            if let Some((granter, _)) = granted {
                let ability = AbilityRef { card: cx.s().obj(granter).card, index: GRANTED_WAY_USED };
                cx.record(Event::AbilityActivated { source: granter, ability, player });
            }
            if action.index == WAY_BACK_FACE {
                // A modal double-faced card played as its back face enters as that face (CR 712.14a).
                ops::put_onto_battlefield_as(cx, source, player, false, true, ZoneReason::PlayLand)?;
            } else {
                let moved = ops::put_onto_battlefield(cx, &[(source, player, false)], ZoneReason::PlayLand)?;
                if let Some((granter, way)) = granted
                    && let Some(n) = way.gains
                    && let Some(&land) = moved.first()
                {
                    let ability = AbilityRef::delayed(cx.s().obj(granter).card, n);
                    Game::add_effect(cx, land, land, EffectKind::AddAbility(ability), Duration::Indefinite);
                }
            }
        }
        ActionKind::CastSpell => {
            // Cast from where a permission allows (CR 601.3), for the cost it names instead of the mana cost: the card, or its
            // Adventure or Omen, which is casting the card too (CR 715.3, 720.3).
            let permitted = match action.index {
                0 => permission_for(cx, source, player).filter(|p| !p.grant.adventure_only),
                WAY_ADVENTURE if !cx.s().obj(source).zone.is(ZoneKind::Hand) => permission_for(cx, source, player),
                WAY_OMEN | WAY_SPLIT_LEFT | WAY_SPLIT_RIGHT | WAY_BACK_FACE if !cx.s().obj(source).zone.is(ZoneKind::Hand) => {
                    permission_for(cx, source, player).filter(|p| !p.grant.adventure_only)
                }
                _ => None,
            };
            let instead = permitted.and_then(|p| p.grant.instead);
            let any_type = permitted.is_some_and(|p| p.grant.any_type);
            let more = permitted.map_or(0, |p| p.grant.more);
            // Life equal to its mana value instead of the mana cost (CR 118.9, 119.4): by a permission that says so, of the face
            // cast (CR 715.3a), or from the top of a library by a permanent's rule (Bolas's Citadel).
            let def = cx.card(cx.s().obj(source).card);
            let (instead, life) = if permitted.is_some_and(|p| p.grant.life_for_mana_value) {
                (Some(ManaCost::ZERO), life_for_mana_value(face_cast(def, action.index)).life)
            } else if cx.s().obj(source).zone.is(ZoneKind::Library) && land_rules(cx, player).top_for_life {
                (Some(ManaCost::ZERO), life_for_mana_value(def).life)
            } else {
                (instead, 0)
            };
            // A flashback a permission gives (Snapcaster Mage, CR 702.34a).
            let flashback = permitted.is_some_and(|p| p.grant.exile_after);
            cast(cx, player, source, action.index, &action.targets, CastTerms { instead, any_type, life, more, less: 0, exile_after: false, flashback, copy: false, way: None }, false)?
        }
        ActionKind::ActivateAbility if action.index >= PLAYER_MANA_ABILITY => {
            activate_mana(cx, player, source, action.index, action.color, false, &[])?;
        }
        ActionKind::ActivateAbility => {
            let ability = cx.chars(source).abilities().nth(action.index as usize).expect("ability");
            let def = activated(cx, ability).expect("an activated ability");
            if def.is_mana_ability() {
                activate_mana(cx, player, source, action.index, action.color, action.mana == Some(Mana::C), &action.targets)?;
            } else if def.special_action {
                special_action(cx, player, source, ability)?;
            } else {
                activate(cx, player, source, ability, action.index, &action.targets)?;
            }
        }
        _ => unreachable!("not a priority action"),
    }
    // Whoever acted receives priority again, and the run of passes is broken (CR 117.3c).
    cx.st().update_turn(|t| {
        t.priority = player;
        t.passes = 0;
    });
    Ok(Flow::Done)
}

pub(crate) fn stack_item(kind: StackKind, object: ObjectId, ability: AbilityRef, controller: PlayerId, targets: &[Target]) -> StackItem {
    let mut item = StackItem {
        kind,
        object,
        ability,
        controller,
        targets: [Target::None; cantrip_state::MAX_TARGETS],
        target_count: targets.len() as u8,
        spec_of: [0; cantrip_state::MAX_TARGETS],
        x: 0,
        kicked: 0,
        paid: 0,
        modes: 0,
        way: 0,
        choice: 0,
        subject: Target::None,
        data: [0; 2],
        members: Span::default(),
        serial: 0,
        gift: None,
        shares: [0; cantrip_state::MAX_TARGETS],
        exile_after: false,
        flashback: false,
        since: 0,
        copied: false,
    };
    item.targets[..targets.len()].copy_from_slice(targets);
    item
}

/// "The next spell you cast this turn can't be countered" (CR 611.2c, 601.2, 701.5a): each such effect of `player` marks
/// this spell as it is cast, before anyone has priority, and is used up; two before one spell both apply to it.
fn apply_next_spell_effects(cx: &mut Cx, player: PlayerId, spell: ObjectId) {
    let ours = |e: &cantrip_state::ContinuousEffect| e.kind == EffectKind::NextSpellCantBeCountered && e.controller == player;
    if !cx.s().effects.iter().any(ours) {
        return;
    }
    cx.st().update_obj(spell, |o| o.marks.cant_be_countered = true);
    let keep: Vec<bool> = cx.s().effects.iter().map(|e| !ours(e)).collect();
    cx.st().effects_keep(&keep);
}

/// Casting a spell (CR 601.2): it moves to the stack, its costs are chosen and paid, and it becomes cast. Cast as an
/// effect resolves (`resolving`, CR 608.2g) its targets are chosen here, not announced with an action, and `instead`
/// is the cost it is cast for rather than its mana cost (CR 118.9).
fn cast(
    cx: &mut Cx,
    player: PlayerId,
    card: ObjectId,
    way: u8,
    targets: &[Target],
    terms: CastTerms,
    resolving: bool,
) -> R<()> {
    let CastTerms { instead, any_type, exile_after, flashback, life, more, less, copy, way: _ } = terms;
    // A way a permanent grants (CR 118.9) names the face cast as well: the cast goes on as that face's own.
    let granted = granted_slot(way);
    let way = granted.map_or(way, |(_, face)| granted_face_way(face));
    let printed = cx.s().obj(card).card;
    let whole = cx.card(printed);
    // Cast as its Adventure or its Omen (CR 715.3, 720.3), it is that spell and nothing else on the stack (CR 715.3b,
    // 720.3b).
    let inset = whole.inset_face(way);
    // Retrace (CR 702.81a): cast from the graveyard for its ordinary cost by a card's say-so that is not its own, a land
    // card discarded in addition to paying its other costs.
    let retraced = way == 0
        && granted.is_none()
        && instead.is_none()
        && cx.s().obj(card).zone.is(ZoneKind::Graveyard)
        && !whole.keywords.has(Keyword::CastFromYourGraveyard)
        && cx.chars(card).has(Keyword::Retrace);
    // Cast as its back face (CR 712.8a), a modal double-faced card is that face and nothing else on the stack.
    let back = if way == WAY_BACK_FACE || way == WAY_TRANSFORMED { whole.back } else { None };
    // Cast bestowed (CR 702.103b), it is an Aura spell and nothing else on the stack.
    let bestow = if way == WAY_BESTOW { whole.bestow } else { None };
    // Cast as one half of a split card (CR 709.3), it is that half and nothing else on the stack.
    let half = match (way, whole.split) {
        (WAY_SPLIT_LEFT, Some((left, _))) => Some(left),
        (WAY_SPLIT_RIGHT, Some((_, right))) => Some(right),
        _ => None,
    };
    let def = inset.or(back).or(bestow).or(half).unwrap_or(whole);
    let face = cx.card_id(def);
    // Cast from exile by a permanent's permission (CR 601.3), it owes what the permission adds (CR 118.8, 601.2f): found now, as
    // the cast begins, so the permanent need not stay while the spell is paid for (Dawnhand Dissident's ruling).
    let exile_permit = if resolving || copy { None } else { exile_cast_permission(cx, card, inset.or(half).or(back).map(|_| face), player) };
    // Flashback (CR 702.34a) a permission or a permanent gave, or aftermath (CR 702.127a): exiled instead of put anywhere
    // else as it leaves the stack. The way a permanent gave is read now: the spell keeps it if the permanent leaves.
    let flashback = flashback || def.aftermath || granted.is_some_and(|(k, _)| granted_ways_of(cx.cards(), cx.s(), &cx.g.log)[k].2.flashback);
    let mut cost = match (way, instead) {
        (_, Some(instead)) => Cost::mana(instead),
        _ if granted.is_some() => {
            let (k, _) = granted.expect("a granted way");
            let (granter, controller, rule) = granted_ways_of(cx.cards(), cx.s(), &cx.g.log)[k];
            let asker = Asker { source: granter, controller };
            // Judged on the face being cast, as the offer was.
            let view = AsFace { cx: &*cx, spell: card, face };
            let q: &dyn Query = if way == 0 { &*cx } else { &view };
            (rule.cost)(q, &asker, card, player).unwrap_or(Cost::mana(def.cost))
        }
        (0, _) => Cost::mana(def.cost),
        (WAY_ADVENTURE | WAY_OMEN | WAY_BACK_FACE | WAY_BESTOW | WAY_SPLIT_LEFT | WAY_SPLIT_RIGHT, _) => Cost::mana(def.cost),
        (WAY_OVERLOAD, _) => Cost::mana(whole.overload.unwrap_or(def.cost)),
        _ => whole.ways[way as usize - 1].cost,
    };
    // The {X} a granted way's cost has: the card's, where the cost is or includes its mana cost; none where it is cast
    // without paying its mana cost (CR 107.3b).
    let granted_x = granted.map(|_| cost.mana.x);
    cost.life = cost.life.saturating_add(life);
    cost.mana.generic = cost.mana.generic.saturating_add(more);

    // CR 601.2a: the card moves to the stack first. Casting a prepare spell's copy unprepares its permanent
    // (CR 722.3c).
    if let Some(from) = cx.s().obj(card).prepared_from {
        cx.st().update_obj(from, |o| o.prepared = false);
    }
    let from = cx.s().obj(card).zone.kind();
    let cast_from = from;
    // CR 307.1: whether a sorcery could have been cast now, read before the spell goes on the stack.
    let untimely = !(cx.s().turn.active == player && matches!(cx.s().turn.step(), Step::PrecombatMain | Step::PostcombatMain) && cx.s().stack.is_empty());
    let spell = ops::move_to_for(cx, card, Zone::STACK, ZoneReason::Cast);
    cx.st().update_obj(spell, |o| {
        o.cast_from = Some(from);
        o.marks.cast_untimely = untimely;
    });
    // A way a permanent gave is noted in the history against it, for a permission "once during each of your turns"
    // (CR 601.3): an activation of `GRANTED_WAY_USED`, past any ability a card prints.
    if let Some((k, _)) = granted {
        let (granter, _, _) = granted_ways_of(cx.cards(), cx.s(), &cx.g.log)[k];
        let ability = AbilityRef { card: cx.s().obj(granter).card, index: GRANTED_WAY_USED };
        cx.record(Event::AbilityActivated { source: granter, ability, player });
        // "If you do, it gains ...": the spell carries it to the permanent it becomes.
        if let Some(n) = granted_ways_of(cx.cards(), cx.s(), &cx.g.log)[k].2.gains {
            let ability = AbilityRef::delayed(cx.s().obj(granter).card, n);
            cx.st().update_obj(spell, |o| o.marks.gains = Some(ability));
        }
    }
    // A permission is used up by the cast (CR 400.7: the card is a new object).
    if !cx.s().permissions.is_empty() {
        cx.st().permissions_retain(|p| p.card != card);
    }
    if inset.is_some() || back.is_some() || bestow.is_some() || half.is_some() {
        cx.st().update_obj(spell, |o| {
            o.copy = Some(Copiable::of(face))
        });
    }
    let mut item = stack_item(StackKind::Spell, spell, AbilityRef { card: face, index: u8::MAX }, player, targets);
    item.way = way;
    item.exile_after = exile_after;
    item.flashback = flashback;
    // Overload (CR 702.96a): "each" for "target", so nothing is targeted.
    let specs: &[TargetSpec] = if way == WAY_OVERLOAD { &[] } else { def.spell_targets() };
    // CR 118.8: an additional cost is owed with every way of casting it; its mana joins the total.
    let mut extra = def.additional_cost.unwrap_or(Cost::NONE);
    if let Some((_, permission)) = exile_permit {
        extra = extra.and(permission.additional);
    }
    if retraced {
        extra = extra.and(RETRACE);
    }
    cost.mana = add_cost(cost.mana, extra.mana);

    // CR 601.2b: a modal spell's modes are chosen first (CR 700.2), each adding its cost (spree, escalate) - unless how
    // many it chooses is defined by whether its optional additional cost is paid or it is kicked, which is announced in
    // the same step: then that is announced first and the modes after it (below).
    let mut mask = 0u8;
    let modes_after_costs = def.modal.is_some_and(|modal| modal.when_paid.is_some());
    if let Some(modal) = def.modal
        && !modes_after_costs
    {
        mask = cast_modes(cx, player, spell, &mut item, modal, (modal.min, modal.max), &mut cost, &mut extra)?;
    }

    // CR 601.2b: an additional cost that is one of several is chosen now, before X (as the C# engine asks it).
    if !def.additional_choice.is_empty() {
        let pick = choose_additional(cx, player, spell, face, def)?;
        // With what is owed already (a permission's additional cost, CR 118.8), not instead of it.
        extra = extra.and(pick);
        cost.mana = add_cost(cost.mana, pick.mana);
    }

    // CR 601.2b: an optional cost that is not mana (teamwork) is announced like a kicker: paid or not, before any target.
    if let Some(optional) = def.optional_cost
        && announce_optional(cx, player, spell, face, def, &optional)?
    {
        extra = extra.and(optional.cost);
        cost.mana = add_cost(cost.mana, optional.cost.mana);
        item.paid = 1;
        cx.st().update_obj(spell, |o| o.paid = 1);
    }

    // CR 702.180b, 601.2b: the creature harmonize taps is chosen as the harmonize cost is, before any target.
    let reducer = choose_reducer(cx, player, spell, &cost)?;

    // CR 601.2b: X, and how many times it is kicked, are announced, and the total cost follows (CR 601.2f).
    // CR 107.3a, 601.2b: X is announced for a cost that includes it. A way that casts the face itself (an Adventure,
    // CR 715.3b; an Omen, CR 720.3b; a back face, CR 712.8a; one half of a split card, CR 709.3) pays that face's own mana
    // cost, X included;
    // an alternative cost is not that cost, so X is 0 (CR 107.3b).
    // A way a permanent grants (Lurrus, Underworld Breach's escape, mayhem) casts the face for its mana cost, or a cost that
    // includes it, so X is announced (CR 107.3a); one that casts it without paying its mana cost (Omniscience) leaves X 0
    // (CR 107.3b). Cast without paying its mana cost (`instead`, CR 118.9), X is 0 (CR 107.3b).
    // A cost paid instead that includes {X} (a miracle cost that is the card's mana cost reduced) announces it (CR 107.3a).
    let x_symbols = match (instead, granted_x) {
        (Some(instead), _) => instead.x,
        (None, Some(x)) => x,
        (None, None) if way == 0 || matches!(way, WAY_ADVENTURE | WAY_OMEN | WAY_BACK_FACE | WAY_SPLIT_LEFT | WAY_SPLIT_RIGHT) => def.cost.x,
        (None, None) => 0,
    };
    let sacrifices_x = matches!(extra.variable, Some(VariableCost::SacrificeX(_)));
    if x_symbols > 0 || def.kicker.is_some() || extra.pay_x_life || sacrifices_x {
        let base = cost.mana;
        // X paid in life is announced up to the life there is (CR 119.4), and X creatures sacrificed up to the creatures
        // there are (CR 601.2b).
        let cap = if extra.pay_x_life {
            cx.s().player(player).life.clamp(0, 1000) as u16
        } else if let Some(variable) = extra.variable.filter(|_| sacrifices_x) {
            variable_candidates(cx, spell, player, variable).len() as u16
        } else {
            1000
        };
        let n = announce(cx, player, spell, base, def.kicker, x_symbols, cap)?;
        if let Some(k) = def.kicker {
            if k.cost.x > 0 {
                // Kicker {X} (CR 702.33a, 107.3): X is the number announced, and kicked when it is not 0.
                cost.mana = kicker_total(cost.mana, k, n);
                item.kicked = (n > 0) as u8;
                item.x = n;
                cx.st().update_obj(spell, |o| {
                    o.kicked = (n > 0) as u8;
                    o.x = n;
                });
            } else {
                cost.mana = kicker_total(cost.mana, k, n);
                item.kicked = n as u8;
                cx.st().update_obj(spell, |o| o.kicked = n as u8);
            }
        } else {
            // What a cost paid instead was reduced by that its own generic mana could not take, off X's (CR 118.7a).
            let xs = (n as u32 * x_symbols as u32).min(255);
            cost.mana.generic = cost.mana.generic.saturating_add((xs - xs.min(less as u32)) as u8);
            item.x = n;
            cx.st().update_obj(spell, |o| o.x = n);
        }
    }
    // CR 601.2b, 700.2a: the modes of a spell that chooses "both instead" when its additional cost was paid (or "any
    // number instead" when it was kicked), now that the intention to pay is announced.
    if let Some(modal) = def.modal
        && let Some(paid_range) = modal.when_paid
    {
        let range = if item.paid > 0 || item.kicked > 0 { paid_range } else { (modal.min, modal.max) };
        mask = cast_modes(cx, player, spell, &mut item, modal, range, &mut cost, &mut extra)?;
    }
    // CR 702.47a-b: as an Arcane spell is cast, cards with splice onto Arcane may be revealed from the hand and their
    // cost paid; their spell abilities are added to this spell's, after its own, in the order chosen.
    if cx.chars(spell).has_subtype(Subtype::Arcane) {
        let hand = cx.s().zone(Zone::hand(player)).to_vec();
        let eligible: Vec<ObjectId> = hand
            .iter()
            .copied()
            .filter(|&c| cx.card(cx.s().obj(c).card).splice.is_some())
            .collect();
        if !eligible.is_empty() {
            let count = eligible.len().min(2) as u8;
            let ask = RequestKind::ChooseCards {
                source: spell,
                purpose: Purpose::Splice,
                cards: hand,
                eligible: eligible.clone(),
                count,
                up_to: true, kind: "", total: None };
            let spliced = match cx.ask_core(player, ask, true)? {
                Answer::Cards(cards) => cards,
                _ => return Err(Stop::Cancel),
            };
            if !spliced.is_empty() {
                Game::reveal(cx, player, &spliced);
            }
            for (n, c) in spliced.into_iter().take(2).enumerate() {
                let splice_card = cx.s().obj(c).card;
                cost.mana = add_cost(cost.mana, cx.card(splice_card).splice.expect("a splice card"));
                item.data[n] = splice_card.0 as u32 + 1;
            }
        }
    }
    let x_life = if extra.pay_x_life { item.x } else { 0 };

    // CR 601.2b, 107.4f: which Phyrexian symbols are paid with life is announced before any target.
    let life = announce_life(cx, player, spell, &mut cost.mana)?;
    // CR 601.2b, 702.174a: gift is an optional additional cost, announced, with the opponent it is promised to, before
    // the targets (CR 702.174m): what may be targeted can depend on it. The opponent is the only one without asking.
    if let Some(gift) = def.gift
        && cx.s().opponents(player).any(|p| !cx.s().player(p).lost)
        && matches!(cx.ask_core(player, RequestKind::May { source: spell, purpose: Purpose::PromiseGift(gift.kind) }, true)?, Answer::Yes)
    {
        let opponents: Vec<PlayerId> = cx.s().opponents(player).collect();
        let to = if opponents.len() == 1 {
            opponents[0]
        } else {
            let ask = RequestKind::ChoosePlayer { source: spell, candidates: opponents, verb: "Gift", amount: 0 };
            match cx.ask_core(player, ask, true)? {
                Answer::Player(p) => p,
                _ => return Err(Stop::Cancel),
            }
        };
        item.gift = Some(to);
        cx.st().update_obj(spell, |o| o.gift = Some(to));
    }
    item.since = cx.g.log.len() as u32;
    cx.st().stack_push(item);

    // CR 601.2c: targets slot by slot, where they were not announced with the action; a slot counted by X or by the
    // times it was kicked is judged by what was announced.
    let counted = if item.x > 0 { item.x } else { item.kicked as u16 };
    let mut targets: Vec<Target> = targets.to_vec();
    let mut of: Vec<u8> = Vec::new();
    if let Some(modal) = def.modal {
        (targets, of) = choose_slots(cx, player, spell, &mode_specs(modal, mask), counted, item.kicked)?;
        set_item_targets(cx, &targets, &of);
    } else if let Some(divide) = def.divide
        && let Some(spec) = specs.first()
    {
        // CR 601.2d: the targets and their shares of a total, in one question; the total may be X.
        let total = match divide.of_x {
            Some(of_x) => of_x(item.x),
            None if divide.total == TargetSpec::COUNTED => item.x,
            None => divide.total as u16,
        };
        match ask_divided(cx, player, spell, spec, total, true)? {
            Divided::Chosen(chosen, shares) => {
                targets = chosen;
                set_item_targets(cx, &targets, &[]);
                set_item_shares(cx, &shares);
            }
            Divided::Skipped => {}
            // A total that needs a target and has none to give it is not a legal announcement.
            Divided::Impossible => return Err(Stop::Cancel),
        }
    } else if (resolving || !targets_with_cast(def)) && !specs.is_empty() {
        (targets, of) = choose_slots(cx, player, spell, specs, counted, item.kicked)?;
        set_item_targets(cx, &targets, &of);
    }

    // CR 702.47c, 601.2c: the targets of spliced text are chosen after the spell's own, in the order the cards were spliced.
    if item.data.iter().any(|&d| d != 0) {
        let mut all = targets.clone();
        // Targets announced with the cast are the lone slot's, the first.
        of.resize(targets.len(), 0);
        // A spliced card's slots are numbered on past the spell's own specs, in splice order (`resolve::slot_spec`), each
        // target recording its slot, so the spell's own slots need hold no set number of targets.
        let mut next = specs.len() as u8;
        for &spliced in item.data.iter().filter(|&&d| d != 0) {
            let specs = cx.card(CardId(spliced as u16 - 1)).spell.map_or(&[][..], |s| s.targets);
            if !specs.is_empty() {
                let (chosen, chosen_of) = choose_slots(cx, player, spell, specs, counted, item.kicked)?;
                of.extend(chosen_of.iter().map(|&s| next + s));
                all.extend(chosen);
            }
            next += specs.len() as u8;
        }
        if all.len() != targets.len() {
            targets = all;
            set_item_targets(cx, &targets, &of);
        }
    }

    // CR 601.2f: the total cost, with what increases or reduces it, once the targets are announced (CR 601.2c): a change may
    // read them ("costs {2} less if it targets ..."). It is locked in here, so nothing that happens to a target later changes
    // what is paid. A change may add life as an additional cost (CR 118.8: "cost an additional 3 life to cast"), part of the
    // total, which a player can pay only with at least that much life, the rest of the life the total asks included
    // (CR 119.4, 118.3, 601.2h: no partial payment): a cast that cannot is taken back.
    let added_life;
    (cost.mana, added_life) = apply_cost_changes_total(cx, player, Costed::Spell(spell), cost.mana, &targets);
    if added_life > 0 {
        cost.life = (cost.life as u32 + added_life).min(255) as u8;
        let owed = cost.life as i32 + extra.life as i32 + life as i32 + x_life as i32;
        if cx.s().player(player).life < owed {
            return Err(Stop::Cancel);
        }
    }
    // CR 702.180a, 601.2f: the total cost comes down by generic mana equal to the tapped creature's power, read as the total
    // cost is determined, after what raises it; colored mana never comes down.
    if let Some(creature) = reducer {
        cost.mana.generic = cost.mana.generic.saturating_sub(reduction_of(cx, creature));
    }

    // Mana of any type may pay for it (CR 609.4b).
    if any_type {
        cost.mana = ManaCost { generic: cost.mana.mana_value().min(255) as u8, ..ManaCost::ZERO };
    }

    // CR 601.2f-h: permanents the cost names are chosen, the mana paid, then the rest of the cost; an additional
    // cost's choices come once the mana is paid (CR 601.2h), as the C# engine asks them, and so do tapped permanents
    // (web-slinging's), which a mana ability activated while paying may have tapped (CR 601.2g).
    let late = permanents_after_mana(&cost);
    let mut chosen = if late { Vec::new() } else { choose_permanents(cx, player, spell, &cost)? };
    let discards = choose_discards(cx, player, spell, &cost)?;
    let mut exiles = choose_exiles(cx, player, spell, &cost)?;
    let (way_sacrifices, way_exiles) = choose_variable(cx, player, spell, &cost, item.x)?;
    exiles.extend(way_exiles);
    let hand_exiles = choose_hand_exiles(cx, player, spell, &cost)?;
    // CR 702.119a: emerge's mana comes down by the mana value of what was just chosen to be sacrificed. Nothing has
    // moved yet, so the permanents are still there to be read.
    if cost.emerge {
        let reduction: u32 = chosen.iter().map(|&id| cx.mana_value(id)).sum();
        cost.mana.generic = cost.mana.generic.saturating_sub(reduction.min(255) as u8);
    }
    // CR 701.67b: the tapping pays only the generic mana of the waterbend cost, whatever else the total has.
    let waterbend = def.kicker.filter(|k| k.waterbend).map_or(0, |k| k.cost.generic.saturating_mul(item.kicked));
    // CR 702.180b, 601.2h: the creature is tapped as the total cost is paid. It is tapped before the mana is, since a cost
    // may be paid in any order, so it cannot also be tapped for mana: the tap it was chosen for is its own.
    if let Some(creature) = reducer {
        ops::tap(cx, creature);
    }
    let life_owed = cost.life as u32 + extra.life as u32 + life as u32 + x_life as u32;
    let helpers = Helpers { with: def.pay_with, waterbend, creatures: false, no_mana: def.keywords.has(Keyword::CantSpendManaToCast), purpose: Paying::Casting, reserved: &exiles, attackers: &[], life: life_owed };
    let (spent, restrictions) = pay_mana_used(cx, player, spell, cost.mana, &helpers)?;
    // CR 106.6: a creature spell paid for with mana that gives it haste has haste once it resolves; a spell paid for with
    // mana that makes it uncounterable can't be countered.
    if restrictions & MANA_GIVES_HASTE != 0 && cx.chars(spell).is(CardType::Creature) {
        cx.st().update_obj(spell, |o| o.marks.gains_haste = true);
    }
    if restrictions & MANA_MAKES_UNCOUNTERABLE != 0 {
        cx.st().update_obj(spell, |o| o.marks.cant_be_countered = true);
    }
    apply_next_spell_effects(cx, player, spell);
    if late {
        chosen = choose_permanents(cx, player, spell, &cost)?;
    }
    let extra_chosen = choose_permanents(cx, player, spell, &extra)?;
    let beheld = choose_behold(cx, player, spell, &cost)?;
    let extra_beheld = choose_behold(cx, player, spell, &extra)?;
    let blighted = choose_blight(cx, player, spell, &cost)?;
    let extra_blighted = choose_blight(cx, player, spell, &extra)?;
    let removed = choose_counters_among(cx, player, spell, &cost)?;
    let extra_removed = choose_counters_among(cx, player, spell, &extra)?;
    let (extra_sacrifices, extra_exiles) = choose_variable(cx, player, spell, &extra, item.x)?;
    // An optional cost that sacrifices one or more: how many were is how many times it was paid (Plumb the Forbidden's
    // linked trigger reads it).
    if matches!(extra.variable, Some(VariableCost::SacrificeOneOrMore(_))) {
        let n = extra_sacrifices.len().min(255) as u8;
        let at = cx.s().stack.len() - 1;
        let mut with = cx.s().stack[at];
        with.paid = n;
        cx.st().stack_set(at, with);
        cx.st().update_obj(spell, |o| o.paid = n);
    }
    let extra_discards = choose_discards(cx, player, spell, &extra)?;
    let extra_tappers = choose_tappers(cx, player, spell, &extra)?;
    let extra_evidence = choose_exiles(cx, player, spell, &extra)?;
    // Sneak (CR 702.190b): the permanent the spell becomes attacks what the creature its cost returns to hand is attacking
    // as the cost is paid, kept on the spell for its resolution. An instant or sorcery becomes no permanent, and keeps none.
    if granted.is_none()
        && instead.is_none()
        && way >= 1
        && whole.ways.get(way as usize - 1).is_some_and(|w| w.enters_attacking)
        && !cx.chars(spell).is(CardType::Instant)
        && !cx.chars(spell).is(CardType::Sorcery)
        && let Some(attack) = chosen.first().and_then(|&c| cx.s().attackers.iter().find(|a| a.id == c)).copied()
    {
        let at = cx.s().stack.len() - 1;
        let mut with = cx.s().stack[at];
        with.subject = attack.target;
        cx.st().stack_set(at, with);
    }
    cx.st().update_obj(spell, |o| {
        o.spent = spent;
        o.cast_way = way;
        o.x = item.x;
    });
    // Counters put on and taken off permanents are paid first (CR 601.2h: in any order), so a permanent the cost also
    // sacrifices leaves with them as the payment left it.
    pay_blight(cx, &cost, blighted);
    pay_blight(cx, &extra, extra_blighted);
    pay_counters_among(cx, &removed);
    pay_counters_among(cx, &extra_removed);
    pay_permanents(cx, &cost, &chosen);
    pay_discards(cx, &discards);
    pay_exiles(cx, &exiles);
    pay_exiles(cx, &hand_exiles);
    ops::sacrifice(cx, &way_sacrifices);
    pay_exiles(cx, &extra_exiles);
    pay_exiles(cx, &extra_evidence);
    pay_life_and_counters(cx, player, spell, &cost);
    pay_permanents(cx, &extra, &extra_chosen);
    pay_behold(cx, player, &beheld);
    pay_behold(cx, player, &extra_beheld);
    ops::sacrifice(cx, &extra_sacrifices);
    pay_discards(cx, &extra_discards);
    pay_life_and_counters(cx, player, spell, &extra);
    pay_exert_and_tappers(cx, spell, &extra, &extra_tappers);
    // CR 119.4: the life paid for Phyrexian symbols and for X, with the rest of the cost.
    if life as u16 + x_life > 0 {
        ops::change_life(cx, player, -(life as i32 + x_life as i32));
    }
    let targets = &targets[..];

    // CR 608.3g: what the way cast for leaves behind to happen at the next end step, aimed at the permanent the spell
    // becomes, which `Obj::became` follows.
    for &(rider_way, delayed) in def.end_step_riders {
        if rider_way == way {
            let since = cx.g.log.len() as u32;
            cx.st().delayed_push(cantrip_state::Delayed {
                when: DelayedWhen::Step { step: Step::End, on_turn_of: None },
                source: spell,
                controller: player,
                ability: AbilityRef::delayed(face, delayed),
                subject: Target::Object(spell),
                members: cantrip_state::Span::default(),
                since,
            });
        }
    }

    // CR 601.2i: it becomes cast.
    let as_cast = cx.chars(spell);
    let x = cx.s().stack.iter().rev().find(|i| i.object == spell).map_or(0, |i| i.x);
    let value = (cx.card(as_cast.card).cost.mana_value() + x as u32).min(255) as u8;
    // "As you cast this spell": the board once the costs are paid (CR 601.2i), asked of the face being cast.
    let fact = def.cast_fact.map_or(0, |f| f(cx, &Asker { source: spell, controller: player }));
    cx.record(Event::Cast(Cast {
        spell,
        player,
        types: as_cast.types,
        colors: as_cast.colors,
        value,
        from: cast_from,
        way,
        fact,
    }));
    // "Until this card is cast from exile" (CR 611.2a, 400.7) ends now, as the card that was in exile becomes cast: the
    // effects about it were there while its costs were paid.
    if cast_from == ZoneKind::Exile && cx.s().effects.iter().any(|e| e.duration == Duration::UntilCastFromExile(card)) {
        cx.st().effects_retain(|e| e.duration != Duration::UntilCastFromExile(card));
    }
    let nth = cx.history().casts_by(player).count().min(255) as u8;
    cx.emit(Event::SpellCast { spell, player, nth });
    ops::became_targets(cx, targets.iter().copied(), spell, player);
    ops::commit_crime(cx, player, spell, targets);
    Ok(())
}

/// Whether the resolving effect's controller could cast `card` now (CR 608.2g): it is not a land and every target it
/// needs has a legal choice.
pub fn could_cast_during_resolution(cx: &Cx, card: ObjectId) -> bool {
    !cast_ways_during_resolution(cx, card).is_empty()
}

/// Each way the resolving effect's controller could cast `card` now (CR 608.2g), each face judged alone (CR 709.3a, 715.3a,
/// 712.8a): the card itself unless it is a land or a split card (whose halves are what is cast, CR 709.3), a split card's two
/// halves (an aftermath half only from a graveyard, CR 702.127a), an Adventurer's Adventure, an omen card's Omen (CR 720.3a),
/// and a modal double-faced card's
/// back face. Fuse is not among them: it applies only in a hand (CR 702.102a).
pub fn cast_ways_during_resolution(cx: &Cx, card: ObjectId) -> Vec<CastWay> {
    let Some(player) = cx.s().stack.last().map(|i| i.controller) else { return Vec::new() };
    let obj = cx.s().obj(card);
    if !live(cx.s(), card) || matches!(obj.zone.kind(), ZoneKind::Battlefield | ZoneKind::Stack) {
        return Vec::new();
    }
    if cast_barred(cx, player).contains(obj.zone.kind()) {
        return Vec::new();
    }
    let def = cx.card(obj.card);
    let asker = Asker { source: card, controller: player };
    let castable = |face: &'static CardDef| {
        cast_only_if_met(cx, face, &asker)
            && match face.modal {
                Some(modal) => modes_possible(cx, &asker, modal),
                None => slots_satisfiable(cx, &asker, face.spell_targets()),
            }
    };
    let mut ways = Vec::new();
    if let Some((left, right)) = def.split {
        for (way, half) in [(WAY_SPLIT_LEFT, left), (WAY_SPLIT_RIGHT, right)] {
            if (!half.aftermath || obj.zone.is(ZoneKind::Graveyard)) && castable(half) {
                ways.push(CastWay { way, face: cx.card_id(half), mana_value: half.cost.mana_value() });
            }
        }
        return ways;
    }
    if !def.is(CardType::Land) && castable(def) {
        ways.push(CastWay { way: 0, face: obj.card, mana_value: cx.mana_value(card) });
    }
    for (way, _, inset) in def.inset_faces() {
        if castable(inset) {
            ways.push(CastWay { way, face: cx.card_id(inset), mana_value: inset.cost.mana_value() });
        }
    }
    if let Some(back) = def.back
        && back.has_cost
        && castable(back)
    {
        ways.push(CastWay { way: WAY_BACK_FACE, face: cx.card_id(back), mana_value: back.cost.mana_value() });
    }
    ways
}

/// Whether the resolving effect's controller could play `card` as a land now (CR 305.1, 608.2g): it is a land card not
/// on the battlefield or the stack, it is their own turn (CR 305.3) and they have a land play left (CR 305.2). The phase,
/// the stack and priority are not asked: the effect itself is the permission to play it now.
pub fn could_play_land_during_resolution(cx: &Cx, card: ObjectId) -> bool {
    let Some(player) = cx.s().stack.last().map(|i| i.controller) else { return false };
    let obj = cx.s().obj(card);
    if !live(cx.s(), card) || matches!(obj.zone.kind(), ZoneKind::Battlefield | ZoneKind::Stack) || cx.s().turn.active != player {
        return false;
    }
    if !cx.card(obj.card).is(CardType::Land) || cx.s().player(player).lost {
        return false;
    }
    let LandRules { extra, unlimited, .. } = land_rules(cx, player);
    unlimited || (cx.s().player(player).lands_played as u32) < 1 + extra
}

/// Plays `card` as a land as an effect resolves (CR 305.1, 608.2g): put onto the battlefield as a land play, counted
/// against the turn's land plays as the special action is (CR 305.2a). The new object, or nothing if it could not be.
pub fn play_land_during_resolution(cx: &mut Cx, card: ObjectId) -> R<Option<ObjectId>> {
    if !could_play_land_during_resolution(cx, card) {
        return Ok(None);
    }
    let player = cx.s().stack.last().expect("a resolving item").controller;
    cx.st().update_player(player, |p| p.lands_played += 1);
    let moved = ops::put_onto_battlefield(cx, &[(card, player, false)], ZoneReason::PlayLand)?;
    Ok(moved.first().copied())
}

/// Casts `card` as an effect resolves (CR 608.2g). A cast given up (CR 733.1) is undone, and nothing happens.
pub fn cast_during_resolution(cx: &mut Cx, card: ObjectId, terms: CastTerms) -> R<bool> {
    let ways = cast_ways_during_resolution(cx, card);
    let player = cx.s().stack.last().expect("a resolving item").controller;
    // CR 709.3, 715.3, 712.8a: which face is cast is chosen as it is cast, before it goes on the stack.
    let way = match (terms.way, ways.len()) {
        (_, 0) => return Ok(false),
        (Some(way), _) if ways.iter().any(|w| w.way == way) => way,
        (Some(_), _) => return Ok(false),
        (None, 1) => ways[0].way,
        (None, _) => {
            let modes = ways.iter().map(|w| Label::Face(w.face)).collect();
            let ask = RequestKind::ChooseModes {
                source: card,
                modes,
                eligible: (0..ways.len() as u8).collect(),
                count: 1,
                at_most: 0,
                affordable: Vec::new(),
                additional_cost: false,
                repeat: false,
            };
            match cx.ask_core(player, ask, false)? {
                Answer::Modes(chosen) => ways[chosen.first().copied().unwrap_or(0) as usize % ways.len()].way,
                _ => ways[0].way,
            }
        }
    };
    let mark = cx.s().mark();
    let log_mark = cx.g.log.len();
    // CR 707.12: it is a copy of the card that is cast, and the card stays where it is. A cast given up reverts the copy too.
    let card = if terms.copy { ops::copy_card(cx, card, player) } else { card };
    match cast(cx, player, card, way, &[], terms, true) {
        Ok(()) => Ok(true),
        Err(Stop::Cancel) => {
            cx.st().revert(mark);
            cx.g.log.truncate(log_mark);
            Ok(false)
        }
        Err(stop) => Err(stop),
    }
}

/// Whether the resolving effect's controller could cast `card` transformed now (CR 608.2g, 712.11a): a double-faced
/// card that is not modal (CR 712.11c-d: its back face has no mana cost of its own), judged as that back face alone.
pub fn could_cast_transformed_during_resolution(cx: &Cx, card: ObjectId) -> bool {
    let Some(player) = cx.s().stack.last().map(|i| i.controller) else { return false };
    let obj = cx.s().obj(card);
    if !live(cx.s(), card) || matches!(obj.zone.kind(), ZoneKind::Battlefield | ZoneKind::Stack) {
        return false;
    }
    if cast_barred(cx, player).contains(obj.zone.kind()) {
        return false;
    }
    let Some(back) = cx.card(obj.card).back else { return false };
    if back.has_cost {
        return false;
    }
    let asker = Asker { source: card, controller: player };
    if !cast_only_if_met(cx, back, &asker) {
        return false;
    }
    if let Some(modal) = back.modal {
        return modes_possible(cx, &asker, modal);
    }
    slots_satisfiable(cx, &asker, back.spell_targets())
}

/// Casts `card` transformed as an effect resolves, for nothing (CR 608.2g, 712.11a, 118.9). A cast given up (CR 733.1)
/// is undone, and nothing happens.
pub fn cast_transformed_during_resolution(cx: &mut Cx, card: ObjectId) -> R<bool> {
    if !could_cast_transformed_during_resolution(cx, card) {
        return Ok(false);
    }
    let player = cx.s().stack.last().expect("a resolving item").controller;
    let mark = cx.s().mark();
    let log_mark = cx.g.log.len();
    match cast(cx, player, card, WAY_TRANSFORMED, &[], CastTerms { instead: Some(ManaCost::ZERO), ..CastTerms::default() }, true) {
        Ok(()) => Ok(true),
        Err(Stop::Cancel) => {
            cx.st().revert(mark);
            cx.g.log.truncate(log_mark);
            Ok(false)
        }
        Err(stop) => Err(stop),
    }
}

/// A slot "with total mana value N or less" (`TargetSpec::total_mana_value`; CR 601.2c, 603.3d, 202.3), `counted` its
/// count: the candidates less any worth more than the limit by itself, which no legal choice could include, and the limit
/// with each remaining candidate's mana value, index for index, for the question (the core refuses an answer over it).
/// A slot without one keeps its candidates and asks nothing of them.
pub(crate) fn within_total(cx: &Cx, spec: &TargetSpec, counted: u16, candidates: Vec<Target>) -> (Vec<Target>, Option<TotalAtMost>) {
    let Some(most) = spec.total_limit(counted) else { return (candidates, None) };
    let value = |t: Target| match t {
        Target::Object(id) => cx.mana_value(id).min(u32::from(u16::MAX)) as u16,
        _ => 0,
    };
    let (kept, values): (Vec<Target>, Vec<u16>) = candidates.into_iter().map(|t| (t, value(t))).filter(|&(_, v)| v <= most).unzip();
    (kept, Some(TotalAtMost { values, most }))
}

/// Each target slot's targets, asked in slot order (CR 601.2c), each a target of the slot's own (a player may be
/// chosen for two slots). A slot that chose nothing ("up to") keeps its place with `Target::None` among the slots, so
/// every target stays at the index of its slot (CR 601.2c, 115.1), as far as the last one that chose. Each target comes
/// with the index of the spec it was chosen for (`StackItem::spec_of`), so a slot of several targets may stand anywhere.
fn choose_slots(cx: &mut Cx, player: PlayerId, source: ObjectId, specs: &[TargetSpec], counted: u16, kicked: u8) -> R<(Vec<Target>, Vec<u8>)> {
    let asker = Asker { source, controller: player };
    let slots = crate::resolve::expand_specs(cx.s(), player, specs);
    let mut targets = Vec::new();
    let mut of = Vec::new();
    for (spec, slot_player, at) in &slots {
        // CR 702.33a, 601.2c: "if this spell was kicked, instead choose ...", the kicker announced by now.
        let spec = &spec.when_kicked(kicked);
        let before = targets.len();
        'slot: {
            let candidates =
                target_candidates_given(cx, &asker, spec, if slots.len() > 1 { &targets } else { &[] }, Target::None, *slot_player);
            let (candidates, total) = within_total(cx, spec, counted, candidates);
            let (least, most) = spec.bounds(counted);
            // Fewer legal targets than the slot's minimum is not a legal announcement (CR 601.2c, 602.2b): given up whole.
            if candidates.len() < least as usize {
                return Err(Stop::Cancel);
            }
            if candidates.is_empty() {
                if spec.required() {
                    return Err(Stop::Cancel);
                }
                break 'slot;
            }
            if most == 0 {
                break 'slot;
            }
            // A stack item holds at most MAX_TARGETS targets: a count past what is left of them is cut to it (known-issues KI-3).
            let room = cantrip_state::MAX_TARGETS.saturating_sub(targets.len());
            let count = (most as usize).min(candidates.len()).min(room) as u8;
            let ask = RequestKind::ChooseTarget { label: spec.label, slot_player: *slot_player,
                source,
                purpose: Purpose::Targets,
                candidates,
                optional: least == 0,
                least: least.min(count),
                count,
                // "Any number of targets" (CR 115.1d) is said so, not as "up to" the candidates there are.
                any_number: spec.is_any_number(),
                total,
            };
            match cx.ask_core(player, ask, true)? {
                Answer::Targets(chosen) => targets.extend(chosen),
                _ => return Err(Stop::Cancel),
            }
        }
        if slots.len() > 1 && spec.max <= 1 && targets.len() == before {
            targets.push(Target::None);
        }
        of.resize(targets.len(), *at);
    }
    while targets.last() == Some(&Target::None) {
        targets.pop();
    }
    of.truncate(targets.len());
    Ok((targets, of))
}

/// What asking a divided slot came to.
pub(crate) enum Divided {
    /// Nothing to ask or nothing chosen: a total of nothing, or an optional slot with no candidate or no target chosen.
    Skipped,
    /// The targets, and each one's share of the total.
    Chosen(Vec<Target>, Vec<u8>),
    /// A required slot with nothing to aim at (CR 601.2c).
    Impossible,
}

/// Asks for the targets of a slot that divides a total among them, and each target's share (CR 601.2d), in one
/// question, as the C# engine does: a total of nothing is not asked; with no candidate an optional slot takes no
/// target. `may_cancel` is for a spell or ability still being announced.
pub(crate) fn ask_divided(cx: &mut Cx, player: PlayerId, source: ObjectId, spec: &TargetSpec, total: u16, may_cancel: bool) -> R<Divided> {
    if total == 0 {
        return Ok(Divided::Skipped);
    }
    let asker = Asker { source, controller: player };
    let candidates = target_candidates(cx, &asker, spec);
    if candidates.is_empty() {
        return Ok(if spec.min == 0 { Divided::Skipped } else { Divided::Impossible });
    }
    let max_targets = if spec.max == TargetSpec::COUNTED { 0 } else { spec.max };
    let ask = RequestKind::ChooseDividedTargets { source, candidates, total, max_targets, optional: spec.min == 0 };
    match cx.ask_core(player, ask, may_cancel)? {
        Answer::Divided(shares) if shares.is_empty() => Ok(Divided::Skipped),
        Answer::Divided(shares) => Ok(Divided::Chosen(
            shares.iter().map(|s| s.0).collect(),
            shares.iter().map(|s| s.1.min(255) as u8).collect(),
        )),
        _ => Err(Stop::Cancel),
    }
}

/// Records the shares chosen for the item being announced, the top of the stack.
fn set_item_shares(cx: &mut Cx, shares: &[u8]) {
    let at = cx.s().stack.len() - 1;
    let mut with = cx.s().stack[at];
    with.shares[..shares.len()].copy_from_slice(shares);
    cx.st().stack_set(at, with);
}

fn add_cost(a: ManaCost, b: ManaCost) -> ManaCost {
    let mut c = a;
    c.generic += b.generic;
    for i in 0..6 {
        c.symbols[i] += b.symbols[i];
    }
    c
}

/// Records the targets chosen for the item being announced, the top of the stack, each with the spec it was chosen for
/// (`StackItem::spec_of`; none given is the first, a lone slot's).
fn set_item_targets(cx: &mut Cx, targets: &[Target], of: &[u8]) {
    let at = cx.s().stack.len() - 1;
    let mut with = cx.s().stack[at];
    with.target_count = targets.len() as u8;
    with.targets[..targets.len()].copy_from_slice(targets);
    with.spec_of = [0; cantrip_state::MAX_TARGETS];
    with.spec_of[..of.len()].copy_from_slice(of);
    cx.st().stack_set(at, with);
}

/// The target slots of the chosen modes, in printed order (CR 700.2c).
pub(crate) fn mode_specs(modal: &ModalDef, mask: u8) -> Vec<TargetSpec> {
    let mut specs = Vec::new();
    for (i, mode) in modal.modes.iter().enumerate() {
        for _ in 0..modal.times(mask, i) {
            specs.extend_from_slice(mode.targets);
        }
    }
    specs
}

/// The cost of choosing these modes on top of `base` (CR 601.2f): each mode's own cost (spree, CR 702.172a), and the
/// escalate cost once for each mode beyond the first (CR 702.120a).
fn modal_cost(modal: &ModalDef, mask: u8, base: ManaCost) -> ManaCost {
    let mut cost = base;
    for (i, mode) in modal.modes.iter().enumerate() {
        for _ in 0..modal.times(mask, i) {
            cost = add_cost(cost, mode.cost);
        }
    }
    add_cost(cost, escalate_cost(modal, mask).mana)
}

/// The escalate cost of a choice of modes (CR 702.120a): paid once for each mode beyond the first.
fn escalate_cost(modal: &ModalDef, mask: u8) -> Cost {
    match modal.escalate {
        Some(escalate) => escalate.times(modal.chosen(mask).saturating_sub(1)),
        None => Cost::NONE,
    }
}

/// A modal spell's modes as it is cast (CR 601.2b, 700.2): `range` of them chosen, each adding its cost (spree, escalate)
/// to the total. Returns the chosen modes, which the stack item now holds.
#[allow(clippy::too_many_arguments)]
fn cast_modes(
    cx: &mut Cx,
    player: PlayerId,
    spell: ObjectId,
    item: &mut StackItem,
    modal: &'static ModalDef,
    range: (u8, u8),
    cost: &mut Cost,
    extra: &mut Cost,
) -> R<u8> {
    let mask = choose_modes(cx, player, spell, item.ability, modal, range, cost.mana)?;
    cost.mana = modal_cost(modal, mask, cost.mana);
    item.modes = mask;
    // CR 702.120a: the rest of the escalate cost (a discard) is an additional cost, chosen and paid with the others.
    let mut escalate = escalate_cost(modal, mask);
    escalate.mana = ManaCost::ZERO;
    *extra = extra.and(escalate);
    Ok(mask)
}

/// Asks for a modal spell's or ability's modes (CR 601.2b, 602.2b, 700.2), as the C# engine's `ChooseModesPrompt`:
/// `range` (at least, at most) of the eligible modes, with the ones whose own cost could be paid beside `base` as advice.
/// Returns the chosen modes as a bit for each.
fn choose_modes(cx: &mut Cx, player: PlayerId, source: ObjectId, ability: AbilityRef, modal: &ModalDef, range: (u8, u8), base: ManaCost) -> R<u8> {
    let (least, max) = range;
    let asker = Asker { source, controller: player };
    let eligible = eligible_modes(cx, &asker, modal);
    let costed = modal.modes.iter().any(|m| m.cost.mana_value() > 0);
    let affordable: Vec<u8> = if costed {
        eligible.iter().copied().filter(|&i| can_afford(cx, player, source, &add_cost(base, modal.modes[i as usize].cost))).collect()
    } else {
        Vec::new()
    };
    // An escalate cost the player cannot pay narrows the range to one they can.
    let mut most = max;
    // CR 700.2: the board as the spell is cast may allow more modes than it prints.
    if let Some(up_to) = cx.card(cx.s().obj(source).card).modes_up_to {
        most = most.max(up_to(cx, &asker));
    }
    if modal.escalate.is_some() {
        let payable = |cx: &Cx, n: u8| {
            let first_n = (1u16 << n).wrapping_sub(1) as u8;
            can_afford(cx, player, source, &modal_cost(modal, first_n, base)) && can_pay_other(cx, source, player, &escalate_cost(modal, first_n))
        };
        while most > least.max(1) && !payable(cx, most) {
            most -= 1;
        }
    }
    let ask = RequestKind::ChooseModes {
        source,
        modes: (0..modal.modes.len() as u8).map(|i| Label::Mode(ability, i)).collect(),
        eligible,
        count: least,
        at_most: if most > least { most } else { 0 },
        affordable,
        additional_cost: false,
        repeat: modal.repeat,
    };
    match cx.ask_core(player, ask, true)? {
        Answer::Modes(chosen) => {
            let modes = chosen.iter().fold(0u8, |mask, &i| modal.with(mask, i as usize));
            cx.record(Event::ModesChosen { source, ability, modes });
            Ok(modes)
        }
        _ => Err(Stop::Cancel),
    }
}

/// Announces which Phyrexian symbols are paid with 2 life (CR 107.4f, 601.2b), as the C# engine's prompt: a choice of
/// up to as many as the life total covers, cheapest first. The symbols not paid with life become colored mana in
/// `cost`. Returns the life to pay with the rest of the cost.
fn announce_life(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &mut ManaCost) -> R<u8> {
    let mut symbols: Vec<Color> = Vec::new();
    for (i, &n) in cost.phyrexian.iter().enumerate() {
        for _ in 0..n {
            symbols.push(Color::ALL[i]);
        }
    }
    let mut life = 0u8;
    if !symbols.is_empty() {
        let have = cx.s().player(player).life.max(0) as usize;
        let most = symbols.len().min(have / 2) as u8;
        if most > 0 {
            // Advice: the fewest symbols that, paid with life, leave a cost the player could pay.
            let affordable = (0..=most)
                .find(|&k| {
                    let mut rest = *cost;
                    for c in &symbols[k as usize..] {
                        rest.symbols[c.mana() as usize] += 1;
                    }
                    rest.phyrexian = [0; 5];
                    can_afford(cx, player, source, &rest)
                })
                .map(|k| (0..k).collect())
                .unwrap_or_default();
            let ask = RequestKind::ChooseModes {
                source,
                modes: symbols.iter().map(|&c| Label::PayLife(c)).collect(),
                eligible: (0..symbols.len() as u8).collect(),
                count: 0,
                at_most: most,
                affordable,
                additional_cost: true,
                repeat: false,
            };
            let chosen = match cx.ask_core(player, ask, true)? {
                Answer::Modes(chosen) => chosen,
                _ => return Err(Stop::Cancel),
            };
            let mut paid = vec![false; symbols.len()];
            for i in chosen {
                paid[i as usize] = true;
                life += 2;
            }
            for (i, c) in symbols.iter().enumerate() {
                if !paid[i] {
                    cost.symbols[c.mana() as usize] += 1;
                }
            }
            cost.phyrexian = [0; 5];
            return Ok(life);
        }
        // Without the life for any, every symbol is paid with its color.
        for c in &symbols {
            cost.symbols[c.mana() as usize] += 1;
        }
        cost.phyrexian = [0; 5];
    }
    Ok(life)
}

/// A cost with its kicker paid `n` times (CR 702.33); a kicker with {X} in it is paid once for the X announced, which
/// is `n`, so it adds that much generic mana (and its other symbols once, when n is not 0).
fn kicker_total(base: ManaCost, k: Kicker, n: u16) -> ManaCost {
    let mut total = base;
    if k.cost.x > 0 {
        if n > 0 {
            total = add_cost(total, ManaCost { x: 0, ..k.cost });
            total.generic = total.generic.saturating_add((n as u32 * k.cost.x as u32).min(255) as u8);
        }
        return total;
    }
    for _ in 0..n {
        total = add_cost(total, k.cost);
    }
    total
}

/// Announces whether an optional cost that is not mana is paid (CR 601.2b), as the C# engine asks it: a choice between
/// casting without it and paying it, the second on offer only when it could be paid. Whether it was chosen.
fn announce_optional(cx: &mut Cx, player: PlayerId, spell: ObjectId, face: CardId, def: &'static CardDef, optional: &OptionalCost) -> R<bool> {
    // A spell that must choose more modes when it is paid ("choose both instead") can be cast that way only when that
    // many modes could be chosen (CR 700.2a, 601.2b): paying is not on offer otherwise.
    let asker = Asker { source: spell, controller: player };
    let modes_allow = def.modal.is_none_or(|modal| {
        modal.when_paid.is_none_or(|(least, _)| eligible_modes(cx, &asker, modal).len() >= least.max(1) as usize)
    });
    let payable = modes_allow && can_pay_other(cx, spell, player, &optional.cost);
    let ask = RequestKind::ChooseModes {
        source: spell,
        modes: (0..def.labels.len().min(2) as u8).map(|i| Label::Card(face, i)).collect(),
        eligible: if payable { vec![0, 1] } else { vec![0] },
        count: 1,
        at_most: 0,
        affordable: Vec::new(),
        additional_cost: true,
        repeat: false,
    };
    match cx.ask_core(player, ask, true)? {
        Answer::Modes(chosen) => Ok(chosen.first() == Some(&1)),
        _ => Err(Stop::Cancel),
    }
}

/// Chooses which additional cost is paid (CR 601.2b): only the ones that can be paid are offered, and a lone one is
/// taken without asking, as the C# engine does.
fn choose_additional(cx: &mut Cx, player: PlayerId, spell: ObjectId, face: CardId, def: &'static CardDef) -> R<Cost> {
    let payable: Vec<u8> = (0..def.additional_choice.len() as u8)
        .filter(|&i| can_pay_other(cx, spell, player, &def.additional_choice[i as usize].cost))
        .collect();
    let Some(&first) = payable.first() else { return Err(Stop::Cancel) };
    let mut picked = first;
    if payable.len() > 1 {
        let ask = RequestKind::ChooseModes {
            source: spell,
            modes: (0..def.additional_choice.len() as u8).map(|i| Label::Extra(face, i)).collect(),
            eligible: payable,
            count: 1,
            at_most: 0,
            affordable: Vec::new(),
            additional_cost: true,
            repeat: false,
        };
        match cx.ask_core(player, ask, true)? {
            Answer::Modes(chosen) => picked = chosen.first().copied().unwrap_or(first),
            _ => return Err(Stop::Cancel),
        }
    }
    Ok(def.additional_choice[picked as usize].cost)
}

/// Announcing X or the number of times a spell is kicked (CR 601.2b), as the C# engine asks it: a pay-costs
/// question that raises and lowers the value until confirmed.
fn announce(cx: &mut Cx, player: PlayerId, spell: ObjectId, base: ManaCost, kicker: Option<Kicker>, x_symbols: u8, cap: u16) -> R<u16> {
    announce_x(cx, player, spell, base, kicker, x_symbols, cap, false)
}

/// `announce`, saying whether the X is a "−X" loyalty cost's (`RequestKind::PayCosts::loyalty`).
#[allow(clippy::too_many_arguments)]
fn announce_x(cx: &mut Cx, player: PlayerId, spell: ObjectId, base: ManaCost, kicker: Option<Kicker>, x_symbols: u8, cap: u16, loyalty: bool) -> R<u16> {
    let most: u16 = match kicker {
        Some(Kicker { multi: false, cost, .. }) if cost.x == 0 => 1,
        _ => cap,
    };
    let mut n: u16 = 0;
    loop {
        let mut total = base;
        match kicker {
            Some(k) => total = kicker_total(total, k, n),
            None if x_symbols > 0 => total.generic = total.generic.saturating_add((n as u32 * x_symbols as u32).min(255) as u8),
            None => {}
        }
        let mut options = Vec::new();
        if n < most {
            options.push(ActionOption::simple(ActionKind::IncreaseX));
        }
        if n > 0 {
            options.push(ActionOption::simple(ActionKind::DecreaseX));
        }
        options.push(ActionOption::simple(ActionKind::ConfirmCasting));
        options.push(ActionOption::simple(ActionKind::CancelCasting));
        let ask = RequestKind::PayCosts { source: spell, total, remaining: total, options, announcing: true, x: n, loyalty, attackers: Vec::new(), life: 0 };
        match cx.ask_core(player, ask, true)? {
            Answer::Act(a) => match a.kind {
                ActionKind::IncreaseX => n += 1,
                ActionKind::DecreaseX => n -= 1,
                ActionKind::ConfirmCasting => return Ok(n),
                _ => return Err(Stop::Cancel),
            },
            _ => return Err(Stop::Cancel),
        }
    }
}

/// Activating a non-mana ability (CR 602.2): on the stack, then its costs.
fn activate(cx: &mut Cx, player: PlayerId, source: ObjectId, ability: AbilityRef, index: u8, targets: &[Target]) -> R<()> {
    let def = activated(cx, ability).expect("activated");
    let mut item = stack_item(StackKind::Activated, source, ability, player, targets);
    let mut mana = def.cost.mana;
    // CR 118.9, 602.2b: a static ability may let the player pay another cost rather than the ability's mana cost.
    if let Some((granter, alternative)) = static_rules::activation_alternative(cx, player, source, def)
        && alternative != mana
        && matches!(cx.ask_core(player, RequestKind::May { source: granter, purpose: Purpose::Ask(&ALTERNATIVE_ACTIVATION_COST) }, true)?, Answer::Yes)
    {
        mana = alternative;
    }

    // CR 602.2b, 601.2b: a modal ability's modes first, then X, then the Phyrexian symbols paid with life.
    let mut mask = 0u8;
    if let Some(modal) = def.modal {
        mask = choose_modes(cx, player, source, ability, modal, (modal.min, modal.max), mana)?;
        item.modes = mask;
    }
    if mana.x > 0 {
        // "X target lands" cannot be announced past the legal targets there are (CR 601.2c, 602.2b).
        let asker = Asker { source, controller: player };
        let cap = def
            .targets
            .iter()
            .filter(|spec| spec.min == TargetSpec::COUNTED)
            .map(|spec| target_candidates(cx, &asker, spec).len().min(1000) as u16)
            .min()
            .unwrap_or(1000);
        let n = announce(cx, player, source, mana, None, mana.x, cap)?;
        mana.generic += n as u8 * mana.x;
        item.x = n;
    }
    // CR 606.4, 107.3, 601.2b by way of 602.2b: "−X" announces X, up to the loyalty counters there are to remove (CR 118.3).
    if def.cost.loyalty == Some(LOYALTY_MINUS_X) {
        let cap = cx.s().obj(source).counters(CounterKind::Loyalty).max(0) as u16;
        item.x = announce_x(cx, player, source, mana, None, 0, cap, true)?;
    }
    let life = announce_life(cx, player, source, &mut mana)?;
    // CR 602.2a, 701.27f: put on the stack now, before its costs are paid.
    item.since = cx.g.log.len() as u32;
    cx.st().stack_push(item);

    let mut targets: Vec<Target> = targets.to_vec();
    if let Some(modal) = def.modal {
        let of;
        (targets, of) = choose_slots(cx, player, source, &mode_specs(modal, mask), item.x, 0)?;
        set_item_targets(cx, &targets, &of);
    } else if !targets_with_activation(def) && !def.targets.is_empty() {
        // CR 601.2c by way of 602.2b: slots that are not announced with the action are chosen now, after X (a slot
        // counted by X, as Candelabra of Tawnos's "untap X target lands", reads the X just announced).
        let of;
        (targets, of) = choose_slots(cx, player, source, def.targets, item.x, 0)?;
        set_item_targets(cx, &targets, &of);
    }
    // CR 601.2f by way of 602.2b: the total cost, with what increases or reduces it, once the targets are announced: a
    // change may read them ("equip abilities you activate that target this creature cost {2} less"). It is locked in
    // here, so nothing that happens to a target later changes what is paid. Life a change adds (CR 118.8) is payable only
    // with that much life, the rest of the cost's included (CR 119.4, 118.3), or the activation is taken back.
    let added_life;
    (mana, added_life) = apply_cost_changes_total(cx, player, Costed::Ability(source, ability), mana, &targets);
    let life = life as u32 + added_life;
    if added_life > 0 && cx.s().player(player).life < life as i32 + def.cost.life as i32 {
        return Err(Stop::Cancel);
    }

    // Behold is paid when a spell is cast only (CR 701.4a): an activated ability that had it would get the cost for free.
    assert!(def.cost.behold.is_none(), "an activated ability's behold is not built");
    // A cost of tapped permanents is chosen once the mana is paid (CR 601.2g-h), which only a spell's does yet: no
    // activated ability has one.
    // `every_activated_ability_has_a_cost_activation_can_pay` (cantrip-engine) refuses such a card before any game.
    assert!(!permanents_after_mana(&def.cost), "an activated ability's cost of tapped permanents is not built");
    let chosen = choose_permanents(cx, player, source, &def.cost)?;
    // What an attacking creature the cost returns or sacrifices was attacking, for the effect to read (ninjutsu,
    // CR 702.49a: the ninja attacks the same player, planeswalker or battle).
    if let Some(first) = chosen.first()
        && let Some(attack) = cx.s().attackers.iter().find(|a| a.id == *first).copied()
    {
        let at = cx.s().stack.len() - 1;
        let mut with = cx.s().stack[at];
        with.subject = attack.target;
        cx.st().stack_set(at, with);
    }
    let discards = choose_discards(cx, player, source, &def.cost)?;
    let exiles = choose_exiles(cx, player, source, &def.cost)?;
    let tappers = choose_tappers(cx, player, source, &def.cost)?;
    // The permanents the cost chose stay with the ability, for its effect to name ("the tapped creature's power",
    // `Query::cost_objects`, CR 602.2b, 601.2h, 608.2h).
    if !chosen.is_empty() || !tappers.is_empty() {
        let objects: Vec<ObjectId> = chosen.iter().chain(&tappers).copied().collect();
        let members = cx.st().members_push(&objects);
        let at = cx.s().stack.len() - 1;
        let mut with = cx.s().stack[at];
        with.members = members;
        cx.st().stack_set(at, with);
    }
    let counter = choose_counter_kind(cx, player, source, &def.cost)?;
    // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
    if mana.mana_value() > 0 && static_rules::spend_any_color_for(cx, player, source) {
        mana = ManaCost { generic: mana.mana_value().min(255) as u8, ..ManaCost::ZERO };
    }
    pay_mana_with(
        cx,
        player,
        source,
        mana,
        &Helpers { waterbend: if def.waterbend { u8::MAX } else { 0 }, creatures: def.tap_creatures_for_mana, purpose: Paying::Activating, ..Helpers::NONE },
    )?;
    // Craft's materials are exiled as the cost is paid (CR 601.2h by way of 602.2b), after the mana abilities (CR 601.2g):
    // they are chosen now, from what is still there, so a Black Lotus sacrificed for the mana is no material. Too few left
    // is a cost that can't be paid, and the activation is taken back.
    let materials = choose_materials(cx, player, source, &def.cost)?;
    // The creature a blight puts its counters on (CR 701.68a) and the counters removed from among permanents (CR 118.3) are
    // chosen as they are paid, after the mana abilities (CR 601.2g-h by way of 602.2b), as a spell's additional cost's are:
    // from what is there once the mana is paid. With no creature left to blight the cost can't be paid (CR 701.68b).
    if def.cost.blight > 0 && blight_candidates(cx, player).is_empty() {
        return Err(Stop::Cancel);
    }
    let blighted = choose_blight(cx, player, source, &def.cost)?;
    let removed = choose_counters_among(cx, player, source, &def.cost)?;
    if life > 0 {
        ops::change_life(cx, player, -(life as i32));
    }
    // The costs are paid together (CR 601.2h by way of 602.2b), so {T} is paid on the permanent before any other part of
    // the cost moves it: it was tapped as it left (CR 118.3), its last-known information says so (CR 608.2h), and
    // "whenever ... becomes tapped" sees the tap of a permanent still on the battlefield ("{T}, Exile this artifact",
    // "{T}, Sacrifice a creature" choosing itself).
    if def.cost.tap {
        ops::tap(cx, source);
    }
    // Counters put on and taken off permanents next, so a permanent the cost also sacrifices leaves with them.
    pay_blight(cx, &def.cost, blighted);
    pay_counters_among(cx, &removed);
    pay_permanents(cx, &def.cost, &chosen);
    pay_discards(cx, &discards);
    pay_exiles(cx, &exiles);
    pay_life_and_counters(cx, player, source, &def.cost);
    if let Some(kind) = counter {
        cx.st().update_obj(source, |o| o.counters[kind as usize] = (o.counters[kind as usize] - 1).max(0));
        ops::end_lapsed_durations(cx, source);
    }
    pay_exert_and_tappers(cx, source, &def.cost, &tappers);
    note_activation(cx, source, index as usize, def);
    // Craft exiles its source with its materials, last of all (`pay_materials`).
    if def.cost.exile_self && def.cost.materials.is_none() {
        ops::move_to_for(cx, source, Zone::exile(cx.s().obj(source).owner), ZoneReason::Exile);
    }
    if def.cost.discard_self {
        ops::discard(cx, source);
        if def.from == ZoneKind::Hand {
            cx.emit(Event::Cycled { card: source, player });
        }
    }
    if let Some(loyalty) = def.cost.loyalty {
        let turn = cx.s().turn.number;
        let change = if loyalty == LOYALTY_MINUS_X { -(item.x as i16) } else { loyalty as i16 };
        cx.st().update_obj(source, |o| {
            o.loyalty_turn = turn;
            let n = &mut o.counters[CounterKind::Loyalty as usize];
            *n = (*n + change).max(0);
        });
        ops::end_lapsed_durations(cx, source);
    }
    if def.cost.sacrifice_self {
        ops::sacrifice(cx, &[source]);
    }
    if def.cost.return_self {
        ops::move_to_for(cx, source, Zone::hand(cx.s().obj(source).owner), ZoneReason::Other);
    }
    if def.cost.materials.is_some() {
        assert!(def.cost.exile_self, "craft exiles its source with its materials (CR 702.167a)");
        pay_materials(cx, source, &materials);
    }
    cx.emit(Event::AbilityActivated { source, ability, player });
    ops::became_targets(cx, targets.iter().copied(), source, player);
    ops::commit_crime(cx, player, source, &targets);
    Ok(())
}

/// The mana a special action costs (CR 116.2): the ability's own, except for the two the rules word as "pay the mana cost of
/// the object": turning a face-down creature card face up costs its mana cost, with X as 0 (CR 701.40b, 107.3b), and
/// unlocking a Room's door the mana cost of that door (CR 709.5e).
/// What a special action costs `player` to take (CR 116.2): its mana cost - a face-down permanent's card's mana cost to turn
/// it face up, a Room door's to unlock it, the printed cost otherwise - with the static abilities that change the cost of
/// that special action applied (`Costed::SpecialAction`: "plotting cards from your hand costs {2} less", CR 116.2k, 118.7),
/// increases first, then reductions, none below {0}.
fn special_action_cost(cx: &Cx, player: PlayerId, source: ObjectId, ability: AbilityRef, def: &ActivatedDef) -> ManaCost {
    let printed = special_action_printed_cost(cx, source, ability, def);
    change_with(cx, &cost_change_sources(cx), player, Costed::SpecialAction(source, ability), printed, None, &[])
}

fn special_action_printed_cost(cx: &Cx, source: ObjectId, ability: AbilityRef, def: &ActivatedDef) -> ManaCost {
    let card = cx.s().obj(source).card;
    if ability.card == cx.cards().face_down {
        return ManaCost { x: 0, ..cx.card(card).cost };
    }
    if ability.card == cx.cards().intrinsic {
        // The Room the permanent is, which a copy of one is too (CR 709.5, 707.2).
        let room = cx.card(cx.chars(source).card);
        let door = match ability.index {
            cantrip_api::intrinsic::UNLOCK_LEFT => room.split.map(|h| h.0),
            cantrip_api::intrinsic::UNLOCK_RIGHT => room.split.map(|h| h.1),
            _ => None,
        };
        if let Some(door) = door {
            return door.cost;
        }
    }
    def.cost.mana
}

/// What the special action `index` of `source` (numbered as its options are) costs `player` now, where a cost change makes
/// it differ from its printed cost (CR 116.2, 118.7): for the menu row, which says so ("costs {B}"). None otherwise, or for
/// anything that is not a special action.
pub fn special_action_costs(game: &crate::Game, player: PlayerId, source: ObjectId, index: u8) -> Option<ManaCost> {
    let ability = game.chars(source).abilities().nth(index as usize)?;
    let AbilityDef::Activated(def) = game.cards.ability(ability) else { return None };
    if !def.special_action {
        return None;
    }
    let changers = game.state.battlefield().iter().any(|&id| {
        game.cards.cost_changes[game.cards.current(game.state.obj(id)).0 as usize] && !game.state.lost_all_abilities(id)
    });
    if !changers {
        return None;
    }
    let mut copy = crate::Game { state: game.state.clone(), log: game.log.clone(), tree: crate::tree::Tree::new(), at: crate::tree::NodeId::ROOT, cards: game.cards, verify: false };
    let cx = Cx { g: &mut copy, frame: usize::MAX, cursor: 0 };
    let printed = special_action_printed_cost(&cx, source, ability, def);
    let cost = special_action_cost(&cx, player, source, ability, def);
    (cost != printed).then_some(cost)
}

/// A special action (CR 116.2) from an ability: its mana cost is paid, and its effect done at once with no stack. The
/// effect asks no questions and has no targets.
fn special_action(cx: &mut Cx, player: PlayerId, source: ObjectId, ability: AbilityRef) -> R<()> {
    let def = activated(cx, ability).expect("activated");
    let mana = special_action_cost(cx, player, source, ability, def);
    if mana != ManaCost::ZERO {
        pay_mana(cx, player, source, mana)?;
    }
    let item = stack_item(StackKind::Activated, source, ability, player, &[]);
    // A special action's effect is done at once; a question it asked would have nowhere to be answered.
    let _ = (def.effect)(cx, &item);
    Ok(())
}

/// Activating a mana ability (CR 605.3): its costs, then its effect, at once.
pub fn activate_mana(
    cx: &mut Cx,
    player: PlayerId,
    source: ObjectId,
    index: u8,
    color: Option<Color>,
    colorless: bool,
    sacrifice: &[Target],
) -> R<()> {
    if index >= PLAYER_MANA_ABILITY {
        return activate_player_mana(cx, player, source, index - PLAYER_MANA_ABILITY);
    }
    let ability = cx.chars(source).abilities().nth(index as usize).expect("ability");
    let def = activated(cx, ability).expect("activated");
    // What restricts its mana, read from the source as it is activated, before a cost may take it away (CR 106.6).
    let restricted = def.restricted.map(|r| restriction_from(cx, source, r));
    // "Sacrifice a creature: Add ..." (Phyrexian Altar): the permanents come with the activation, as its option's
    // candidates (a mana ability never targets, CR 605.1a, so the slots are free for them).
    let chosen: Vec<ObjectId> = sacrifice.iter().filter_map(|t| t.object()).collect();
    // "Any type the sacrificed permanent could produce" (CR 106.7): read before it is gone, and the type named must be one.
    let mut adds_nothing = false;
    if def.colors_from_sacrifice {
        let (colors, can_colorless) = chosen.first().map_or((ColorSet::NONE, false), |&land| producible_types(cx, land));
        adds_nothing = colors.is_empty() && !can_colorless;
        let fits = match (color, colorless) {
            (Some(c), false) => colors.has(c),
            (None, true) => can_colorless,
            (None, false) => adds_nothing,
            (Some(_), true) => false,
        };
        if !fits {
            return Err(Stop::Cancel);
        }
    }
    // An {X} in the cost is announced first (CR 107.3a, 601.2b, 605.3a) and kept on the source for the effect to read.
    let mut mana = def.cost.mana;
    if mana.x > 0 {
        let n = announce(cx, player, source, mana, None, mana.x, 1000)?;
        mana.generic = mana.generic.saturating_add((n as u32 * mana.x as u32).min(255) as u8);
        mana.x = 0;
        cx.st().update_obj(source, |o| o.x = n);
    }
    note_activation(cx, source, index as usize, def);
    // A mana ability is activated (CR 605.3) as any ability is, and the history says so. It is not emitted: the cards here
    // that trigger on activations name loyalty abilities or abilities that aren't mana abilities; one that triggers on a
    // mana ability's activation too ("an ability of a creature": Elrond, Moon-Reader) needs it emitted, its trigger
    // waiting for the next time a player would receive priority (CR 603.3).
    cx.record(Event::AbilityActivated { source, ability, player });
    // {T} is paid on the permanent before any other part of the cost moves it (CR 605.3a, 601.2h, 118.3): see `activate`.
    if def.cost.tap {
        ops::tap(cx, source);
    }
    if def.cost.exile_self {
        ops::move_to_for(cx, source, Zone::exile(cx.s().obj(source).owner), ZoneReason::Exile);
    }
    // A mana ability's own mana cost (Chromatic Star's {1}) is paid like any other (CR 605.3a, 601.2g).
    if mana != ManaCost::ZERO {
        // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
        let mana = if static_rules::spend_any_color_for(cx, player, source) {
            ManaCost { generic: mana.mana_value().min(255) as u8, ..ManaCost::ZERO }
        } else {
            mana
        };
        pay_mana_with(cx, player, source, mana, &Helpers { purpose: Paying::Activating, ..Helpers::NONE })?;
    }
    pay_exert_and_tappers(cx, source, &def.cost, &[]);
    if def.cost.discard_hand {
        let hand = choose_discards(cx, player, source, &def.cost)?;
        pay_discards(cx, &hand);
    }
    pay_permanents(cx, &def.cost, &chosen);
    pay_life_and_counters(cx, player, source, &def.cost);
    if def.cost.sacrifice_self {
        ops::sacrifice(cx, &[source]);
    }
    let asker = Asker { source, controller: player };
    // What the ability itself produces (CR 106.12a), told from the pool before and after it: whatever replaced or added to
    // its mana, and of whatever restriction, but not what the triggered mana abilities it triggers add.
    let before = if def.cost.tap { mana_held(cx, player) } else { ManaPool::default() };
    if !adds_nothing {
        match def.mana {
            Some(effect) => effect(cx, &asker, color)?,
            None => {
                let def = ActivatedDef { restricted, ..*def };
                add_listed(cx, player, &def, if colorless { Some(Mana::C) } else { color.map(Color::mana) })
            }
        }
    }
    if def.cost.tap {
        let after = mana_held(cx, player);
        let produced = ManaPool(std::array::from_fn(|m| after.0[m].saturating_sub(before.0[m])));
        tapped_for_mana(cx, player, source, produced)?;
    }
    Ok(())
}

/// The mana in `player`'s pool, of every restriction and none.
fn mana_held(cx: &Cx, player: PlayerId) -> ManaPool {
    let mut held = cx.s().player(player).pool;
    for (_, pool) in cx.s().restricted_of(player) {
        for m in 0..6 {
            held.0[m] = held.0[m].saturating_add(pool.0[m]);
        }
    }
    held
}

/// Activating a mana ability the player has of their own (CR 605.1a, 611.2a, 605.3): the nth of those that effects gave them.
/// Its cost is paid, then its mana added at once. It has no source object (CR 605.1a): `source` is only the card that gave it.
fn activate_player_mana(cx: &mut Cx, player: PlayerId, source: ObjectId, n: u8) -> R<()> {
    let Some(EffectKind::PlayerManaAbility(ability)) = cx
        .s()
        .effects
        .iter()
        .filter(|e| matches!(e.kind, EffectKind::PlayerManaAbility(_)) && e.controller == player)
        .nth(n as usize)
        .map(|e| e.kind)
    else {
        return Err(Stop::Cancel);
    };
    let def = activated(cx, ability).expect("activated");
    pay_life_and_counters(cx, player, source, &def.cost);
    match def.mana {
        Some(effect) => effect(cx, &Asker { source, controller: player }, None)?,
        None => add_listed(cx, player, def, None),
    }
    Ok(())
}

/// The mana types a permanent could produce (CR 106.7): the colors its mana abilities add or let the player name, and
/// whether they add colorless. Restrictions and costs are not considered.
fn producible_types(cx: &Cx, land: ObjectId) -> (ColorSet, bool) {
    let controller = cx.s().obj(land).controller;
    let (mut colors, mut colorless) = (ColorSet::NONE, false);
    for ability in cx.chars(land).abilities() {
        let Some(def) = activated(cx, ability) else { continue };
        if !def.is_mana_ability() || def.from != ZoneKind::Battlefield {
            continue;
        }
        colorless |= def.adds.0[Mana::C as usize] > 0;
        for c in Color::ALL {
            if def.adds.0[c.mana() as usize] > 0 {
                colors = colors.with(c);
            }
        }
        if def.any_color {
            let named = match def.colors_from {
                Some(from) => from(cx, &Asker { source: land, controller }),
                None if def.colors.is_empty() => ColorSet(0b11111),
                None => def.colors,
            };
            colors = ColorSet(colors.0 | named.0);
        }
    }
    (colors, colorless)
}

/// A permanent was tapped for mana (CR 106.12, 605.1b) and produced `produced`: the fact is recorded, and, if it produced any
/// mana, the triggered mana abilities it triggers (CR 106.12a) resolve at once, as part of the mana ability that triggered
/// them (CR 605.4a): "whenever you tap a Forest for mana, add an additional {G}". They are looked for among the permanents
/// whose cards have one. One that asks (a type of the mana that land produced) suspends the activation, to be run again
/// with the answer.
fn tapped_for_mana(cx: &mut Cx, player: PlayerId, source: ObjectId, produced: ManaPool) -> R<()> {
    let event = Event::TappedForMana { permanent: source, player, produced };
    cx.record(event);
    if produced.is_empty() {
        return Ok(());
    }
    let holders: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            cx.cards().mana_triggers[obj.copy.map_or(obj.card, |c| c.card).0 as usize]
        })
        .collect();
    let mut statics = crate::settle::TriggerStatics::new(None);
    for id in holders {
        let controller = cx.s().obj(id).controller;
        for ability in cx.chars(id).abilities().collect::<Vec<_>>() {
            let AbilityDef::Triggered(t) = cx.cards().ability(ability) else { continue };
            if !t.mana_ability {
                continue;
            }
            if let Some(fired) = t.fires(&*cx, &Asker { source: id, controller }, &event) {
                // Static abilities that stop it triggering or make it trigger again apply to it as to any triggered ability
                // (CR 603.2, 603.2d): each triggering resolves at once.
                let triggering = Triggering { source: id, controller, def: t, permanent: true };
                if statics.stop(cx, &event, &triggering) {
                    continue;
                }
                let mut again = crate::settle::Again::new();
                statics.additional(cx, &event, &triggering, &mut again);
                let mut item = stack_item(StackKind::Triggered, id, ability, controller, &[]);
                item.subject = fired.subject;
                item.data = fired.data;
                for _ in 0..=again.len() {
                    (t.effect)(cx, &item)?;
                }
            }
        }
    }
    Ok(())
}

// --- Paying ----------------------------------------------------------------------------------------------------------

/// What is still owed after spending one unit of `mana`: a colored symbol of its color first, else generic.
fn spend(owed: &mut ManaCost, mana: Mana) {
    if owed.symbols[mana as usize] > 0 {
        owed.symbols[mana as usize] -= 1;
    } else if let Some(slot) = owed.hybrid.iter().position(|h| matches!(h, Some((a, b)) if *a == mana || *b == mana)) {
        owed.hybrid[slot] = None;
    } else if owed.generic > 0 {
        owed.generic -= 1;
    }
}

/// Whether a unit of `mana` pays anything still owed.
fn pays(owed: &ManaCost, mana: Mana) -> bool {
    owed.generic > 0
        || owed.symbols[mana as usize] > 0
        || owed.hybrid.iter().flatten().any(|(a, b)| *a == mana || *b == mana)
}

/// How the pool would pay `owed`: none (it cannot), exactly one way, or more than one.
enum Ways {
    None,
    One([u8; 6]),
    Many,
}

/// What of `owed` the pool does not cover, for the payer to make mana for: each colored symbol the pool has the color for
/// is covered, a hybrid or Phyrexian symbol by either of its colors, and the generic mana by what is left over. Advice for
/// the prompt, not a payment: the pool is spent once it covers the whole cost.
fn uncovered(pool: &ManaPool, owed: &ManaCost) -> ManaCost {
    let mut left = pool.0;
    let mut out = *owed;
    for mana in Mana::ALL {
        let i = mana as usize;
        let take = left[i].min(out.symbols[i]);
        left[i] -= take;
        out.symbols[i] -= take;
    }
    for slot in out.hybrid.iter_mut() {
        if let Some((a, b)) = *slot {
            if let Some(m) = [a, b].into_iter().find(|&m| left[m as usize] > 0) {
                left[m as usize] -= 1;
                *slot = None;
            }
        }
    }
    for (i, n) in out.phyrexian.iter_mut().enumerate() {
        let take = left[i].min(*n);
        left[i] -= take;
        *n -= take;
    }
    let spare: u32 = left.iter().map(|&n| n as u32).sum();
    out.generic = out.generic.saturating_sub(spare.min(255) as u8);
    out
}

fn ways(pool: &ManaPool, owed: &ManaCost) -> Ways {
    let mut left = pool.0;
    let mut spent = [0u8; 6];
    for mana in Mana::ALL {
        let need = owed.symbols[mana as usize];
        if left[mana as usize] < need {
            return Ways::None;
        }
        left[mana as usize] -= need;
        spent[mana as usize] += need;
    }
    // A hybrid symbol is paid one way when only one of its colors is in the pool, and is a choice when both are.
    for (a, b) in owed.hybrid.iter().flatten() {
        match (left[*a as usize] > 0, left[*b as usize] > 0) {
            (true, false) => {
                left[*a as usize] -= 1;
                spent[*a as usize] += 1;
            }
            (false, true) => {
                left[*b as usize] -= 1;
                spent[*b as usize] += 1;
            }
            (true, true) => return Ways::Many,
            (false, false) => return Ways::None,
        }
    }
    let generic = owed.generic;
    let spare: u32 = left.iter().map(|&n| n as u32).sum();
    if spare < generic as u32 {
        return Ways::None;
    }
    let kinds = left.iter().filter(|&&n| n > 0).count();
    if generic == 0 || spare == generic as u32 || kinds == 1 {
        let mut g = generic;
        for mana in Mana::ALL {
            let take = left[mana as usize].min(g);
            spent[mana as usize] += take;
            g -= take;
        }
        return Ways::One(spent);
    }
    Ways::Many
}

/// The pool units that would pay something owed, as `AssignMana` options, in the C# engine's order: colorless first,
/// then colors the cost does not name, then the colors it names.
fn assign_options(pool: &ManaPool, restricted: &[(ManaRestriction, ManaPool)], owed: &ManaCost) -> Vec<ActionOption> {
    let named = |mana: Mana| {
        owed.symbols[mana as usize] > 0 || owed.hybrid.iter().flatten().any(|(a, b)| *a == mana || *b == mana)
    };
    let order = [Mana::C, Mana::W, Mana::U, Mana::B, Mana::R, Mana::G];
    let mut out = Vec::new();
    // Restricted mana first, then the rest.
    for (restriction, pool) in restricted {
        for mana in order {
            if pool.get(mana) > 0 && pays(owed, mana) {
                let mut option = ActionOption::simple(ActionKind::AssignMana);
                option.mana = Some(mana);
                option.restriction = Some(*restriction);
                out.push(option);
            }
        }
    }
    for pass in 0..3 {
        for mana in order {
            let kind = if mana == Mana::C { 0 } else if named(mana) { 2 } else { 1 };
            if kind != pass || pool.get(mana) == 0 || !pays(owed, mana) {
                continue;
            }
            let mut option = ActionOption::simple(ActionKind::AssignMana);
            option.mana = Some(mana);
            out.push(option);
        }
    }
    out
}

/// What paying for `source` is, as far as restricted mana is concerned (CR 106.6): casting it, when it is a spell on the
/// stack being cast; activating one of its abilities; or anything else. Only casting a spell is limited to spells (CR
/// 601.2g): paying "unless its controller pays" (CR 118.12) or a copy's cost for the spell on the stack is not casting it;
/// and only an ability's activation is activating one (CR 602.2b, 605.3a), never a special action (CR 116.2). The source
/// is judged as it is now, as the cost is paid (an ability of a land card in the hand is an ability of a land source).
fn payment_for(cx: &Cx, player: PlayerId, source: ObjectId, purpose: Paying) -> ManaPayment {
    match purpose {
        Paying::Casting if cx.s().obj(source).zone.is(ZoneKind::Stack) => {
            // The subtypes, only where some of the player's mana is limited by them.
            spell_payment(cx, source, cx.s().restricted_of(player).any(|(r, _)| r.names_subtypes()))
        }
        Paying::Activating => {
            let chars = cx.chars(source);
            ManaPayment::Ability { source: chars.types, subtypes: chars.subtypes }
        }
        _ => ManaPayment::Other,
    }
}

/// Casting `spell`, a spell on the stack, as it is there (CR 601.2g): its card types, supertypes, mana value with X as
/// announced (CR 202.3e: 0 for a copy with no mana cost or of a back face, CR 707.9b, 202.3b, and for a face-down spell,
/// CR 708.4; the cast half's alone, CR 709.3b), whether its mana cost has {X} and the zone it was cast from (CR 601.2a); its
/// subtypes only `with_subtypes`, where some mana at hand is limited by them.
fn spell_payment(cx: &Cx, spell: ObjectId, with_subtypes: bool) -> ManaPayment {
    let chars = cx.chars(spell);
    let subtypes = if with_subtypes { chars.subtypes } else { SubtypeSet::NONE };
    ManaPayment::Spell {
        types: chars.types,
        subtypes,
        legendary: chars.supertypes.legendary,
        mana_value: cx.mana_value(spell),
        x: cx.mana_cost(spell).x > 0,
        from: cx.s().obj(spell).cast_from.unwrap_or(ZoneKind::Stack),
    }
}

/// The restriction a mana ability of `source` puts on the mana it adds (CR 106.6): a subtype its text names is read
/// through the source's words (CR 612: Artificial Evolution makes Maelstrom of the Spirit Dragon's mana for Elf spells),
/// and "the chosen type" is the creature type chosen for it as it entered (CR 607.2a, 614.12a), or none if none was.
pub fn restriction_from(cx: &Cx, source: ObjectId, restriction: ManaRestriction) -> ManaRestriction {
    let ManaRestriction::Subtypes(mut s) = restriction else { return restriction };
    if s.chosen {
        s.subtypes = [cx.s().obj(source).chosen_subtype, None];
        s.chosen = false;
    } else {
        let words = cantrip_api::words(cx.s(), source);
        s.subtypes = s.subtypes.map(|t| t.map(|t| words.subtype(t)));
    }
    ManaRestriction::Subtypes(s)
}

/// The restricted mana `player` has that may pay for `source`, with its restriction.
fn restricted_for(cx: &Cx, player: PlayerId, source: ObjectId, purpose: Paying) -> Vec<(ManaRestriction, ManaPool)> {
    if !cx.s().has_restricted_mana(player) {
        return Vec::new();
    }
    let payment = payment_for(cx, player, source, purpose);
    cx.s().restricted_of(player).filter(|(restriction, _)| restriction.may_pay(&payment)).collect()
}

/// The pool a cost for `source` may be paid from: unrestricted mana and the restricted mana that may pay for it.
fn spendable(cx: &Cx, player: PlayerId, source: ObjectId, purpose: Paying) -> ManaPool {
    let mut pool = cx.s().player(player).pool;
    for (_, restricted) in restricted_for(cx, player, source, purpose) {
        for m in Mana::ALL {
            pool.0[m as usize] += restricted.0[m as usize];
        }
    }
    pool
}

/// Takes `spent` out of the pool, restricted mana first (it is what the C# engine prefers spending). What the mana taken
/// does to what it pays for (`ManaRestriction::effects`).
fn take_mana(cx: &mut Cx, player: PlayerId, source: ObjectId, purpose: Paying, spent: &[u8; 6]) -> u8 {
    let restricted = restricted_for(cx, player, source, purpose);
    let mut used = 0u8;
    for mana in Mana::ALL {
        let mut need = spent[mana as usize];
        for (restriction, _) in &restricted {
            let take = cx.st().spend_restricted_mana(player, *restriction, mana, need);
            if take > 0 {
                used |= restriction.effects();
            }
            need -= take;
        }
        if need > 0 {
            cx.st().update_player(player, |p| p.spend_mana(mana, need));
        }
    }
    used
}

/// The mana abilities `player` may activate while paying for `source` (CR 601.2g, 605.3a), each whose mana is restricted
/// so that none of it may pay this cost marked as paying nothing (CR 106.6).
fn mana_options_while_paying(cx: &Cx, player: PlayerId, source: ObjectId, purpose: Paying) -> Vec<ActionOption> {
    let mut options = Vec::new();
    mana_ability_options(cx, player, &mut options);
    if options.iter().any(|o| o.restriction.is_some()) {
        let payment = payment_for(cx, player, source, purpose);
        for option in &mut options {
            option.pays_nothing = option.restriction.is_some_and(|r| !r.may_pay(&payment));
        }
    }
    options
}

/// What besides mana may pay a spell's total cost (CR 601.2f-h): convoke (CR 702.51a), delve (CR 702.66a), improvise
/// (CR 702.126a), and whether mana may be spent on it at all ("You can't spend mana to cast this spell", CR 601.2h).
#[derive(Clone, Copy)]
pub(crate) struct Helpers<'a> {
    pub with: PayWith,
    /// Waterbend (CR 701.67a-b): how many of the generic mana an untapped artifact or creature tapped may pay {1} of.
    pub waterbend: u8,
    /// An untapped creature tapped pays {1} of the generic mana (Heirloom Epic).
    pub creatures: bool,
    pub no_mana: bool,
    /// What the mana pays for (CR 106.6): casting `source`, a spell (CR 601.2g), activating one of its abilities (CR
    /// 602.2b, 605.3a), or anything else.
    pub purpose: Paying,
    /// Cards another part of the cost was already chosen to exile (escape's), which delve may not also exile.
    pub reserved: &'a [ObjectId],
    /// The attacking creatures whose cost to attack this pays (CR 508.1h-j), for the question to name.
    pub attackers: &'a [ObjectId],
    /// The life the total cost asks besides its mana (CR 119.4, 601.2h), paid with it once the mana is: for the question to
    /// say.
    pub life: u32,
}

impl Helpers<'static> {
    pub const NONE: Helpers<'static> = Helpers { with: PayWith::NONE, waterbend: 0, creatures: false, no_mana: false, purpose: Paying::Other, reserved: &[], attackers: &[], life: 0 };
}

/// What a mana payment is for, as far as restricted mana is concerned (CR 106.6): see `payment_for`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Paying {
    Casting,
    Activating,
    Other,
}

impl Helpers<'_> {
    /// Whether anything besides mana may pay.
    pub fn any(&self) -> bool {
        self.with.any() || self.waterbend > 0 || self.creatures
    }
}

/// Pays a mana cost (CR 601.2g-h) with nothing but mana, for anything but casting a spell or activating an ability.
pub fn pay_mana(cx: &mut Cx, player: PlayerId, source: ObjectId, total: ManaCost) -> R<ManaPool> {
    pay_mana_with(cx, player, source, total, &Helpers::NONE)
}

/// The pips of a cost a creature tapped for convoke may pay (CR 702.51a): a colored symbol of a color it has, or either
/// color of a hybrid symbol it has one of. Colorless is not a color, so never {C} (CR 105.1).
fn convoke_pips(cost: &ManaCost) -> Vec<[Option<Mana>; 2]> {
    let mut pips = Vec::new();
    for mana in Mana::ALL {
        for _ in 0..cost.symbols[mana as usize] {
            pips.push([Some(mana), None]);
        }
    }
    for (a, b) in cost.hybrid.iter().flatten() {
        pips.push([Some(*a), Some(*b)]);
    }
    pips
}

/// Which creature (by index) pays each pip when as many pips as possible are paid by creatures of their colors.
fn convoke_match(pips: &[[Option<Mana>; 2]], creatures: &[ColorSet]) -> Vec<Option<usize>> {
    fn accepts(pip: &[Option<Mana>; 2], colors: ColorSet) -> bool {
        pip.iter().flatten().any(|m| m.color().is_some_and(|c| colors.has(c)))
    }
    fn assign(p: usize, pips: &[[Option<Mana>; 2]], creatures: &[ColorSet], paid_by: &mut [Option<usize>], seen: &mut [bool]) -> bool {
        for c in 0..creatures.len() {
            if seen[c] || !accepts(&pips[p], creatures[c]) {
                continue;
            }
            seen[c] = true;
            let holder = paid_by.iter().position(|&x| x == Some(c));
            if holder.is_none_or(|q| assign(q, pips, creatures, paid_by, seen)) {
                paid_by[p] = Some(c);
                return true;
            }
        }
        false
    }
    let mut paid_by = vec![None; pips.len()];
    for p in 0..pips.len() {
        let mut seen = vec![false; creatures.len()];
        assign(p, pips, creatures, &mut paid_by, &mut seen);
    }
    paid_by
}

/// What is left of a cost once creatures with these colors have been tapped for convoke (CR 702.51a): each pays a
/// symbol of a color it has where one can, and {1} otherwise, which leaves the most for mana to pay.
fn convoked(cost: &ManaCost, creatures: &[ColorSet]) -> ManaCost {
    if creatures.is_empty() {
        return *cost;
    }
    let pips = convoke_pips(cost);
    let paid_by = convoke_match(&pips, creatures);
    let matched = paid_by.iter().flatten().count();
    let mut left = *cost;
    left.generic = left.generic.saturating_sub((creatures.len() - matched) as u8);
    // The pips come in the cost's order, colored symbols first and hybrid symbols after.
    let mut at = 0;
    for mana in Mana::ALL {
        let mut kept = 0;
        for _ in 0..cost.symbols[mana as usize] {
            if paid_by[at].is_none() {
                kept += 1;
            }
            at += 1;
        }
        left.symbols[mana as usize] = kept;
    }
    for slot in 0..4 {
        if cost.hybrid[slot].is_some() {
            if paid_by[at].is_some() {
                left.hybrid[slot] = None;
            }
            at += 1;
        }
    }
    left
}

/// Whether creatures with these colors could all be tapped for convoke towards a cost: each paying a different symbol
/// of a color it has, and the rest no more than the generic mana.
fn can_convoke(cost: &ManaCost, creatures: &[ColorSet]) -> bool {
    let pips = convoke_pips(cost);
    let matched = convoke_match(&pips, creatures).iter().flatten().count();
    creatures.len() - matched <= cost.generic as usize
}

/// The permanents and cards that could pay something of `due` instead of mana right now (CR 601.2h), in the order the
/// C# engine offers them: delve's graveyard cards, then improvise's artifacts, then convoke's creatures; each once.
fn pay_candidates(
    cx: &Cx,
    player: PlayerId,
    source: ObjectId,
    helpers: &Helpers,
    cost: &ManaCost,
    due: &ManaCost,
    tapped: &[ColorSet],
    delved: &[ObjectId],
    bent: u8,
) -> Vec<ActionOption> {
    let mut out: Vec<ActionOption> = Vec::new();
    if helpers.with.delve && due.generic > 0 {
        for &id in cx.s().zone(Zone::graveyard(player)) {
            if id != source && !helpers.reserved.contains(&id) && !delved.contains(&id) {
                out.push(ActionOption::on(ActionKind::ExileToPay, id, PAY_DELVE));
            }
        }
    }
    if helpers.with.improvise || helpers.with.convoke || helpers.waterbend > 0 || helpers.creatures {
        for id in cx.permanents_of(player) {
            if cx.s().obj(id).tapped || out.iter().any(|o| o.source == Some(id)) {
                continue;
            }
            let chars = cx.chars(id);
            if helpers.waterbend > bent && due.generic > 0 && (chars.is(CardType::Artifact) || chars.is(CardType::Creature)) {
                out.push(ActionOption::on(ActionKind::TapToPay, id, PAY_WATERBEND));
            } else if helpers.creatures && due.generic > 0 && chars.is(CardType::Creature) {
                out.push(ActionOption::on(ActionKind::TapToPay, id, PAY_WATERBEND));
            } else if helpers.with.improvise && due.generic > 0 && chars.is(CardType::Artifact) {
                out.push(ActionOption::on(ActionKind::TapToPay, id, PAY_IMPROVISE));
            } else if helpers.with.convoke && chars.is(CardType::Creature) {
                let mut with = tapped.to_vec();
                with.push(chars.colors);
                if can_convoke(cost, &with) {
                    out.push(ActionOption::on(ActionKind::TapToPay, id, PAY_CONVOKE));
                }
            }
        }
    }
    out
}

/// Pays a mana cost (CR 601.2g-h): silently when the pool pays it in exactly one way and nothing else could pay any of
/// it, otherwise by asking, with the player's mana abilities, the pool's mana and whatever else may pay (convoke's
/// creatures, delve's cards, improvise's artifacts) offered, until nothing is owed. Once one of those has paid, no
/// mana ability is offered any more: the cost is being paid, which comes after every mana ability (CR 601.2g-h).
pub(crate) fn pay_mana_with(cx: &mut Cx, player: PlayerId, source: ObjectId, total: ManaCost, helpers: &Helpers) -> R<ManaPool> {
    Ok(pay_mana_used(cx, player, source, total, helpers)?.0)
}

/// `pay_mana_with`, and what the mana spent does to what it paid for (`ManaRestriction::effects`, CR 106.6).
pub(crate) fn pay_mana_used(cx: &mut Cx, player: PlayerId, source: ObjectId, total: ManaCost, helpers: &Helpers) -> R<(ManaPool, u8)> {
    let mut owed = total;
    let mut paid = ManaPool::default();
    let mut used = 0u8;
    // What has paid instead of mana: creatures tapped for convoke (by their colors), and cards exiled or artifacts
    // tapped for {1} each.
    let mut convoking: Vec<ColorSet> = Vec::new();
    let mut generic_paid: u8 = 0;
    // The cards delve exiles: one payment (CR 702.66a, 601.2h), exiled together when the cost is paid, so that a
    // "one or more cards are put into exile" ability triggers once (CR 603.2c).
    let mut delved: Vec<ObjectId> = Vec::new();
    // How many of the generic mana tapping has paid for as waterbend (CR 701.67b), against what waterbend may pay.
    let mut bent: u8 = 0;
    loop {
        let mut cost = owed;
        cost.generic = cost.generic.saturating_sub(generic_paid);
        let due = convoked(&cost, &convoking);
        if due.is_zero() {
            break;
        }
        let pool = if helpers.no_mana { ManaPool::default() } else { spendable(cx, player, source, helpers.purpose) };
        let candidates =
            if helpers.any() { pay_candidates(cx, player, source, helpers, &cost, &due, &convoking, &delved, bent) } else { Vec::new() };
        let mut decided = ways(&pool, &due);
        // Ordinary mana and restricted mana of a kind are different units (CR 106.6): spending only some of that kind
        // is the player's choice, even when the pool pays the cost one way.
        if let Ways::One(spent) = decided {
            let player_state = cx.s().player(player);
            let restricted = restricted_for(cx, player, source, helpers.purpose);
            let mixed = Mana::ALL.iter().any(|&m| {
                let held_restricted: u8 = restricted.iter().map(|(_, p)| p.get(m)).sum();
                held_restricted > 0
                    && player_state.pool.get(m) > 0
                    && spent[m as usize] > 0
                    && spent[m as usize] < player_state.pool.get(m) + held_restricted
            });
            if mixed {
                decided = Ways::Many;
            }
        }
        let spend_pool = |cx: &mut Cx, paid: &mut ManaPool, used: &mut u8, spent: [u8; 6]| {
            *used |= take_mana(cx, player, source, helpers.purpose, &spent);
            for mana in Mana::ALL {
                paid.0[mana as usize] += spent[mana as usize];
            }
        };
        if let Ways::One(spent) = decided
            && candidates.is_empty()
        {
            spend_pool(cx, &mut paid, &mut used, spent);
            break;
        }
        let paying = !convoking.is_empty() || generic_paid > 0;
        let mut options = if paying { Vec::new() } else { mana_options_while_paying(cx, player, source, helpers.purpose) };
        if matches!(decided, Ways::One(_)) {
            // The pool's one way of paying, which the player may take or pay some other way instead.
            options.push(ActionOption::simple(ActionKind::ConfirmCasting));
        } else if !helpers.no_mana {
            let restricted = restricted_for(cx, player, source, helpers.purpose);
            options.extend(assign_options(&cx.s().player(player).pool, &restricted, &due));
        }
        options.extend(candidates);
        options.push(ActionOption::simple(ActionKind::CancelCasting));
        let shown = if helpers.any() {
            let mut t = total;
            t.generic = t.generic.saturating_sub(generic_paid);
            convoked(&t, &convoking)
        } else {
            total
        };
        let ask = RequestKind::PayCosts { source, total: shown, remaining: due, options, announcing: false, x: 0, loyalty: false, attackers: helpers.attackers.to_vec(), life: helpers.life };
        let answer = cx.ask_core(player, ask, true)?;
        match answer {
            Answer::Act(action) => match action.kind {
                ActionKind::ActivateAbility => {
                    let from = action.source.expect("source");
                    activate_mana(cx, player, from, action.index, action.color, action.mana == Some(Mana::C), &action.targets)?;
                }
                ActionKind::AssignMana => {
                    let mana = action.mana.expect("mana");
                    match action.restriction {
                        // Only mana of a restriction that may pay for this (CR 106.6): `validate` holds the answer
                        // to the options, which offer no other.
                        Some(r) => {
                            used |= r.effects();
                            cx.st().spend_restricted_mana(player, r, mana, 1);
                        }
                        None => cx.st().update_player(player, |p| p.spend_mana(mana, 1)),
                    }
                    paid.0[mana as usize] += 1;
                    spend(&mut owed, mana);
                }
                ActionKind::ConfirmCasting => {
                    let Ways::One(spent) = decided else { return Err(Stop::Cancel) };
                    spend_pool(cx, &mut paid, &mut used, spent);
                    break;
                }
                ActionKind::TapToPay => {
                    let id = action.source.expect("source");
                    let colors = cx.chars(id).colors;
                    ops::tap(cx, id);
                    if action.index == PAY_CONVOKE {
                        convoking.push(colors);
                    } else {
                        generic_paid += 1;
                        if action.index == PAY_WATERBEND {
                            bent = bent.saturating_add(1);
                        }
                    }
                }
                ActionKind::ExileToPay => {
                    let id = action.source.expect("source");
                    delved.push(id);
                    generic_paid += 1;
                }
                _ => return Err(Stop::Cancel),
            },
            _ => return Err(Stop::Cancel),
        }
    }
    if !delved.is_empty() {
        ops::exile_all(cx, &delved);
    }
    Ok((paid, used))
}

/// Whether `player` could pay `life` life now (CR 119.4): any amount of 0 (CR 119.4b), otherwise only with a life total
/// at least that high. ("Can't lose life", CR 119.8, would make it unpayable too; no card in content has it.)
pub fn can_pay_life(cx: &Cx, player: PlayerId, life: u8) -> bool {
    life == 0 || cx.s().player(player).life >= i32::from(life)
}

/// A cost paid as a spell or ability resolves (CR 118.12, 608.2d): "[payer] may pay [cost]. If they do", and "unless
/// [payer] pays [cost]" (CR 118.12a), which means the same. It is one payment of the whole cost, its mana and its life
/// together. The payer may activate mana abilities while asked, when the cost has mana in it (CR 605.3a, 118.3c); paying
/// is offered once the pool covers the mana and the life total covers the life (CR 118.3), and the life is judged as it
/// is paid (CR 119.4): a player without that much life is not asked at all, and one whose mana ability takes it below
/// that (Mana Confluence's life, a painland's damage) can no longer pay, and is offered only declining. Whether it was
/// paid.
pub fn may_pay(cx: &mut Cx, payer: PlayerId, source: ObjectId, subject: ObjectId, cost: &Cost, question: &'static Question) -> R<bool> {
    assert!(only_mana_and_life(cost), "a cost paid during resolution is mana and life; its other parts are not built");
    Ok(pay_during_resolution(cx, payer, source, subject, cost, question, None)?.is_some())
}

/// The most X a payer may raise "you may pay {X}" to while choosing it: past what any game could produce, a bound for a
/// player who raises without paying.
const MOST_X_PAID: u16 = 1000;

/// "You may pay {X}" as a spell or ability resolves (CR 118.12, 608.2d), X chosen by the payer as they pay it, since the
/// text does not define it (CR 107.3f): one question (`RequestKind::UnlessPays` with its `x`) raises and lowers X, makes mana
/// (CR 605.3a), and pays {X} once the pool covers it, or declines. Any X may be chosen, 0 included (paying {0} is still
/// paying, CR 118.5). The X paid, or `None` when the payer declined: "when you do" (CR 603.12) asks whether it was paid, and
/// "X" afterwards is the X paid. `cost` is the cost with its {X} (and any other mana or life beside it).
pub fn may_pay_x(cx: &mut Cx, payer: PlayerId, source: ObjectId, subject: ObjectId, cost: &Cost, question: &'static Question) -> R<Option<u16>> {
    assert!(cost.mana.x > 0, "may_pay_x pays a cost with an {{X}} in it");
    let without_x = Cost { mana: ManaCost { x: 0, ..cost.mana }, ..*cost };
    assert!(only_mana_and_life(&without_x), "a cost paid during resolution is mana and life; its other parts are not built");
    pay_during_resolution(cx, payer, source, subject, cost, question, Some(0))
}

/// The payment `may_pay` and `may_pay_x` ask: with `x` the X chosen so far for a cost with an {X}, else none. What was
/// paid: the X (0 for a cost without one), or `None` declined.
fn pay_during_resolution(
    cx: &mut Cx,
    payer: PlayerId,
    source: ObjectId,
    subject: ObjectId,
    cost: &Cost,
    question: &'static Question,
    mut x: Option<u16>,
) -> R<Option<u16>> {
    let life = cost.life;
    if !can_pay_life(cx, payer, life) {
        return Ok(None);
    }
    loop {
        // The cost as X now stands: each {X} is X generic mana (CR 107.3f).
        let total = match x {
            Some(n) => {
                let generic = u32::from(cost.mana.generic) + u32::from(n) * u32::from(cost.mana.x);
                ManaCost { generic: generic.min(255) as u8, x: 0, ..cost.mana }
            }
            None => cost.mana,
        };
        // Once a mana ability has cost the payer the life the payment needs, there is nothing left to do but decline.
        let mut options = Vec::new();
        let pool = spendable(cx, payer, source, Paying::Other);
        let remaining = uncovered(&pool, &total);
        let life_payable = can_pay_life(cx, payer, life);
        if life_payable {
            if total != ManaCost::ZERO || x.is_some() {
                options = mana_options_while_paying(cx, payer, source, Paying::Other);
            }
            if total == ManaCost::ZERO || !matches!(ways(&pool, &total), Ways::None) {
                options.push(ActionOption::simple(ActionKind::PayCost));
            }
            if let Some(n) = x {
                // The generic mana a cost holds is a byte: X goes no higher than that leaves room for.
                if n < MOST_X_PAID && u32::from(total.generic) + u32::from(cost.mana.x) <= 255 {
                    options.push(ActionOption::simple(ActionKind::IncreaseX));
                }
                if n > 0 {
                    options.push(ActionOption::simple(ActionKind::DecreaseX));
                }
            }
        }
        options.push(ActionOption::simple(ActionKind::DeclineCost));
        let ask = RequestKind::UnlessPays { source, subject, total, remaining, life, life_payable, question, options, x };
        let answer = cx.ask_core(payer, ask, false)?;
        let Answer::Act(action) = answer else { return Ok(None) };
        match action.kind {
            ActionKind::ActivateAbility => {
                let from = action.source.expect("source");
                activate_mana(cx, payer, from, action.index, action.color, action.mana == Some(Mana::C), &action.targets)?;
            }
            ActionKind::IncreaseX => x = x.map(|n| n + 1),
            ActionKind::DecreaseX => x = x.map(|n| n.saturating_sub(1)),
            ActionKind::PayCost => {
                if total != ManaCost::ZERO {
                    pay_mana(cx, payer, source, total)?;
                }
                if life > 0 {
                    ops::change_life(cx, payer, -i32::from(life));
                }
                return Ok(Some(x.unwrap_or(0)));
            }
            _ => return Ok(None),
        }
    }
}

/// Whether a cost is only mana and life, the parts a payment during resolution pays. Every field is named, so a new kind
/// of cost part is looked at here.
fn only_mana_and_life(cost: &Cost) -> bool {
    let Cost {
        mana,
        life: _,
        tap,
        sacrifice_self,
        discard_self,
        permanents,
        loyalty,
        remove_counters,
        put_counters,
        remove_any_counter,
        remove_counters_among,
        exile_self,
        discard,
        pay_x_life,
        exile_from_graveyard,
        exile_from_graveyard_of,
        exert,
        tap_creatures,
        emerge,
        exile_from_hand,
        tap_permanents,
        unattach_self,
        return_self,
        variable,
        discard_hand,
        collect_evidence,
        behold,
        blight,
        tap_to_reduce,
        materials,
    } = *cost;
    mana.x == 0
        && !(tap || sacrifice_self || discard_self || remove_any_counter || exile_self || pay_x_life || exert || emerge)
        && !(tap_permanents || unattach_self || return_self || discard_hand || tap_to_reduce)
        && permanents.is_none()
        && loyalty.is_none()
        && remove_counters.is_none()
        && remove_counters_among.is_none()
        && put_counters.is_none()
        && discard.is_none()
        && exile_from_graveyard == 0
        && exile_from_graveyard_of.is_none()
        && tap_creatures.is_none()
        && exile_from_hand.is_none()
        && variable.is_none()
        && collect_evidence == 0
        && behold.is_none()
        && blight == 0
        && materials.is_none()
}

/// Whether an object is somewhere it can be found by its id: still the same object.
#[allow(dead_code)]
fn is_live(cx: &Cx, id: ObjectId) -> bool {
    live(cx.s(), id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_generic_reduction_takes_only_generic_mana_off() {
        // CR 118.7a: {2} off {1}{W}{W} leaves {W}{W}, never touching the white.
        assert_eq!(reduce(mana("{1}{W}{W}"), &mana("{2}")), mana("{W}{W}"));
    }

    #[test]
    fn a_colored_reduction_takes_its_own_color_off_and_the_rest_from_generic() {
        // CR 702.193b, 118.7: {3}{W}{W} off {5}{W}{W} is {W}{W} off the white and {3} off the generic.
        assert_eq!(reduce(mana("{5}{W}{W}"), &mana("{3}{W}{W}")), mana("{2}"));
        // CR 118.7b: blue the cost does not have comes off its generic mana.
        assert_eq!(reduce(mana("{3}{R}"), &mana("{U}")), mana("{2}{R}"));
        // CR 118.7c: more white than the cost has; the excess comes off its generic mana.
        assert_eq!(reduce(mana("{2}{W}"), &mana("{W}{W}")), mana("{1}"));
        // CR 118.7d: colorless likewise.
        assert_eq!(reduce(mana("{2}{C}"), &mana("{C}{C}")), mana("{1}"));
        // CR 118.7f: a Phyrexian symbol takes off mana of its color.
        assert_eq!(reduce(mana("{1}{G}"), &mana("{G/P}")), mana("{1}"));
    }

    #[test]
    #[should_panic(expected = "CR 118.7e")]
    fn a_reduction_by_hybrid_mana_is_refused() {
        // CR 118.7e: the payer would choose a half as the reduction is applied; that choice is not built.
        reduce(mana("{3}{G}{W}"), &mana("{2}{G/W}"));
    }

    #[test]
    fn a_reduction_takes_nothing_below_zero() {
        // CR 601.2f: the mana is reduced to nothing, {0}, and no further.
        assert_eq!(reduce(mana("{1}{U}"), &mana("{3}{U}{U}")), ManaCost::ZERO);
    }
}
