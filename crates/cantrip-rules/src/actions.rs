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
        AbilityDef::Triggered(_) => None,
    }
}

/// Whether the non-mana parts of a cost could be paid (CR 602.2b, 118.3): an activation that cannot be is not
/// offered at all.
fn can_pay_other(cx: &Cx, source: ObjectId, controller: PlayerId, cost: &Cost) -> bool {
    let obj = cx.s().obj(source);
    if cost.tap && (obj.tapped || summoning_sick(cx.cards(), cx.s(), source)) {
        return false;
    }
    if let Some(loyalty) = cost.loyalty
        && loyalty < 0
        && obj.counters(CounterKind::Loyalty) < -(loyalty as i16)
    {
        return false;
    }
    if cost.life > 0 && cx.s().player(controller).life < cost.life as i32 {
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
    if cost.exile_from_graveyard > 0
        && cx.s().zone(Zone::graveyard(controller)).iter().filter(|&&c| c != source).count() < cost.exile_from_graveyard as usize
    {
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
        let candidates = cx.permanents_of(controller).into_iter().filter(|&id| (p.filter)(cx, &asker, id)).count();
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
    if let Some(t) = cost.tap_creatures {
        let (_, powers) = tap_candidates(cx, source, controller, &t);
        if powers.iter().map(|&p| p as u32).sum::<u32>() < t.power as u32 {
            return false;
        }
    }
    true
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
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Behold, cards: eligible.clone(), eligible, count: n, up_to: false, kind: "" };
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
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Blight, cards: eligible.clone(), eligible, count: 1, up_to: false, kind: "" };
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
fn limit_reached(cx: &Cx, id: ObjectId, index: usize, def: &ActivatedDef) -> bool {
    // Exhaust (CR 702.177a): this permanent has activated this ability before, in any turn. A permanent that left
    // and came back is another object, with its exhaust abilities unused.
    if def.exhaust
        && let Some(ability) = cx.chars(id).abilities().nth(index)
        && cx
            .history()
            .select(EventQuery::kind(EventMask::ABILITY_ACTIVATED).object(id))
            .any(|r| matches!(r.event, Event::AbilityActivated { ability: a, .. } if a == ability))
    {
        return true;
    }
    if !def.once_each_turn {
        return false;
    }
    let obj = cx.s().obj(id);
    obj.activated_turn == cx.s().turn.number && obj.activated_mask & (1 << index) != 0
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
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::Discard, cards: hand, eligible, count: n, up_to: false, kind: "" };
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
    let ask = RequestKind::ChooseCards { source, purpose: Purpose::ExileToPay, cards: hand, eligible, count: n, up_to: false, kind: "" };
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

/// Chooses the cards a cost exiles from the graveyard (CR 601.2h), always asked, even when the choice is forced.
fn choose_exiles(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    if cost.exile_from_graveyard == 0 {
        return choose_evidence(cx, player, source, cost);
    }
    let cards: Vec<ObjectId> = cx.s().zone(Zone::graveyard(player)).iter().copied().filter(|&c| c != source).collect();
    let ask = RequestKind::ChooseCards {
        source,
        purpose: Purpose::ExileToPay,
        cards: cards.clone(),
        eligible: cards,
        count: cost.exile_from_graveyard,
        up_to: false, kind: "" };
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

/// Chooses the permanents a cost names (CR 601.2h), always asked, even when the choice is forced.
fn choose_permanents(cx: &mut Cx, player: PlayerId, source: ObjectId, cost: &Cost) -> R<Vec<ObjectId>> {
    let Some(p) = cost.permanents else { return Ok(Vec::new()) };
    let asker = Asker { source, controller: player };
    let eligible: Vec<ObjectId> = cx.permanents_of(player).into_iter().filter(|&id| (p.filter)(cx, &asker, id)).collect();
    let purpose = if cost.tap_permanents {
        Purpose::TapToPay
    } else if p.to_hand {
        Purpose::ReturnToHand
    } else {
        Purpose::SacrificeCost
    };
    let ask = RequestKind::ChooseCards { source, purpose, cards: eligible.clone(), eligible, count: p.count, up_to: false, kind: "" };
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
    let ask = RequestKind::ChooseCards { source, purpose, cards: cards.clone(), eligible: cards, count: count.min(255) as u8, up_to, kind: "" };
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
                || limit_reached(cx, id, index, def)
                || !can_pay_other(cx, id, player, &def.cost)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            if let Some(condition) = def.condition
                && !condition(cx, &Asker { source: id, controller: player })
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
                if !a.covers_ability(&cost) {
                    continue;
                }
            }
            let mut option = ActionOption::on(ActionKind::ActivateAbility, id, index as u8);
            if let Some(p) = def.cost.permanents {
                let asker = Asker { source: id, controller: player };
                option.target_candidates = cx
                    .permanents_of(player)
                    .into_iter()
                    .filter(|&o| (p.filter)(cx, &asker, o))
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
            options.push(ActionOption::on(ActionKind::ActivateAbility, id, index as u8));
        }
    }
}

/// The candidates for one target slot (CR 601.2c), in a stable order: players in turn order, then objects on the
/// battlefield, the stack and in graveyards.
pub fn target_candidates(cx: &Cx, asker: &Asker, spec: &TargetSpec) -> Vec<Target> {
    target_candidates_given(cx, asker, spec, &[], Target::None, None)
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
    let s = cx.s();
    let cx = &crate::resolve::Given { q: cx, earlier, subject, slot_player };
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

/// Fills in an option's targets; false when a required slot has no candidate, so the option is not offered.
fn with_targets(cx: &Cx, asker: &Asker, specs: &[TargetSpec], option: &mut ActionOption) -> bool {
    if !targets_with_action(specs) {
        return specs.iter().all(|spec| !spec.required() || !target_candidates(cx, asker, spec).is_empty());
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
            modal.modes[i as usize].targets.iter().all(|spec| !spec.required() || !target_candidates(cx, asker, spec).is_empty())
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
/// does not choose its target after its kicker or X (CR 601.2b-c).
fn targets_with_cast(def: &'static CardDef) -> bool {
    def.modal.is_none() && !def.late_targets && def.divide.is_none() && targets_with_action(def.spell_targets())
}

/// Fills in a spell option's targets, or checks that its modes and targets could be announced: a modal spell needs
/// enough modes, one whose target waits on its kicker is offered if a target exists either way.
fn offer_spell(cx: &Cx, asker: &Asker, def: &'static CardDef, option: &mut ActionOption) -> bool {
    if let Some(modal) = def.modal {
        return modes_possible(cx, asker, modal);
    }
    let specs = def.spell_targets();
    if def.late_targets || def.divide.is_some() {
        return def.kicker.is_some() || specs.iter().all(|spec| !spec.required() || !target_candidates(cx, asker, spec).is_empty());
    }
    with_targets(cx, asker, specs, option)
}

/// A cost with the static abilities that change it applied (CR 601.2f, 118.7): increases first, then reductions, a
/// reduction taking nothing below zero generic mana (CR 118.7c).
pub fn apply_cost_changes(cx: &Cx, player: PlayerId, costed: Costed, cost: ManaCost) -> ManaCost {
    change_with(cx, &cost_change_sources(cx), player, costed, cost, None)
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
            cx.cards().cost_changes[cx.cards().current(obj).0 as usize]
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
        if id == self.spell { layers::chars_as_face(self.cx.cards(), self.cx.s(), id, self.face) } else { self.cx.chars(id) }
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.cx.card_id(def)
    }

    fn events(&self) -> &[Record] {
        self.cx.events()
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        self.cx.can_target(asker, target)
    }
}

fn change_with(cx: &Cx, sources: &[ObjectId], player: PlayerId, costed: Costed, mut cost: ManaCost, face: Option<CardId>) -> ManaCost {
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
        Costed::Ability(id, ability) => (id, Some(ability)),
    };
    let own: &[CostChange] = {
        let obj = cx.s().obj(id);
        match face { Some(face) => cx.card(face), None => cx.card(obj.copy.map_or(obj.card, |c| c.card)) }.own_cost_changes
    };
    if sources.is_empty() && own.is_empty() {
        return cost;
    }
    let (mut more, mut less) = (0i32, 0i32);
    for change in own {
        let delta = change(q, id, player, costed) as i32;
        if delta > 0 {
            more += delta;
        } else {
            less -= delta;
        }
    }
    let mut discount = 0u32;
    for &source in sources {
        let obj = cx.s().obj(source);
        let def = cx.card(cx.cards().current(obj));
        for change in def.cost_changes {
            let delta = change(q, source, player, costed) as i32;
            if delta > 0 {
                more += delta;
            } else {
                less -= delta;
            }
        }
        if let Some(ability) = ability {
            for d in def.ability_discounts {
                discount += d(q, source, player, id, ability) as u32;
            }
        }
    }
    cost.generic = (cost.generic as i32 + more - less).clamp(0, 255) as u8;
    // "Can't reduce the mana in that cost to less than one mana": applied to the cost the other changes left (CR 601.2f).
    if discount > 0 {
        let room = cost.mana_value().saturating_sub(1);
        cost.generic -= discount.min(room).min(cost.generic as u32) as u8;
    }
    cost
}

/// The least mana a spell could be cast for with this base cost, for the advice that a cast that cannot be paid is
/// not offered (CR 601.2, 118.3): X at 0, no kicker, the cheapest modes it must choose, Phyrexian symbols paid with
/// life where the life total allows.
fn least_cost(cx: &Cx, available: &Available, player: PlayerId, card: ObjectId, def: &CardDef, base: ManaCost) -> ManaCost {
    let mut cost = base;
    if let Some(extra) = def.additional_cost {
        cost = add_cost(cost, extra.mana);
    }
    if let Some(modal) = def.modal {
        let mut costs: Vec<ManaCost> = modal.modes.iter().map(|m| m.cost).collect();
        costs.sort_by_key(|c| c.mana_value());
        for c in costs.into_iter().take(modal.min.max(1) as usize) {
            cost = add_cost(cost, c);
        }
    }
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
    // A face cast from the hand (an Adventure, a half, a back face) is judged as that face (CR 715.4, 709.3, 712.8a).
    let whole = cx.card(cx.s().obj(card).card);
    let faces = [whole.adventure, whole.back, whole.bestow, whole.split.map(|s| s.0), whole.split.map(|s| s.1)];
    let face = faces.into_iter().flatten().find(|&f| std::ptr::eq(f, def)).map(|f| cx.card_id(f));
    change_with(cx, &available.changers, player, Costed::Spell(card), cost, face)
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
    if cx.s().obj(source).zone.is(ZoneKind::Stack) {
        let chars = cx.chars(source);
        let kind = SpellKind {
            artifact: chars.is(CardType::Artifact),
            instant_or_sorcery: chars.is(CardType::Instant) || chars.is(CardType::Sorcery),
            legendary: chars.supertypes.legendary,
        };
        a.covers_spell(cost, kind)
    } else {
        a.covers_ability(cost)
    }
}

/// The mana a player has to pay with: the pool and what their mana abilities add, summed once for every cost judged.
pub struct Available {
    fixed: [u8; 6],
    /// Mana of a color to be chosen.
    flex: u32,
    /// Some mana ability's yield is not stated: anything may be payable.
    unknown: bool,
    /// Mana that the mana abilities with a mana cost of their own (Chromatic Star's {1}) must be paid first: it is
    /// spent on them, so it is not there to pay for the cost they are for.
    consume: u32,
    /// The permanents that change what things cost.
    changers: Vec<ObjectId>,
    /// Restricted mana the pool and the one-ability mana sources hold, by `ManaRestriction` (CR 106.6): each kind pays
    /// only what `Available::covers_spell` and `covers_ability` say it may. Mana that pays anything (`GainsHaste`) is in
    /// `fixed`.
    restricted: [[u8; 6]; ManaRestriction::COUNT],
    /// Of the same, mana of any one color a mana ability adds (Great Hall of the Biblioplex, Delighted Halfling).
    restricted_flex: [u32; ManaRestriction::COUNT],
}

pub fn available(cx: &Cx, player: PlayerId) -> Available {
    available_without(cx, player, None)
}

/// What `player` has to pay with, `skip` apart: a permanent whose own {T} is in the cost being paid cannot also be
/// tapped for mana to pay it (CR 602.2b, 605.3b).
fn available_without(cx: &Cx, player: PlayerId, skip: Option<ObjectId>) -> Available {
    let mut fixed = cx.s().player(player).pool.0;
    let mut restricted = cx.s().player(player).restricted.map(|pool| pool.0);
    // Mana that pays anything is not a limit (CR 106.6): it is spent as ordinary mana is.
    for m in 0..6 {
        fixed[m] += std::mem::take(&mut restricted[ManaRestriction::GainsHaste as usize][m]);
    }
    let mut restricted_flex = [0u32; ManaRestriction::COUNT];
    let mut flex: u32 = 0;
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
        let mut widest = [0u32; ManaRestriction::COUNT + 1];
        let mut spent = 0u32;
        // The one ability that is not restricted, when there is exactly one: its yield is stated, not any color.
        let mut unrestricted: Option<&ActivatedDef> = None;
        let mut unrestricted_count = 0;
        for (index, ability) in chars.abilities().enumerate() {
            let Some(def) = activated(cx, ability) else { continue };
            if !def.is_mana_ability()
                || def.from != ZoneKind::Battlefield
                || limit_reached(cx, id, index, def)
                || !can_pay_other(cx, id, player, &def.cost)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            count += 1;
            only = Some(def);
            spent = spent.max(def.cost.mana.mana_value());
            let class = match def.restricted {
                Some(ManaRestriction::GainsHaste) | None => ManaRestriction::COUNT,
                Some(kind) => kind as usize,
            };
            if class == ManaRestriction::COUNT {
                unrestricted = Some(def);
                unrestricted_count += 1;
            }
            widest[class] = widest[class].max(def.adds.0.iter().map(|&n| n as u32).sum::<u32>() + def.adds_any as u32);
            if def.adds.0.iter().all(|&n| n == 0) && def.adds_any == 0 {
                return Available { fixed, flex, consume: 0, unknown: true, changers: cost_change_sources(cx), restricted, restricted_flex };
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
                flex += def.adds_any as u32;
            }
            (1, Some(def)) if let Some(kind) = def.restricted => {
                for m in 0..6 {
                    restricted[kind as usize][m] += def.adds.0[m];
                }
                restricted_flex[kind as usize] += def.adds_any as u32;
            }
            (1, Some(def)) => {
                for m in 0..6 {
                    fixed[m] += def.adds.0[m];
                }
                flex += def.adds_any as u32;
            }
            _ => {
                match unrestricted {
                    Some(def) if unrestricted_count == 1 => {
                        for m in 0..6 {
                            fixed[m] += def.adds.0[m];
                        }
                        flex += def.adds_any as u32;
                    }
                    _ => flex += widest[ManaRestriction::COUNT],
                }
                for kind in 0..ManaRestriction::COUNT {
                    restricted_flex[kind] += widest[kind];
                }
            }
        }
    }
    Available { fixed, flex, consume, unknown: false, changers: cost_change_sources(cx), restricted, restricted_flex }
}

/// What a spell is, as far as the mana limited to some spells is concerned (CR 106.6).
#[derive(Clone, Copy)]
pub struct SpellKind {
    pub artifact: bool,
    pub instant_or_sorcery: bool,
    pub legendary: bool,
}

/// The kind of spell a card or face is when cast.
fn spell_kind(def: &CardDef) -> SpellKind {
    SpellKind {
        artifact: def.is(CardType::Artifact),
        instant_or_sorcery: def.is(CardType::Instant) || def.is(CardType::Sorcery),
        legendary: def.supertypes.legendary,
    }
}

impl Available {
    /// This with the restricted mana of the kinds in `kinds` (a bit for each `ManaRestriction`) added to what any spell
    /// or ability may spend.
    fn with_restricted(&self, kinds: u8) -> Available {
        let mut with = Available {
            fixed: self.fixed,
            flex: self.flex,
            consume: self.consume,
            unknown: self.unknown,
            changers: Vec::new(),
            restricted: [[0; 6]; ManaRestriction::COUNT],
            restricted_flex: [0; ManaRestriction::COUNT],
        };
        for kind in 0..ManaRestriction::COUNT {
            if kinds & (1 << kind) != 0 {
                for m in 0..6 {
                    with.fixed[m] += self.restricted[kind][m];
                }
                with.flex += self.restricted_flex[kind];
            }
        }
        with
    }

    /// Whether `cost` could be paid for a spell of this kind: restricted mana that may be spent on a spell like it counts
    /// (CR 106.6, 601.2g).
    pub fn covers_spell(&self, cost: &ManaCost, kind: SpellKind) -> bool {
        let mut kinds = 0u8;
        if kind.artifact {
            kinds |= 1 << ManaRestriction::NotNonartifactSpells as u8 | 1 << ManaRestriction::ArtifactSpells as u8;
        }
        if kind.instant_or_sorcery {
            kinds |= 1 << ManaRestriction::InstantOrSorcerySpells as u8;
        }
        if kind.legendary {
            kinds |= 1 << ManaRestriction::LegendaryUncounterable as u8;
        }
        self.with_restricted(kinds).covers(cost)
    }

    /// Whether `cost` could be paid for an ability: restricted mana that is not limited to spells counts.
    pub fn covers_ability(&self, cost: &ManaCost) -> bool {
        self.with_restricted(1 << ManaRestriction::NotNonartifactSpells as u8).covers(cost)
    }

    /// Whether `cost` could be paid with it (advice, CR 601.2g).
    pub fn covers(&self, cost: &ManaCost) -> bool {
        if self.unknown || cost.mana_value() == 0 {
            return true;
        }
        let (fixed, flex) = (self.fixed, self.flex);
        // What a mana ability costs is paid from what is already there.
        if flex > 0 && (fixed.iter().map(|&n| n as u32).sum::<u32>()) < self.consume {
            return false;
        }
        let mut left = fixed;
        let mut flex_needed: u32 = 0;
        for m in 0..6 {
            let need = cost.symbols[m];
            let have = left[m];
            let used = need.min(have);
            left[m] -= used;
            if need > used {
                if m == Mana::C as usize {
                    return false;
                }
                flex_needed += (need - used) as u32;
            }
        }
        for (a, b) in cost.hybrid.iter().flatten() {
            if left[*a as usize] > 0 {
                left[*a as usize] -= 1;
            } else if left[*b as usize] > 0 {
                left[*b as usize] -= 1;
            } else {
                flex_needed += 1;
            }
        }
        if flex_needed > flex {
            return false;
        }
        let spare: u32 = (left.iter().map(|&n| n as u32).sum::<u32>() + (flex - flex_needed)).saturating_sub(self.consume);
        spare >= cost.generic as u32
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
                || limit_reached(cx, id, index, def)
                || static_rules::activation_barred(cx, &forbidders, id, def)
            {
                continue;
            }
            match def.timing {
                // Foretell (CR 702.143a): during your own turn.
                Timing::OwnTurn if cx.s().turn.active != player => continue,
                // Suspend (CR 702.62a): whenever the card could be cast.
                Timing::AsCast
                    if !(chars.is(CardType::Instant) || chars.has(Keyword::Flash) || sorcery_timing(cx, player))
                        || !static_rules::may_cast(cx, player) =>
                {
                    continue;
                }
                _ => {}
            }
            let sorcery = def.timing == Timing::Sorcery || def.cost.loyalty.is_some();
            // A static ability may lift the timing restriction of a loyalty ability (CR 606.3).
            if sorcery && !sorcery_timing(cx, player) && !static_rules::instant_activation(cx, player, id, index as u8) {
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
            let owed = if def.special_action { special_action_cost(cx, id, ability, def) } else { def.cost.mana };
            if owed.x == 0 && owed.mana_value() > 0 && !def.tap_creatures_for_mana && !def.waterbend {
                // The source's own mana abilities cannot pay for an ability that taps it (CR 602.2b, 605.3b).
                let own = (def.cost.tap && has_mana_source(cx, id, player)).then(|| available_without(cx, player, Some(id)));
                let a = own.as_ref().unwrap_or_else(|| available.get_or_init(|| self::available(cx, player)));
                // A special action is no activated ability (CR 116.2): what changes the cost of activating abilities does not
                // touch it, and `special_action` pays the cost as it stands.
                let mut cost = if def.special_action { owed } else { change_with(cx, &a.changers, player, Costed::Ability(id, ability), owed, None) };
                // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
                if static_rules::spend_any_color_for(cx, player, id) {
                    cost = ManaCost { generic: cost.mana_value().min(255) as u8, ..ManaCost::ZERO };
                }
                if !a.covers_ability(&cost) && static_rules::activation_alternative(cx, player, id, def).is_none() {
                    continue;
                }
            }
            let mut option = ActionOption::on(ActionKind::ActivateAbility, id, index as u8);
            let possible = match def.modal {
                Some(modal) => modes_possible(cx, &asker, modal),
                None => with_targets(cx, &asker, def.targets, &mut option),
            };
            if possible {
                options.push(option);
            }
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
    let s = cx.s();
    s.permissions
        .iter()
        .find(|p| {
            p.card == card
                && p.player == player
                && !(p.grant.later_turns && p.turn == s.turn.number)
                && p.grant.when.is_none_or(|when| play_condition_holds(cx, player, when))
        })
        .copied()
}

/// Whether the condition on a permission to play holds now (CR 611.2): asked of the history of this turn, which no later
/// event can change, so it is a creature as it was declared that counts, dead or changed since.
fn play_condition_holds(cx: &Cx, player: PlayerId, when: PlayCondition) -> bool {
    match when {
        PlayCondition::AttackedWithThisTurn(kind) => cx.history().this_turn().iter().any(|r| {
            matches!(r.event, Event::Attacks { attacker, .. }
                if cx.s().obj(attacker).controller == player && cx.chars(attacker).has_subtype(kind))
        }),
    }
}

/// Whether an effect lets `player` play lands and cast spells from their graveyard now (`EffectKind::PlayFromGraveyard`).
fn graveyard_play(cx: &Cx, player: PlayerId) -> bool {
    cx.s().effects.iter().any(|e| e.kind == EffectKind::PlayFromGraveyard && e.controller == player)
}

/// What `player`'s permanents let them do about land plays (CR 305.2): the land plays beyond the one, whether there is
/// no limit, and whether lands may be played from the graveyard.
fn land_rules(cx: &Cx, player: PlayerId) -> (u32, bool, bool, bool) {
    // "Until end of turn, you may play lands and cast spells from your graveyard" (CR 305.1, 601.3): Yawgmoth's Will.
    let (mut extra, mut unlimited, mut graveyard, mut top) = (0u32, false, graveyard_play(cx, player), false);
    if !cx.cards().any_play_rules {
        return (extra, unlimited, graveyard, top);
    }
    for &id in cx.s().battlefield() {
        let obj = cx.s().obj(id);
        let card = obj.copy.map_or(obj.card, |c| c.card);
        if obj.controller != player || !cx.cards().play_rules[card.0 as usize] {
            continue;
        }
        for rule in cx.card(card).play_rules {
            match rule {
                PlayRule::ExtraLandPlays(n) => extra += *n as u32,
                PlayRule::AnyNumberOfLandPlays => unlimited = true,
                PlayRule::LandsFromGraveyard => graveyard = true,
                PlayRule::LibraryTop => top = true,
                PlayRule::LibraryTopSpells(_) => {}
                PlayRule::NoMaximumHandSize => {}
            }
        }
    }
    (extra, unlimited, graveyard, top)
}

/// Whether a permanent `player` controls, an emblem they have (CR 114.4), or an effect a resolved spell made for the rest
/// of the game says they have no maximum hand size (CR 402.2).
pub fn no_maximum_hand_size(cx: &Cx, player: PlayerId) -> bool {
    if cx.s().effects.iter().any(|e| e.kind == EffectKind::NoMaximumHandSize && e.controller == player) {
        return true;
    }
    cx.cards().any_play_rules
        && cx.s().battlefield().iter().chain(cx.s().zone(Zone::COMMAND)).any(|&id| {
            let obj = cx.s().obj(id);
            let card = obj.copy.map_or(obj.card, |c| c.card);
            obj.controller == player
                && cx.cards().play_rules[card.0 as usize]
                && cx.card(card).play_rules.iter().any(|r| matches!(r, PlayRule::NoMaximumHandSize))
        })
}

fn land_plays(cx: &Cx, player: PlayerId, options: &mut Vec<ActionOption>) {
    let (extra, unlimited, from_graveyard, from_top) = land_rules(cx, player);
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
    // The land cards in the graveyard that a way a permanent gives lets the player play (CR 305.1, 604): once during each of
    // their turns, say. Looked for only when the graveyard holds a land card.
    let mut granted: Option<Vec<(ObjectId, PlayerId, &'static GrantedWay)>> = None;
    for &id in cx.s().zone(Zone::graveyard(player)) {
        if !cx.chars(id).is(CardType::Land) {
            continue;
        }
        let ways = granted.get_or_insert_with(|| granted_ways_of(cx.cards(), cx.s()));
        for (k, &(granter, controller, way)) in ways.iter().enumerate().take(8) {
            if way.lands && (way.cost)(cx, &Asker { source: granter, controller }, id, player).is_some() {
                options.push(ActionOption::on(ActionKind::PlayLand, id, WAY_GRANTED + k as u8));
            }
        }
    }
    // A land a card lets this player play from where it is (CR 305.1): "you may play that card".
    for p in &cx.s().permissions {
        if p.player == player
            && p.grant.lands
            && !cx.s().obj(p.card).zone.is(ZoneKind::Hand)
            && !(from_graveyard && cx.s().obj(p.card).zone == Zone::graveyard(player))
        {
            offer(p.card, options);
        }
    }
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
            cx.card(obj.copy.map_or(obj.card, |c| c.card)).cast_limit
        })
        .filter(|&n| n > 0)
        .min()
}

/// The ways the permanents on the battlefield give cards in graveyards to be cast (CR 604, 118.9): each with the permanent
/// and its controller, in the order a way number counts them. Nothing is allocated while no permanent has one.
fn granted_ways_of(cards: &CardTable, s: &GameState) -> Vec<(ObjectId, PlayerId, &'static GrantedWay)> {
    let mut ways = Vec::new();
    for &id in s.battlefield() {
        let obj = s.obj(id);
        let def = cards.get(cards.current(obj));
        for way in def.granted_ways {
            ways.push((id, obj.controller, way));
        }
    }
    ways
}

/// Whether the additional costs a card has (CR 118.8) could be paid, whatever way it is cast.
fn extra_payable(cx: &Cx, id: ObjectId, player: PlayerId, def: &CardDef) -> bool {
    def.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
        && (def.additional_choice.is_empty() || def.additional_choice.iter().any(|c| can_pay_other(cx, id, player, &c.cost)))
}

fn castable(cx: &Cx, player: PlayerId, available: &std::cell::OnceCell<Available>, instants: bool, options: &mut Vec<ActionOption>) {
    if cast_limit(cx).is_some_and(|limit| cx.history().casts_by(player).count() >= limit as usize) || !static_rules::may_cast(cx, player) {
        return;
    }
    // Types of spell the player may cast as though they had flash (CR 307.5).
    let flash = static_rules::flash_types(cx, player);
    // Or by a static ability's say-so about a kind of spell (Sigarda's Aid): looked for once, empty unless one is on the board.
    let flash_granters = static_rules::flash_granters(cx);
    // Any player's exile, for a card a permission lets them cast from there; their own other zones.
    let zones = [Zone::hand(player), Zone::graveyard(player)].into_iter().chain(cx.s().seats().map(Zone::exile));
    let granted: std::cell::OnceCell<Vec<(ObjectId, PlayerId, &'static GrantedWay)>> = std::cell::OnceCell::new();
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
            if zone.is(ZoneKind::Exile) && !prepared_copy && !adventured && !permitted {
                continue;
            }
            let def = cx.card(obj.card);
            if def.is(CardType::Land) {
                continue;
            }
            let asker = Asker { source: id, controller: player };
            if let Some(only_if) = def.cast_only_if
                && !only_if(cx, &asker)
            {
                continue;
            }
            // What a permission to cast it from here says to pay instead of its mana cost (CR 118.9).
            let grant = if permitted { permission_for(cx, id, player).map(|p| p.grant) } else { None };
            let instead = grant.and_then(|g| g.instead);
            let any_type = grant.is_some_and(|g| g.any_type);
            let more = grant.map_or(0, |g| g.more);
            let instant = (def.is(CardType::Instant) || def.keywords.has(Keyword::Flash) || def.cast_as_though_flash || def.types.0 & flash.0 != 0
                || (!flash_granters.is_empty() && static_rules::grants_flash(cx, &flash_granters, player, id)))
                && !grant.is_some_and(|g| g.sorcery_speed);
            if instant == instants {
                // Or by the card's own say-so, for its ordinary cost (CR 113.6b): "You may cast this card from your graveyard".
                let from_graveyard = zone.is(ZoneKind::Graveyard) && own && (def.keywords.has(Keyword::CastFromYourGraveyard) || graveyard_play(cx, player));
                // Or by retrace (CR 702.81a), printed or granted: for its ordinary cost and a land card discarded.
                let retrace = zone.is(ZoneKind::Graveyard)
                    && own
                    && !from_graveyard
                    && cx.chars(id).has(Keyword::Retrace)
                    && can_pay_other(cx, id, player, &RETRACE);
                if def.split.is_none()
                    && (zone.is(ZoneKind::Hand) || prepared_copy || adventured || permitted || from_graveyard || retrace)
                    && (def.has_cost || instead.is_some())
                    && def.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
                    && (def.additional_choice.is_empty()
                        || def.additional_choice.iter().any(|c| can_pay_other(cx, id, player, &c.cost)))
                    && {
                        let a = available.get_or_init(|| self::available(cx, player));
                        let mut least = least_cost(cx, a, player, id, def, instead.unwrap_or(def.cost));
                        least.generic = least.generic.saturating_add(more);
                        if any_type {
                            // Mana of any type pays every symbol of it (CR 609.4b).
                            least = ManaCost { generic: least.mana_value().min(255) as u8, ..ManaCost::ZERO };
                        }
                        covers_total(cx, a, player, id, def, least, 0)
                    }
                {
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
                    if offer_spell(cx, &asker, def, &mut option) {
                        options.push(option);
                    }
                }
                for (n, way) in def.ways.iter().enumerate() {
                    // Sneak and the like are offered below, at instant speed whatever the card is (CR 702.190a).
                    if way.instant_speed {
                        continue;
                    }
                    // An alternative cost may be used whatever zone the spell is cast from (CR 118.9, 601.2b): one that is
                    // paid from the hand, as Daze's, Fireblast's and Force of Will's are, also when a permission or the
                    // card's own say-so has it cast from elsewhere, as Yawgmoth's Will does.
                    let elsewhere = way.from == ZoneKind::Hand && !zone.is(ZoneKind::Hand) && (permitted || from_graveyard || retrace || adventured);
                    if (way.from != zone.kind() && !elsewhere) || !own {
                        continue;
                    }
                    if let Some(condition) = way.condition
                        && !condition(cx, &asker)
                    {
                        continue;
                    }
                    // An additional cost is owed with every way of casting it (CR 118.8, 601.2f).
                    if !can_pay_other(cx, id, player, &way.cost) || !extra_payable(cx, id, player, def) {
                        continue;
                    }
                    let a = available.get_or_init(|| self::available(cx, player));
                    let mut cost = least_cost(cx, a, player, id, def, way.cost.mana);
                    cost.generic = cost.generic.saturating_sub(emerge_reduction(cx, player, id, &way.cost));
                    if !covers_total(cx, a, player, id, def, cost, way.cost.exile_from_graveyard) {
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
                    let cost = least_cost(cx, a, player, id, def, overload);
                    if covers_total(cx, a, player, id, def, cost, 0) {
                        options.push(ActionOption::on(ActionKind::CastSpell, id, WAY_OVERLOAD));
                    }
                }
            }
            // An alternative cost that may be used whenever an instant could be cast, whatever the card is (sneak, CR 702.190a).
            if instants {
                for (n, way) in def.ways.iter().enumerate() {
                    if !way.instant_speed || way.from != zone.kind() || !own {
                        continue;
                    }
                    if let Some(condition) = way.condition
                        && !condition(cx, &asker)
                    {
                        continue;
                    }
                    if !can_pay_other(cx, id, player, &way.cost) || !extra_payable(cx, id, player, def) {
                        continue;
                    }
                    let a = available.get_or_init(|| self::available(cx, player));
                    let mut cost = least_cost(cx, a, player, id, def, way.cost.mana);
                    cost.generic = cost.generic.saturating_sub(emerge_reduction(cx, player, id, &way.cost));
                    if !covers_total(cx, a, player, id, def, cost, way.cost.exile_from_graveyard) {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, (n + 1) as u8);
                    if offer_spell(cx, &asker, def, &mut option) {
                        options.push(option);
                    }
                }
            }
            // The ways a permanent gives a card in its owner's graveyard to be cast (CR 604, 118.9).
            if instant == instants && zone.is(ZoneKind::Graveyard) && own {
                for (k, &(granter, controller, way)) in granted.get_or_init(|| granted_ways_of(cx.cards(), cx.s())).iter().enumerate().take(8) {
                    let Some(cost) = (way.cost)(cx, &Asker { source: granter, controller }, id, player) else { continue };
                    if !can_pay_other(cx, id, player, &cost) || !extra_payable(cx, id, player, def) {
                        continue;
                    }
                    let a = available.get_or_init(|| self::available(cx, player));
                    let least = least_cost(cx, a, player, id, def, cost.mana);
                    if !covers_total(cx, a, player, id, def, least, cost.exile_from_graveyard) {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_GRANTED + k as u8);
                    if offer_spell(cx, &asker, def, &mut option) {
                        options.push(option);
                    }
                }
            }
            // A split card's halves (CR 709.3): each cast for its own cost at its own speed, from the hand, or for an
            // aftermath half (CR 702.127a) from the graveyard only.
            if let Some((left, right)) = def.split {
                for (way, half) in [(WAY_SPLIT_LEFT, left), (WAY_SPLIT_RIGHT, right)] {
                    let from = if half.aftermath { ZoneKind::Graveyard } else { ZoneKind::Hand };
                    if !zone.is(from)
                        || !own
                        || (half.is(CardType::Instant) || half.keywords.has(Keyword::Flash) || half.types.0 & flash.0 != 0) != instants
                    {
                        continue;
                    }
                    let mut option = ActionOption::on(ActionKind::CastSpell, id, way);
                    if {
                        let a = available.get_or_init(|| self::available(cx, player));
                        a.covers_spell(&least_cost(cx, a, player, id, half, half.cost), spell_kind(half))
                    } && offer_spell(cx, &asker, half, &mut option)
                    {
                        options.push(option);
                    }
                }
            }
            // A modal double-faced card's back face, from the hand, at the speed of the back face's own type (CR 712.2,
            // 712.8a): the back face of a transforming card has no mana cost and is not cast.
            if zone.is(ZoneKind::Hand)
                && let Some(back) = def.back
                && back.has_cost
                && (back.is(CardType::Instant) || back.keywords.has(Keyword::Flash)) == instants
            {
                let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_BACK_FACE);
                if {
                    let a = available.get_or_init(|| self::available(cx, player));
                    a.covers_spell(&least_cost(cx, a, player, id, back, back.cost), spell_kind(back))
                } && offer_spell(cx, &asker, back, &mut option)
                {
                    options.push(option);
                }
            }
            // Bestow (CR 702.103a): from the hand, as the Aura it becomes, which needs a creature to enchant (CR 601.2c).
            if (zone.is(ZoneKind::Hand) || (zone.is(ZoneKind::Graveyard) && own && def.bestow_from_graveyard))
                && let Some(bestow) = def.bestow
                && (bestow.keywords.has(Keyword::Flash) || bestow.types.0 & flash.0 != 0) == instants
                && bestow.additional_cost.is_none_or(|extra| can_pay_other(cx, id, player, &extra))
            {
                let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_BESTOW);
                if {
                    let a = available.get_or_init(|| self::available(cx, player));
                    a.covers_spell(&least_cost(cx, a, player, id, bestow, bestow.cost), spell_kind(bestow))
                } && offer_spell(cx, &asker, bestow, &mut option)
                {
                    options.push(option);
                }
            }
            // Its Adventure, from the hand, at the speed of the Adventure's own type (CR 715.3).
            if zone.is(ZoneKind::Hand)
                && let Some(adventure) = def.adventure
                && (adventure.is(CardType::Instant) || adventure.keywords.has(Keyword::Flash) || adventure.types.0 & flash.0 != 0)
                    == instants
            {
                let mut option = ActionOption::on(ActionKind::CastSpell, id, WAY_ADVENTURE);
                if {
                    let a = available.get_or_init(|| self::available(cx, player));
                    a.covers_spell(&least_cost(cx, a, player, id, adventure, adventure.cost), spell_kind(adventure))
                } && offer_spell(cx, &asker, adventure, &mut option)
                {
                    options.push(option);
                }
            }
        }
    }
    // The top card of the library, for a player a permanent lets cast spells from there for their mana cost (CR 601.3).
    if cx.cards().any_play_rules
        && let Some(&id) = cx.s().zone(Zone::library(player)).first()
    {
        let def = cx.card(cx.s().obj(id).card);
        let allowed = cx.s().battlefield().iter().any(|&p| {
            let obj = cx.s().obj(p);
            let card = cx.cards().current(obj);
            obj.controller == player
                && cx.cards().play_rules[card.0 as usize]
                && cx.card(card).play_rules.iter().any(|r| matches!(r, PlayRule::LibraryTopSpells(f) if f(cx, id)))
        });
        let instant = def.is(CardType::Instant) || def.keywords.has(Keyword::Flash) || def.types.0 & flash.0 != 0;
        // Where a permanent lets them pay life instead, that cast below is the one offered.
        if allowed && !land_rules(cx, player).3 && !def.is(CardType::Land) && def.has_cost && instant == instants {
            let a = available.get_or_init(|| self::available(cx, player));
            let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
            if covers_total(cx, a, player, id, def, least_cost(cx, a, player, id, def, def.cost), 0)
                && offer_spell(cx, &Asker { source: id, controller: player }, def, &mut option)
            {
                options.push(option);
            }
        }
    }
    // The top card of the library, for a player a permanent lets cast from there for life (CR 601.3, 118.9).
    if land_rules(cx, player).3
        && let Some(&id) = cx.s().zone(Zone::library(player)).first()
    {
        let def = cx.card(cx.s().obj(id).card);
        let instant = def.is(CardType::Instant) || def.keywords.has(Keyword::Flash) || def.types.0 & flash.0 != 0;
        let life = Cost { life: def.cost.mana_value().min(255) as u8, ..Cost::NONE };
        if !def.is(CardType::Land) && def.has_cost && instant == instants && can_pay_other(cx, id, player, &life) {
            let mut option = ActionOption::on(ActionKind::CastSpell, id, 0);
            if offer_spell(cx, &Asker { source: id, controller: player }, def, &mut option) {
                options.push(option);
            }
        }
    }
}

/// The most an emerge cost could come down by right now (CR 702.119a): the mana values of the richest permanents it could
/// sacrifice. What it comes down by is not known until the player has chosen, so this is only what makes the cast worth
/// offering.
fn emerge_reduction(cx: &Cx, player: PlayerId, card: ObjectId, cost: &Cost) -> u8 {
    let Some(p) = cost.permanents.filter(|_| cost.emerge) else { return 0 };
    let asker = Asker { source: card, controller: player };
    let mut values: Vec<u32> =
        cx.permanents_of(player).into_iter().filter(|&id| (p.filter)(cx, &asker, id)).map(|id| cx.mana_value(id)).collect();
    values.sort_unstable_by(|a, b| b.cmp(a));
    values.iter().take(p.count as usize).sum::<u32>().min(255) as u8
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
    cx.chars(id).abilities().enumerate().any(|(index, ability)| {
        activated(cx, ability).is_some_and(|def| {
            def.is_mana_ability()
                && !limit_reached(cx, id, index, def)
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
        return a.covers_spell(&cost, spell_kind(def));
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
    if no_mana { cost.is_zero() } else { a.covers_spell(&cost, spell_kind(def)) }
}

/// The text an option is matched by and a menu row shows: an alternative cost's, a granted way's or an ability's. A cast a
/// standing permission lets a card have (CR 601.3) says what the permission changes: "Flashback {2}{R}" for a flashback granted
/// (Snapcaster Mage), "Cast without paying its mana cost", "Cast by paying {1}{U} instead".
pub fn describe(cards: &CardTable, state: &GameState, option: &ActionOption) -> std::borrow::Cow<'static, str> {
    let Some(source) = option.source else { return "".into() };
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
    describe_way(cards, state, option, source).into()
}

fn describe_way(cards: &CardTable, state: &GameState, option: &ActionOption, source: ObjectId) -> &'static str {
    match option.kind {
        ActionKind::CastSpell if option.index == WAY_ADVENTURE => {
            cards.get(state.obj(source).card).adventure.map_or("", |a| a.name)
        }
        _ if option.index == WAY_BACK_FACE => cards.get(state.obj(source).card).back.map_or("", |b| b.name),
        ActionKind::CastSpell if option.index == WAY_BESTOW => "Bestow",
        ActionKind::CastSpell | ActionKind::PlayLand if (WAY_GRANTED..WAY_GRANTED + 8).contains(&option.index) => {
            granted_ways_of(cards, state).get((option.index - WAY_GRANTED) as usize).map_or("", |&(_, _, way)| way.text)
        }
        ActionKind::CastSpell if option.index == WAY_SPLIT_LEFT => cards.get(state.obj(source).card).split.map_or("", |h| h.0.name),
        ActionKind::CastSpell if option.index == WAY_SPLIT_RIGHT => cards.get(state.obj(source).card).split.map_or("", |h| h.1.name),
        ActionKind::CastSpell if option.index > 0 && option.index < WAY_SPLIT_RIGHT => cards.get(state.obj(source).card).ways[option.index as usize - 1].text,
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
            let chars = layers::chars(cards, state, source);
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
            let granted = (WAY_GRANTED..WAY_GRANTED + 8).contains(&action.index).then(|| {
                let (granter, _, way) = granted_ways_of(cx.cards(), cx.s())[(action.index - WAY_GRANTED) as usize];
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
            // Cast from where a permission allows (CR 601.3), for the cost it names instead of the mana cost.
            let permitted = if action.index == 0 { permission_for(cx, source, player) } else { None };
            let instead = permitted.and_then(|p| p.grant.instead);
            let any_type = permitted.is_some_and(|p| p.grant.any_type);
            let more = permitted.map_or(0, |p| p.grant.more);
            // From the top of a library by a permanent's permission: life instead of the mana cost (CR 118.9).
            let (instead, life) = if cx.s().obj(source).zone.is(ZoneKind::Library) && land_rules(cx, player).3 {
                (Some(ManaCost::ZERO), cx.card(cx.s().obj(source).card).cost.mana_value().min(255) as u8)
            } else {
                (instead, 0)
            };
            let exile_after = permitted.is_some_and(|p| p.grant.exile_after);
            cast(cx, player, source, action.index, &action.targets, CastTerms { instead, any_type, life, more, exile_after, copy: false, way: None }, false)?
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

fn stack_item(kind: StackKind, object: ObjectId, ability: AbilityRef, controller: PlayerId, targets: &[Target]) -> StackItem {
    let mut item = StackItem {
        kind,
        object,
        ability,
        controller,
        targets: [Target::None; cantrip_state::MAX_TARGETS],
        target_count: targets.len() as u8,
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
    let CastTerms { instead, any_type, exile_after, life, more, copy: _, way: _ } = terms;
    let printed = cx.s().obj(card).card;
    let whole = cx.card(printed);
    // Cast as its Adventure (CR 715.3), it is that spell and nothing else on the stack (CR 715.3b).
    let adventure = if way == WAY_ADVENTURE { whole.adventure } else { None };
    // Retrace (CR 702.81a): cast from the graveyard for its ordinary cost by a card's say-so that is not its own, a land
    // card discarded in addition to paying its other costs.
    let retraced = way == 0
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
    let def = adventure.or(back).or(bestow).or(half).unwrap_or(whole);
    let face = cx.card_id(def);
    // Aftermath (CR 702.127a): exiled instead of put anywhere else as it leaves the stack.
    let exile_after = exile_after || def.aftermath;
    let mut cost = match (way, instead) {
        (_, Some(instead)) => Cost::mana(instead),
        (0, _) => Cost::mana(def.cost),
        (WAY_ADVENTURE | WAY_BACK_FACE | WAY_BESTOW | WAY_SPLIT_LEFT | WAY_SPLIT_RIGHT, _) => Cost::mana(def.cost),
        (w, _) if (WAY_GRANTED..WAY_GRANTED + 8).contains(&w) => {
            let (granter, controller, rule) = granted_ways_of(cx.cards(), cx.s())[(w - WAY_GRANTED) as usize];
            (rule.cost)(cx, &Asker { source: granter, controller }, card, player).unwrap_or(Cost::mana(def.cost))
        }
        (WAY_OVERLOAD, _) => Cost::mana(whole.overload.unwrap_or(def.cost)),
        _ => whole.ways[way as usize - 1].cost,
    };
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
    if (WAY_GRANTED..WAY_GRANTED + 8).contains(&way) {
        let (granter, _, _) = granted_ways_of(cx.cards(), cx.s())[(way - WAY_GRANTED) as usize];
        let ability = AbilityRef { card: cx.s().obj(granter).card, index: GRANTED_WAY_USED };
        cx.record(Event::AbilityActivated { source: granter, ability, player });
        // "If you do, it gains ...": the spell carries it to the permanent it becomes.
        if let Some(n) = granted_ways_of(cx.cards(), cx.s())[(way - WAY_GRANTED) as usize].2.gains {
            let ability = AbilityRef::delayed(cx.s().obj(granter).card, n);
            cx.st().update_obj(spell, |o| o.marks.gains = Some(ability));
        }
    }
    // A permission is used up by the cast (CR 400.7: the card is a new object).
    if !cx.s().permissions.is_empty() {
        cx.st().permissions_retain(|p| p.card != card);
    }
    if adventure.is_some() || back.is_some() || bestow.is_some() || half.is_some() {
        cx.st().update_obj(spell, |o| {
            o.copy = Some(Copiable::of(face))
        });
    }
    let mut item = stack_item(StackKind::Spell, spell, AbilityRef { card: face, index: u8::MAX }, player, targets);
    item.way = way;
    item.exile_after = exile_after;
    // Overload (CR 702.96a): "each" for "target", so nothing is targeted.
    let specs: &[TargetSpec] = if way == WAY_OVERLOAD { &[] } else { def.spell_targets() };
    // CR 118.8: an additional cost is owed with every way of casting it; its mana joins the total.
    let mut extra = def.additional_cost.unwrap_or(Cost::NONE);
    if retraced {
        extra = extra.and(RETRACE);
    }
    cost.mana = add_cost(cost.mana, extra.mana);

    // CR 601.2b: a modal spell's modes are chosen first (CR 700.2), each adding its cost (spree, escalate).
    let mut mask = 0u8;
    if let Some(modal) = def.modal {
        mask = choose_modes(cx, player, spell, item.ability, modal, cost.mana)?;
        cost.mana = modal_cost(modal, mask, cost.mana);
        item.modes = mask;
        // CR 702.120a: the rest of the escalate cost (a discard) is an additional cost, chosen and paid with the others.
        let mut escalate = escalate_cost(modal, mask);
        escalate.mana = ManaCost::ZERO;
        extra = extra.and(escalate);
    }

    // CR 601.2b: an additional cost that is one of several is chosen now, before X (as the C# engine asks it).
    if !def.additional_choice.is_empty() {
        let pick = choose_additional(cx, player, spell, face, def)?;
        extra = pick;
        cost.mana = add_cost(cost.mana, extra.mana);
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

    // CR 601.2b: X, and how many times it is kicked, are announced, and the total cost follows (CR 601.2f).
    let x_symbols = if way == 0 { def.cost.x } else { 0 };
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
            cost.mana.generic += n as u8 * x_symbols;
            item.x = n;
            cx.st().update_obj(spell, |o| o.x = n);
        }
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
                up_to: true, kind: "" };
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
    // CR 601.2f: the total cost, with what increases or reduces it.
    cost.mana = apply_cost_changes(cx, player, Costed::Spell(spell), cost.mana);
    // CR 601.2b, 702.174a: gift is an optional additional cost, announced before the targets.
    let promised = def.gift.is_some()
        && cx.s().opponents(player).any(|p| !cx.s().player(p).lost)
        && matches!(cx.ask_core(player, RequestKind::May { source: spell, purpose: Purpose::PromiseGift }, true)?, Answer::Yes);
    cx.st().stack_push(item);

    // CR 601.2c: targets slot by slot, where they were not announced with the action; a slot counted by X or by the
    // times it was kicked is judged by what was announced.
    let counted = if item.x > 0 { item.x } else { item.kicked as u16 };
    let mut targets: Vec<Target> = targets.to_vec();
    if let Some(modal) = def.modal {
        targets = choose_slots(cx, player, spell, &mode_specs(modal, mask), counted)?;
        set_item_targets(cx, &targets);
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
                set_item_targets(cx, &targets);
                set_item_shares(cx, &shares);
            }
            Divided::Skipped => {}
            // A total that needs a target and has none to give it is not a legal announcement.
            Divided::Impossible => return Err(Stop::Cancel),
        }
    } else if (resolving || !targets_with_cast(def)) && !specs.is_empty() {
        targets = choose_slots(cx, player, spell, specs, counted)?;
        set_item_targets(cx, &targets);
    }

    // CR 702.47c, 601.2c: the targets of spliced text are chosen after the spell's own, in the order the cards were spliced.
    if item.data.iter().any(|&d| d != 0) {
        let mut all = targets.clone();
        all.resize(crate::resolve::expand_specs(cx.s(), player, specs).len().max(all.len()), Target::None);
        for &spliced in item.data.iter().filter(|&&d| d != 0) {
            let specs = cx.card(CardId(spliced as u16 - 1)).spell.map_or(&[][..], |s| s.targets);
            if !specs.is_empty() {
                all.extend(choose_slots(cx, player, spell, specs, counted)?);
            }
        }
        if all.len() != targets.len() {
            targets = all;
            set_item_targets(cx, &targets);
        }
    }

    // CR 601.2h, 702.174a: a promised gift is paid by choosing the opponent, the only one without asking.
    if promised {
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
        let at = cx.s().stack.len() - 1;
        let mut with = cx.s().stack[at];
        with.gift = Some(to);
        cx.st().stack_set(at, with);
    }

    // Mana of any type may pay for it (CR 609.4b).
    if any_type {
        cost.mana = ManaCost { generic: cost.mana.mana_value().min(255) as u8, ..ManaCost::ZERO };
    }

    // CR 601.2f-h: permanents the cost names are chosen, the mana paid, then the rest of the cost; an additional
    // cost's choices come once the mana is paid (CR 601.2h), as the C# engine asks them.
    let chosen = choose_permanents(cx, player, spell, &cost)?;
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
    let helpers = Helpers { with: def.pay_with, waterbend, creatures: false, no_mana: def.keywords.has(Keyword::CantSpendManaToCast), casting: true, reserved: &exiles };
    let (spent, restrictions) = pay_mana_used(cx, player, spell, cost.mana, &helpers)?;
    // CR 106.6: a creature spell paid for with mana that gives it haste has haste once it resolves; a spell paid for with
    // mana that makes it uncounterable can't be countered.
    if restrictions & (1 << ManaRestriction::GainsHaste as u8) != 0 && cx.chars(spell).is(CardType::Creature) {
        cx.st().update_obj(spell, |o| o.marks.gains_haste = true);
    }
    if restrictions & (1 << ManaRestriction::LegendaryUncounterable as u8) != 0 {
        cx.st().update_obj(spell, |o| o.marks.cant_be_countered = true);
    }
    apply_next_spell_effects(cx, player, spell);
    let extra_chosen = choose_permanents(cx, player, spell, &extra)?;
    let beheld = choose_behold(cx, player, spell, &cost)?;
    let extra_beheld = choose_behold(cx, player, spell, &extra)?;
    let blighted = choose_blight(cx, player, spell, &cost)?;
    let extra_blighted = choose_blight(cx, player, spell, &extra)?;
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
    cx.st().update_obj(spell, |o| {
        o.spent = spent;
        o.cast_way = way;
        o.x = item.x;
    });
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
    pay_blight(cx, &cost, blighted);
    pay_blight(cx, &extra, extra_blighted);
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
            cx.st().delayed_push(cantrip_state::Delayed {
                when: DelayedWhen::Step { step: Step::End, on_turn_of: None },
                source: spell,
                controller: player,
                ability: AbilityRef::delayed(face, delayed),
                subject: Target::Object(spell),
                members: cantrip_state::Span::default(),
            });
        }
    }

    // CR 601.2i: it becomes cast.
    let as_cast = cx.chars(spell);
    let x = cx.s().stack.iter().rev().find(|i| i.object == spell).map_or(0, |i| i.x);
    let value = (cx.card(as_cast.card).cost.mana_value() + x as u32).min(255) as u8;
    cx.record(Event::Cast(Cast {
        spell,
        player,
        types: as_cast.types,
        colors: as_cast.colors,
        value,
        from: cast_from,
        way,
    }));
    let nth = cx.history().casts_by(player).count().min(255) as u8;
    cx.emit(Event::SpellCast { spell, player, nth });
    for &t in targets.iter().filter(|&&t| t != Target::None) {
        cx.emit(Event::BecameTarget { target: t, by: spell, controller: player });
    }
    Ok(())
}

/// Whether the resolving effect's controller could cast `card` now (CR 608.2g): it is not a land and every target it
/// needs has a legal choice.
pub fn could_cast_during_resolution(cx: &Cx, card: ObjectId) -> bool {
    !cast_ways_during_resolution(cx, card).is_empty()
}

/// Each way the resolving effect's controller could cast `card` now (CR 608.2g), each face judged alone (CR 709.3a, 715.3a,
/// 712.8a): the card itself unless it is a land or a split card (whose halves are what is cast, CR 709.3), a split card's two
/// halves (an aftermath half only from a graveyard, CR 702.127a), an Adventurer's Adventure, and a modal double-faced card's
/// back face. Fuse is not among them: it applies only in a hand (CR 702.102a).
pub fn cast_ways_during_resolution(cx: &Cx, card: ObjectId) -> Vec<CastWay> {
    let Some(player) = cx.s().stack.last().map(|i| i.controller) else { return Vec::new() };
    if !static_rules::may_cast(cx, player) {
        return Vec::new();
    }
    let obj = cx.s().obj(card);
    if !live(cx.s(), card) || matches!(obj.zone.kind(), ZoneKind::Battlefield | ZoneKind::Stack) {
        return Vec::new();
    }
    let def = cx.card(obj.card);
    let asker = Asker { source: card, controller: player };
    let castable = |face: &'static CardDef| match face.modal {
        Some(modal) => modes_possible(cx, &asker, modal),
        None => face.spell_targets().iter().all(|spec| !spec.required() || !target_candidates(cx, &asker, spec).is_empty()),
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
    if let Some(adventure) = def.adventure
        && castable(adventure)
    {
        ways.push(CastWay { way: WAY_ADVENTURE, face: cx.card_id(adventure), mana_value: adventure.cost.mana_value() });
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
    let (extra, unlimited, _, _) = land_rules(cx, player);
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
    if !static_rules::may_cast(cx, player) {
        return false;
    }
    let obj = cx.s().obj(card);
    if !live(cx.s(), card) || matches!(obj.zone.kind(), ZoneKind::Battlefield | ZoneKind::Stack) {
        return false;
    }
    let Some(back) = cx.card(obj.card).back else { return false };
    if back.has_cost {
        return false;
    }
    let asker = Asker { source: card, controller: player };
    if let Some(modal) = back.modal {
        return modes_possible(cx, &asker, modal);
    }
    back.spell_targets().iter().all(|spec| !spec.required() || !target_candidates(cx, &asker, spec).is_empty())
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

/// Each target slot's targets, asked in slot order (CR 601.2c), each a target of the slot's own (a player may be
/// chosen for two slots). A slot that chose nothing ("up to") keeps its place with `Target::None` among the slots, so
/// every target stays at the index of its slot (CR 601.2c, 115.1), as far as the last one that chose.
fn choose_slots(cx: &mut Cx, player: PlayerId, source: ObjectId, specs: &[TargetSpec], counted: u16) -> R<Vec<Target>> {
    let asker = Asker { source, controller: player };
    let slots = crate::resolve::expand_specs(cx.s(), player, specs);
    let mut targets = Vec::new();
    for (spec, slot_player) in &slots {
        let before = targets.len();
        'slot: {
            let candidates =
                target_candidates_given(cx, &asker, spec, if slots.len() > 1 { &targets } else { &[] }, Target::None, *slot_player);
            if candidates.is_empty() {
                // A required slot with nothing to aim at is not a legal announcement (CR 601.2c): given up whole.
                if spec.required() {
                    return Err(Stop::Cancel);
                }
                break 'slot;
            }
            let (least, most) = spec.bounds(counted);
            if most == 0 {
                break 'slot;
            }
            let count = (most as usize).min(candidates.len()) as u8;
            let ask = RequestKind::ChooseTarget { label: spec.label, slot_player: *slot_player,
                source,
                purpose: Purpose::Targets,
                candidates,
                optional: least == 0,
                least: least.min(count),
                count,
                any_number: false,
            };
            match cx.ask_core(player, ask, true)? {
                Answer::Targets(chosen) => targets.extend(chosen),
                _ => return Err(Stop::Cancel),
            }
        }
        if slots.len() > 1 && spec.max <= 1 && targets.len() == before {
            targets.push(Target::None);
        }
    }
    while targets.last() == Some(&Target::None) {
        targets.pop();
    }
    Ok(targets)
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

/// Records the targets chosen for the item being announced, the top of the stack.
fn set_item_targets(cx: &mut Cx, targets: &[Target]) {
    let at = cx.s().stack.len() - 1;
    let mut with = cx.s().stack[at];
    with.target_count = targets.len() as u8;
    with.targets[..targets.len()].copy_from_slice(targets);
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

/// Asks for a modal spell's or ability's modes (CR 601.2b, 602.2b, 700.2), as the C# engine's `ChooseModesPrompt`:
/// `count` to `at_most` of the eligible modes, with the ones whose own cost could be paid beside `base` as advice.
/// Returns the chosen modes as a bit for each.
fn choose_modes(cx: &mut Cx, player: PlayerId, source: ObjectId, ability: AbilityRef, modal: &ModalDef, base: ManaCost) -> R<u8> {
    let asker = Asker { source, controller: player };
    let eligible = eligible_modes(cx, &asker, modal);
    let costed = modal.modes.iter().any(|m| m.cost.mana_value() > 0);
    let affordable: Vec<u8> = if costed {
        eligible.iter().copied().filter(|&i| can_afford(cx, player, source, &add_cost(base, modal.modes[i as usize].cost))).collect()
    } else {
        Vec::new()
    };
    // An escalate cost the player cannot pay narrows the range to one they can.
    let mut most = modal.max;
    // CR 700.2: the board as the spell is cast may allow more modes than it prints.
    if let Some(up_to) = cx.card(cx.s().obj(source).card).modes_up_to {
        most = most.max(up_to(cx, &asker));
    }
    if modal.escalate.is_some() {
        let payable = |cx: &Cx, n: u8| {
            let first_n = (1u16 << n).wrapping_sub(1) as u8;
            can_afford(cx, player, source, &modal_cost(modal, first_n, base)) && can_pay_other(cx, source, player, &escalate_cost(modal, first_n))
        };
        while most > modal.min.max(1) && !payable(cx, most) {
            most -= 1;
        }
    }
    let ask = RequestKind::ChooseModes {
        source,
        modes: (0..modal.modes.len() as u8).map(|i| Label::Mode(ability, i)).collect(),
        eligible,
        count: modal.min,
        at_most: if most > modal.min { most } else { 0 },
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
    let payable = can_pay_other(cx, spell, player, &optional.cost);
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
        let ask = RequestKind::PayCosts { source: spell, total, remaining: total, options, announcing: true, x: n };
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
        && matches!(cx.ask_core(player, RequestKind::May { source: granter, purpose: Purpose::AlternativeCost }, true)?, Answer::Yes)
    {
        mana = alternative;
    }

    // CR 602.2b, 601.2b: a modal ability's modes first, then X, then the Phyrexian symbols paid with life.
    let mut mask = 0u8;
    if let Some(modal) = def.modal {
        mask = choose_modes(cx, player, source, ability, modal, mana)?;
        item.modes = mask;
    }
    if mana.x > 0 {
        let n = announce(cx, player, source, mana, None, mana.x, 1000)?;
        mana.generic += n as u8 * mana.x;
        item.x = n;
    }
    let life = announce_life(cx, player, source, &mut mana)?;
    mana = apply_cost_changes(cx, player, Costed::Ability(source, ability), mana);
    cx.st().stack_push(item);

    let mut targets: Vec<Target> = targets.to_vec();
    if let Some(modal) = def.modal {
        targets = choose_slots(cx, player, source, &mode_specs(modal, mask), item.x)?;
        set_item_targets(cx, &targets);
    } else if !targets_with_action(def.targets) && !def.targets.is_empty() {
        // CR 601.2c by way of 602.2b: slots that are not announced with the action are chosen now, after X (a slot
        // counted by X, as Candelabra of Tawnos's "untap X target lands", reads the X just announced).
        targets = choose_slots(cx, player, source, def.targets, item.x)?;
        set_item_targets(cx, &targets);
    }

    // Behold and blight are paid when a spell is cast only (CR 701.4a, 701.68a): an activated ability that had them would
    // get the cost for free.
    debug_assert!(def.cost.behold.is_none() && def.cost.blight == 0, "an activated ability's behold or blight is not built");
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
    let counter = choose_counter_kind(cx, player, source, &def.cost)?;
    // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
    if mana.mana_value() > 0 && static_rules::spend_any_color_for(cx, player, source) {
        mana = ManaCost { generic: mana.mana_value().min(255) as u8, ..ManaCost::ZERO };
    }
    pay_mana_with(cx, player, source, mana, &Helpers { waterbend: if def.waterbend { u8::MAX } else { 0 }, creatures: def.tap_creatures_for_mana, ..Helpers::NONE })?;
    if life > 0 {
        ops::change_life(cx, player, -(life as i32));
    }
    pay_permanents(cx, &def.cost, &chosen);
    pay_discards(cx, &discards);
    pay_exiles(cx, &exiles);
    pay_life_and_counters(cx, player, source, &def.cost);
    if let Some(kind) = counter {
        cx.st().update_obj(source, |o| o.counters[kind as usize] = (o.counters[kind as usize] - 1).max(0));
    }
    pay_exert_and_tappers(cx, source, &def.cost, &tappers);
    note_activation(cx, source, index as usize, def);
    if def.cost.exile_self {
        ops::move_to_for(cx, source, Zone::exile(cx.s().obj(source).owner), ZoneReason::Exile);
    }
    if def.cost.tap {
        ops::tap(cx, source);
    }
    if def.cost.discard_self {
        ops::discard(cx, source);
        if def.from == ZoneKind::Hand {
            cx.emit(Event::Cycled { card: source, player });
        }
    }
    if let Some(loyalty) = def.cost.loyalty {
        let turn = cx.s().turn.number;
        cx.st().update_obj(source, |o| {
            o.loyalty_turn = turn;
            let n = &mut o.counters[CounterKind::Loyalty as usize];
            *n = (*n + loyalty as i16).max(0);
        });
    }
    if def.cost.sacrifice_self {
        ops::sacrifice(cx, &[source]);
    }
    if def.cost.return_self {
        ops::move_to_for(cx, source, Zone::hand(cx.s().obj(source).owner), ZoneReason::Other);
    }
    cx.emit(Event::AbilityActivated { source, ability, player });
    for &t in targets.iter().filter(|&&t| t != Target::None) {
        cx.emit(Event::BecameTarget { target: t, by: source, controller: player });
    }
    Ok(())
}

/// The mana a special action costs (CR 116.2): the ability's own, except for the two the rules word as "pay the mana cost of
/// the object": turning a face-down creature card face up costs its mana cost, with X as 0 (CR 701.40b, 107.3b), and
/// unlocking a Room's door the mana cost of that door (CR 709.5e).
fn special_action_cost(cx: &Cx, source: ObjectId, ability: AbilityRef, def: &ActivatedDef) -> ManaCost {
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

/// A special action (CR 116.2) from an ability: its mana cost is paid, and its effect done at once with no stack. The
/// effect asks no questions and has no targets.
fn special_action(cx: &mut Cx, player: PlayerId, source: ObjectId, ability: AbilityRef) -> R<()> {
    let def = activated(cx, ability).expect("activated");
    let mana = special_action_cost(cx, source, ability, def);
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
    // A mana ability is activated (CR 605.3) as any ability is, and the history says so; no ability triggers on it, for
    // the abilities that care about activations are about those that aren't mana abilities.
    cx.record(Event::AbilityActivated { source, ability, player });
    if def.cost.exile_self {
        ops::move_to_for(cx, source, Zone::exile(cx.s().obj(source).owner), ZoneReason::Exile);
    }
    if def.cost.tap {
        ops::tap(cx, source);
    }
    // A mana ability's own mana cost (Chromatic Star's {1}) is paid like any other (CR 605.3a, 601.2g).
    if mana != ManaCost::ZERO {
        // Mana of any color pays for it (CR 609.4b): every symbol is paid as generic mana.
        let mana = if static_rules::spend_any_color_for(cx, player, source) {
            ManaCost { generic: mana.mana_value().min(255) as u8, ..ManaCost::ZERO }
        } else {
            mana
        };
        pay_mana(cx, player, source, mana)?;
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
    if !adds_nothing {
        match def.mana {
            Some(effect) => effect(cx, &asker, color)?,
            None => add_listed(cx, player, def, if colorless { Some(Mana::C) } else { color.map(Color::mana) }),
        }
    }
    if def.cost.tap {
        tapped_for_mana(cx, player, source);
    }
    Ok(())
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

/// A permanent was tapped for mana (CR 605.1b): the fact is recorded, and the triggered mana abilities it triggers resolve at
/// once, as part of the mana ability that triggered them (CR 605.4a): "whenever you tap a Forest for mana, add an additional
/// {G}". They are looked for among the permanents whose cards have one.
fn tapped_for_mana(cx: &mut Cx, player: PlayerId, source: ObjectId) {
    let event = Event::TappedForMana { permanent: source, player };
    cx.record(event);
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
    for id in holders {
        let controller = cx.s().obj(id).controller;
        for ability in cx.chars(id).abilities().collect::<Vec<_>>() {
            let AbilityDef::Triggered(t) = cx.cards().ability(ability) else { continue };
            if !t.mana_ability {
                continue;
            }
            if let Some(fired) = (t.matches)(&*cx, id, &event) {
                let mut item = stack_item(StackKind::Triggered, id, ability, controller, &[]);
                item.subject = fired.subject;
                item.data = fired.data;
                // A mana ability asks nothing (CR 605.1b): there is no one to answer.
                let _ = (t.effect)(cx, &item);
            }
        }
    }
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

/// Whether mana under this restriction may pay for what `source` is (CR 106.6): "can't be spent to cast nonartifact
/// spells" pays an artifact spell, and everything that is not casting a spell; a limit to some spells pays only the casting
/// of such a spell (`casting`).
fn may_spend(cx: &Cx, source: ObjectId, restriction: ManaRestriction, casting: bool) -> bool {
    // Only casting a spell is limited to spells (CR 106.6, 601.2g): paying "unless its controller pays" (CR 118.12) or a
    // copy's cost for the spell on the stack is not casting it.
    let spell = casting && cx.s().obj(source).zone.is(ZoneKind::Stack);
    match restriction {
        ManaRestriction::NotNonartifactSpells => !spell || cx.chars(source).is(CardType::Artifact),
        ManaRestriction::GainsHaste => true,
        ManaRestriction::LegendaryUncounterable => spell && cx.chars(source).supertypes.legendary,
        ManaRestriction::ArtifactSpells => spell && cx.chars(source).is(CardType::Artifact),
        ManaRestriction::InstantOrSorcerySpells => {
            spell && (cx.chars(source).is(CardType::Instant) || cx.chars(source).is(CardType::Sorcery))
        }
    }
}

/// The restricted mana `player` has that may pay for `source`, with its restriction.
fn restricted_for(cx: &Cx, player: PlayerId, source: ObjectId, casting: bool) -> Vec<(ManaRestriction, ManaPool)> {
    let p = cx.s().player(player);
    let mut out = Vec::new();
    if p.restricted.iter().all(|r| r.is_empty()) {
        return out;
    }
    for restriction in ManaRestriction::ALL {
        let pool = p.restricted[restriction as usize];
        if !pool.is_empty() && may_spend(cx, source, restriction, casting) {
            out.push((restriction, pool));
        }
    }
    out
}

/// The pool a cost for `source` may be paid from: unrestricted mana and the restricted mana that may pay for it.
fn spendable(cx: &Cx, player: PlayerId, source: ObjectId, casting: bool) -> ManaPool {
    let mut pool = cx.s().player(player).pool;
    for (_, restricted) in restricted_for(cx, player, source, casting) {
        for m in Mana::ALL {
            pool.0[m as usize] += restricted.0[m as usize];
        }
    }
    pool
}

/// Takes `spent` out of the pool, restricted mana first (it is what the C# engine prefers spending). Which restrictions
/// the mana taken carried, as a bitmask by `ManaRestriction`.
fn take_mana(cx: &mut Cx, player: PlayerId, source: ObjectId, casting: bool, spent: &[u8; 6]) -> u8 {
    let restricted = restricted_for(cx, player, source, casting);
    let mut used = 0u8;
    cx.st().update_player(player, |p| {
        for mana in Mana::ALL {
            let mut need = spent[mana as usize];
            for (restriction, _) in &restricted {
                let have = &mut p.restricted[*restriction as usize].0[mana as usize];
                let take = (*have).min(need);
                if take > 0 {
                    used |= 1 << *restriction as u8;
                }
                *have -= take;
                need -= take;
            }
            p.spend_mana(mana, need);
        }
    });
    used
}

fn mana_options_while_paying(cx: &Cx, player: PlayerId) -> Vec<ActionOption> {
    let mut options = Vec::new();
    mana_ability_options(cx, player, &mut options);
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
    /// The mana pays for casting `source`, a spell (CR 601.2g): mana limited to some spells (CR 106.6) may be spent.
    pub casting: bool,
    /// Cards another part of the cost was already chosen to exile (escape's), which delve may not also exile.
    pub reserved: &'a [ObjectId],
}

impl Helpers<'static> {
    pub const NONE: Helpers<'static> = Helpers { with: PayWith::NONE, waterbend: 0, creatures: false, no_mana: false, casting: false, reserved: &[] };
}

impl Helpers<'_> {
    /// Whether anything besides mana may pay.
    pub fn any(&self) -> bool {
        self.with.any() || self.waterbend > 0 || self.creatures
    }
}

/// Pays a mana cost (CR 601.2g-h) with nothing but mana.
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

/// `pay_mana_with`, and which restrictions the mana spent carried (a bitmask by `ManaRestriction`, CR 106.6).
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
        let pool = if helpers.no_mana { ManaPool::default() } else { spendable(cx, player, source, helpers.casting) };
        let candidates =
            if helpers.any() { pay_candidates(cx, player, source, helpers, &cost, &due, &convoking, &delved, bent) } else { Vec::new() };
        let mut decided = ways(&pool, &due);
        // Ordinary mana and restricted mana of a kind are different units (CR 106.6): spending only some of that kind
        // is the player's choice, even when the pool pays the cost one way.
        if let Ways::One(spent) = decided {
            let player_state = cx.s().player(player);
            let restricted = restricted_for(cx, player, source, helpers.casting);
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
            *used |= take_mana(cx, player, source, helpers.casting, &spent);
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
        let mut options = if paying { Vec::new() } else { mana_options_while_paying(cx, player) };
        if matches!(decided, Ways::One(_)) {
            // The pool's one way of paying, which the player may take or pay some other way instead.
            options.push(ActionOption::simple(ActionKind::ConfirmCasting));
        } else if !helpers.no_mana {
            let restricted = restricted_for(cx, player, source, helpers.casting);
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
        let ask = RequestKind::PayCosts { source, total: shown, remaining: due, options, announcing: false, x: 0 };
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
                        Some(r) => {
                            used |= 1 << r as u8;
                            cx.st().update_player(player, |p| p.restricted[r as usize].0[mana as usize] -= 1)
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

/// "Unless [payer] pays [cost]" (CR 118.12): the payer may activate mana abilities, then pay once the pool covers the
/// cost, or decline.
pub fn unless_pays(cx: &mut Cx, payer: PlayerId, source: ObjectId, subject: ObjectId, total: ManaCost) -> R<bool> {
    loop {
        let pool = spendable(cx, payer, source, false);
        let mut options = mana_options_while_paying(cx, payer);
        if !matches!(ways(&pool, &total), Ways::None) {
            options.push(ActionOption::simple(ActionKind::PayCost));
        }
        options.push(ActionOption::simple(ActionKind::DeclineCost));
        let answer = cx.ask_core(payer, RequestKind::UnlessPays { source, subject, total, options }, false)?;
        let Answer::Act(action) = answer else { return Ok(false) };
        match action.kind {
            ActionKind::ActivateAbility => {
                let from = action.source.expect("source");
                activate_mana(cx, payer, from, action.index, action.color, action.mana == Some(Mana::C), &action.targets)?;
            }
            ActionKind::PayCost => {
                pay_mana(cx, payer, source, total)?;
                return Ok(true);
            }
            _ => return Ok(false),
        }
    }
}

/// Whether an object is somewhere it can be found by its id: still the same object.
#[allow(dead_code)]
fn is_live(cx: &Cx, id: ObjectId) -> bool {
    live(cx.s(), id)
}
