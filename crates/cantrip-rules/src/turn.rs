//! The turn (CR 500-514): steps and their turn-based actions, combat, cleanup, and the priority round of each step.

use cantrip_api::*;
use cantrip_state::{Frame, FrameKind};

use crate::layers::{self, live, summoning_sick};
use crate::{Cx, Flow, Game, R, Stop, actions, ops};

/// What a card that may begin the game on the battlefield asks (CR 103.6a): the Leylines, Gemstone Caverns.
const BEGIN_ON_BATTLEFIELD: Question = Question::new(
    "BeginOnBattlefield",
    "Begin the game with {source} on the battlefield?",
    "If you don't, it stays in your opening hand",
    "Put it onto the battlefield",
    "Keep it in hand",
);

/// Shuffles a zone with the state's own generator (Fisher-Yates).
pub fn shuffle(state: &mut GameState, zone: Zone) {
    let mut cards: Vec<ObjectId> = state.zone(zone).to_vec();
    for i in (1..cards.len()).rev() {
        let j = state.random(i as u32 + 1) as usize;
        cards.swap(i, j);
    }
    for &id in &cards {
        state.zone_remove(zone, id);
    }
    for id in cards {
        state.zone_push(zone, id);
    }
}

/// Moves the game on to its next step, or its first turn, and puts that step's work on the agenda.
pub fn next_step(game: &mut Game) {
    let s = &mut game.state;
    let turn = s.turn;
    let ending = turn.step();
    // The combat phase ends with its end of combat step, or with whatever step of it an effect ended the turn in
    // (CR 724.1d).
    let combat_ends = ending == Step::EndOfCombat
        || (turn.ending && matches!(ending, Step::BeginCombat | Step::DeclareAttackers | Step::DeclareBlockers | Step::CombatDamage));

    // Mana empties as each step ends (CR 500.4), except the mana that lasts until the end of combat, which is lost as
    // the combat phase ends (CR 106.4, 500.5a).
    if turn.number > 0 {
        for p in s.seats().collect::<Vec<_>>() {
            if !s.player(p).pool.is_empty() || !s.player(p).lasting.is_empty() {
                s.update_player(p, |pl| {
                    pl.pool = if combat_ends { ManaPool::default() } else { pl.lasting_mana() };
                    pl.lasting = pl.pool;
                });
            }
            s.empty_restricted_mana(p);
        }
    }
    // CR 724.1d: a turn ended during combat removes every creature and planeswalker from combat as the step ends, as
    // the end of the combat phase does.
    if turn.number > 0 && combat_ends {
        s.set_attackers(Vec::new());
        s.set_blocks(Vec::new());
        s.update_turn(|t| {
            t.attacked = false;
            t.damage_round = 0;
        });
    }

    let (number, active, step) = if turn.number == 0 {
        (1, turn.starting, Step::Untap)
    } else if turn.ending || (ending == Step::Cleanup && turn.cleanup_again) {
        // CR 724.1d: the turn ended, so the game skips straight to the cleanup step, a new one if it was ended during
        // one; and a cleanup step in which players had priority is followed by another (CR 514.3a).
        (turn.number, turn.active, Step::Cleanup)
    } else {
        match ending.next() {
            // With no attackers declared, the declare blockers and combat damage steps are skipped (CR 508.8): an
            // attacker removed from combat since does not count.
            Some(Step::DeclareBlockers) if !turn.attacked => (turn.number, turn.active, Step::EndOfCombat),
            // After a first-strike damage step comes the regular one (CR 510.4).
            Some(Step::EndOfCombat) if ending == Step::CombatDamage && turn.damage_round == 1 => {
                (turn.number, turn.active, Step::CombatDamage)
            }
            // An additional combat phase follows the one that ends (CR 500.8): a whole new one, from its beginning.
            Some(Step::PostcombatMain) if ending == Step::EndOfCombat && turn.extra_combats > 0 => {
                (turn.number, turn.active, Step::BeginCombat)
            }
            Some(next) => (turn.number, turn.active, next),
            None => (turn.number + 1, next_turn_taker(s, turn.active), Step::Untap),
        }
    };
    let previous = turn.active;
    s.update_turn(|t| {
        if number != turn.number {
            t.previous = previous;
        }
        t.number = number;
        t.active = active;
        if number != turn.number {
            t.extra_combats = 0;
        } else if ending == Step::EndOfCombat && step == Step::BeginCombat {
            t.extra_combats -= 1;
        }
        t.set_step(step);
        t.priority = active;
        t.passes = 0;
        t.ending = false;
        t.cleanup_again = false;
    });
    s.frame_push(Frame::new(FrameKind::BeginStep));
}

/// Who takes the next turn: the most recently created extra turn first (CR 500.7), skipping a player who has left the
/// game (CR 800.4k), else the next player in turn order.
fn next_turn_taker(s: &mut GameState, after: PlayerId) -> PlayerId {
    let mut queue = s.extra_turns.clone();
    let mut taker = None;
    while let Some(p) = queue.pop() {
        if !s.player(p).lost {
            taker = Some(p);
            break;
        }
    }
    if queue.len() != s.extra_turns.len() {
        s.set_extra_turns(queue);
    }
    let rotation = s.turn.rotation;
    match taker {
        // An extra turn: the rotation stays where it was, at the player whose ordinary turn this one follows.
        Some(p) => {
            if rotation.is_none() {
                s.update_turn(|t| t.rotation = Some(after));
            }
            p
        }
        // The rotation goes on from the last ordinary turn, not from an extra turn taken since (CR 500.7).
        None => {
            if rotation.is_some() {
                s.update_turn(|t| t.rotation = None);
            }
            next_in_game(s, rotation.unwrap_or(after))
        }
    }
}

/// The start of the game (CR 103): who goes first, the opening hands, mulligans.
pub fn opening(cx: &mut Cx) -> R<Flow> {
    // A scenario's fixed deal has dealt and kept the hands already, and goes straight to the opening-hand actions.
    let dealt = cx.s().seats().all(|p| cx.s().player(p).kept);
    if !dealt {
        let chooser = cx.s().turn.starting;
        let candidates: Vec<PlayerId> = cx.s().seats().collect();
        if let Answer::Player(first) =
            cx.ask_core(chooser, RequestKind::ChooseStartingPlayer { candidates }, false)?
        {
            cx.st().update_turn(|t| {
                t.starting = first;
                t.active = first;
                t.priority = first;
            });
        }
    }
    let order: Vec<PlayerId> = cx.s().apnap().collect();
    if !dealt {
        for &p in &order {
            for _ in 0..7 {
                ops::draw(cx, p);
            }
        }
    }

    // The London mulligan (CR 103.5): undecided players decide in turn order, those who mulligan redraw together,
    // and the round repeats.
    loop {
        let undecided: Vec<PlayerId> = order.iter().copied().filter(|&p| !cx.s().player(p).kept).collect();
        if undecided.is_empty() {
            break;
        }
        let mut redraw = Vec::new();
        for p in undecided {
            let mulligans = cx.s().player(p).mulligans;
            if mulligans >= 7 {
                cx.st().update_player(p, |pl| pl.kept = true);
                continue;
            }
            match cx.ask_core(p, RequestKind::Mulligan { mulligans }, false)? {
                Answer::Yes => redraw.push(p),
                _ => {
                    cx.st().update_player(p, |pl| pl.kept = true);
                    if mulligans > 0 {
                        let hand = cx.s().zone(Zone::hand(p)).to_vec();
                        let answer = cx.ask_core(
                            p,
                            RequestKind::ChooseCards {
                                source: hand[0],
                                purpose: Purpose::Discard,
                                cards: hand.clone(),
                                eligible: hand,
                                count: mulligans,
                                up_to: false, kind: "", total: None },
                            false,
                        )?;
                        if let Answer::Cards(bottom) = answer {
                            for card in bottom {
                                let library = Zone::library(p);
                                let new = ops::move_to(cx, card, library);
                                // To the bottom, not the top.
                                cx.st().zone_remove(library, new);
                                cx.st().zone_push(library, new);
                            }
                        }
                    }
                }
            }
        }
        for p in redraw {
            for id in cx.s().zone(Zone::hand(p)).to_vec() {
                ops::move_to(cx, id, Zone::library(p));
            }
            shuffle(cx.st(), Zone::library(p));
            for _ in 0..7 {
                ops::draw(cx, p);
            }
            cx.st().update_player(p, |pl| pl.mulligans += 1);
        }
    }

    // CR 103.6a: with the hands kept, each player in turn order, the starting player first, may begin the game with cards
    // of their opening hand on the battlefield.
    for &p in &order {
        for card in cx.s().zone(Zone::hand(p)).to_vec() {
            let Some(action) = cx.card(cx.s().obj(card).card).opening_hand else { continue };
            if !(action.condition)(cx, &Asker { source: card, controller: p }) {
                continue;
            }
            if !matches!(cx.ask_core(p, RequestKind::May { source: card, purpose: Purpose::Ask(&BEGIN_ON_BATTLEFIELD) }, false)?, Answer::Yes) {
                continue;
            }
            let with: Vec<(CounterKind, i16)> = action.counter.map(|(k, n)| (k, n as i16)).into_iter().collect();
            ops::put_onto_battlefield_with(cx, &[(card, p, false)], ZoneReason::Other, &with)?;
            let hand = cx.s().zone(Zone::hand(p)).to_vec();
            if action.exile_from_hand > 0 && !hand.is_empty() {
                let count = action.exile_from_hand.min(hand.len() as u8);
                let ask = RequestKind::ChooseCards { source: card, purpose: Purpose::Exile, cards: hand.clone(), eligible: hand, count, up_to: false, kind: "", total: None };
                if let Answer::Cards(cards) = cx.ask_core(p, ask, false)? {
                    for c in cards {
                        ops::move_to_for(cx, c, Zone::exile(p), ZoneReason::Exile);
                    }
                }
            }
        }
    }
    Ok(Flow::Done)
}

fn push_priority(cx: &mut Cx) {
    cx.st().frame_push(Frame::new(FrameKind::Priority));
}

/// The step's turn-based actions (CR 500.2), then its priority round.
pub fn begin_step(cx: &mut Cx) -> R<Flow> {
    let turn = cx.s().turn;
    let active = turn.active;
    if turn.step() != Step::Untap {
        crate::settle::step_began(cx, turn.step());
    }
    match turn.step() {
        Step::Untap => {
            // CR 614.10, 500.11: a turn that would begin may be skipped instead, and then never begins.
            if skip_turn(cx, active)? {
                // The next player's turn takes its place, under this one's number: as if cleanup had just ended it.
                cx.st().update_turn(|t| {
                    t.number -= 1;
                    t.set_step(Step::Cleanup);
                });
                return Ok(Flow::Done);
            }
            // CR 502: the turn begins; the active player's permanents untap. No one gets priority.
            let number = turn.number;
            cx.st().update_player(active, |p| {
                p.turn_began = number;
                p.turns_taken += 1;
                p.lands_played = 0;
            });
            // CR 502.2: the untap step's second turn-based action reads what the previous turn's active player cast,
            // before the counts are cleared for this turn.
            crate::day_night::check_untap_step(cx);
            // "Until your next turn" ends as that turn begins (CR 611.2b).
            if cx.s().effects.iter().any(|e| e.duration == Duration::UntilNextTurnOf(active)) {
                cx.st().effects_retain(|e| e.duration != Duration::UntilNextTurnOf(active));
            }
            // The same for a delayed ability that lasts until then (CR 603.7b).
            for at in (0..cx.s().delayed.len()).rev() {
                if cx.s().delayed[at].when == DelayedWhen::UntilNextTurnOf(active) {
                    cx.st().delayed_remove(at);
                }
            }
            untap_step(cx, active);
            return Ok(Flow::Done);
        }
        Step::Draw => {
            // CR 103.8a: the player who goes first skips the draw of their first turn (in a two-player game).
            let skip = turn.number == 1 && active == turn.starting && cx.s().players.len() == 2;
            if !skip {
                ops::draw_cards(cx, active, 1);
            }
        }
        Step::PrecombatMain => ops::add_lore_counters(cx),
        Step::DeclareAttackers => declare_attackers(cx)?,
        Step::DeclareBlockers => declare_blockers(cx)?,
        Step::CombatDamage => combat_damage(cx)?,
        Step::Cleanup => {
            cleanup(cx)?;
            // CR 514.3a: the state-based actions due are performed; if any were, or a triggered ability is waiting to be
            // put on the stack (one that triggered during the cleanup step, or while an effect ended the turn, CR
            // 724.1f), players get priority, and another cleanup step follows this one.
            let acted = crate::settle::state_based_actions_alone(cx)?;
            if acted || !cx.s().triggers.is_empty() {
                cx.st().update_turn(|t| t.cleanup_again = true);
                push_priority(cx);
            }
            return Ok(Flow::Done);
        }
        _ => {}
    }
    push_priority(cx);
    Ok(Flow::Done)
}

/// What the effect that makes a player skip their next turn does, for the player who orders it with an optional skip of the
/// same turn (CR 616.1).
const SKIP_NEXT_TURN_HINT: &str = "you skip this turn";

/// Whether the player skips the turn about to begin (CR 614.10, 500.11). Two kinds of replacement effect skip it:
///
/// - a resolved spell or ability's "[that player] skips their next N turns", one `EffectKind::SkipNextTurn` for each turn
///   to skip, which must be applied: the oldest is used up by this turn, the rest wait for the turns after it
///   (CR 614.10a);
/// - a permanent's "if you would begin your turn while ..., you may skip that turn instead. If you do, ..." (Time
///   Vault, `CardDef::turn_begin`), which the player may decline. Each that applies is offered in turn, and the first
///   accepted skips the turn and does what it says it does then.
///
/// With both, the player chooses which applies (CR 616.1): an optional skip chosen first is taken (declining it would
/// leave the turn to the effect that must skip it), and the effect that must skip a turn then still waits for the next
/// turn, as it skipped none (CR 614.10a). Nothing is asked when nothing applies.
fn skip_turn(cx: &mut Cx, player: PlayerId) -> R<bool> {
    let pending = cx.s().effects.iter().position(|e| e.kind == EffectKind::SkipNextTurn(player));
    let mut optional: Vec<(ObjectId, &'static TurnBeginRule)> = Vec::new();
    for id in cx.s().battlefield().to_vec() {
        let obj = cx.s().obj(id);
        if obj.controller != player {
            continue;
        }
        let card = obj.copy.map_or(obj.card, |c| c.card);
        let asker = Asker { source: id, controller: player };
        // A scan once a turn of what is on the battlefield, for a card that has such a rule.
        if !cx.card(card).turn_begin.is_empty() && cx.s().lost_all_abilities(id) {
            continue;
        }
        for rule in cx.card(card).turn_begin {
            if (rule.applies)(&*cx, &asker, player) {
                optional.push((id, rule));
            }
        }
    }
    let Some(at) = pending else {
        for (id, rule) in optional {
            if cx.ask_core(player, RequestKind::May { source: id, purpose: Purpose::Ask(rule.question) }, false)? == Answer::Yes {
                (rule.on_skip)(cx, &Asker { source: id, controller: player })?;
                return Ok(true);
            }
        }
        return Ok(false);
    };
    if !optional.is_empty() {
        let mut sources = vec![cx.s().effects[at].source];
        let mut hints = vec![SKIP_NEXT_TURN_HINT];
        for &(id, rule) in &optional {
            sources.push(id);
            hints.push(rule.hint);
        }
        let ask = RequestKind::ChooseReplacement { subject: None, life: None, sources, hints, damage: None, turn: true };
        if let Answer::Index(i) = cx.ask_core(player, ask, false)?
            && i > 0
            && let Some(&(id, rule)) = optional.get(i as usize - 1)
        {
            (rule.on_skip)(cx, &Asker { source: id, controller: player })?;
            return Ok(true);
        }
    }
    let keep: Vec<bool> = (0..cx.s().effects.len()).map(|i| i != at).collect();
    cx.st().effects_keep(&keep);
    Ok(true)
}

/// CR 502.3: the active player's permanents untap, all at once. Except an exerted one, for the untap step it sits out
/// (CR 701.43a), and one that doesn't untap during its controller's untap step (CR 502.3); and one with a stun counter
/// has a counter removed from it instead (CR 122.1d).
fn untap_step(cx: &mut Cx, active: PlayerId) {
    for id in cx.s().battlefield().to_vec() {
        let obj = cx.s().obj(id);
        if obj.controller != active || !obj.tapped {
            continue;
        }
        if obj.marks.exerted_by == Some(active) || cx.chars(id).has(Keyword::DoesNotUntap) {
            continue;
        }
        ops::untap(cx, id);
    }
    // That untap step has now happened for everything its player exerted, whoever controls it now (CR 701.43a).
    for id in cx.s().battlefield().to_vec() {
        if cx.s().obj(id).marks.exerted_by == Some(active) {
            cx.st().update_obj(id, |o| o.marks.exerted_by = None);
        }
    }
}

/// The battles the active player may attack (CR 310.9b, 508.1b): those whose protector is one of the defending
/// players. A Siege its attacker controls is one of them (CR 310.12a); one the attacker protects never is.
fn attackable_battles(cx: &Cx, defending: &[PlayerId]) -> Vec<ObjectId> {
    if !cx.s().battlefield().iter().any(|&id| cx.s().obj(id).marks.protector.is_some()) {
        return Vec::new();
    }
    cx.s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            cx.s().obj(id).marks.protector.is_some_and(|p| defending.contains(&p)) && {
                let c = cx.chars(id);
                c.is(CardType::Battle) && !c.has(Keyword::CantBeAttacked)
            }
        })
        .collect()
}

/// The planeswalkers the active player may attack (CR 508.1b): those a defending player controls, except one that
/// can't be attacked (CR 508.1c).
fn attackable_planeswalkers(cx: &Cx, defending: &[PlayerId]) -> Vec<ObjectId> {
    cx.s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| defending.contains(&cx.s().obj(id).controller) && {
            let c = cx.chars(id);
            c.is(CardType::Planeswalker) && !c.has(Keyword::CantBeAttacked)
        })
        .collect()
}

fn defending_player(cx: &Cx, attack: Target) -> Option<PlayerId> {
    defending_player_of(cx.s(), attack)
}

fn declare_attackers(cx: &mut Cx) -> R<()> {
    let active = cx.s().turn.active;
    let eligible: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            let chars = cx.chars(id);
            obj.controller == active
                && chars.is(CardType::Creature)
                && !chars.is(CardType::Battle)
                && !obj.tapped
                // CR 508.1c: an effect saying it can't attack.
                && !cx.s().effects.iter().any(|e| e.affected == id && e.kind == EffectKind::CantAttack)
                && !summoning_sick(cx.cards(), cx.s(), &cx.g.log, id)
                // CR 702.3b: defender, unless an effect or a static ability lets it attack as though it didn't have it
                // (CR 609.4), which lifts no other restriction: asked last, of a creature that could otherwise attack.
                && (!chars.has(Keyword::Defender) || crate::static_rules::may_attack_despite_defender(&*cx, cx.cards(), id))
        })
        .collect();
    if eligible.is_empty() {
        return Ok(());
    }
    let players: Vec<PlayerId> = cx.s().opponents(active).collect();
    // CR 310.9b, 508.1b: every battle a defending player protects, whoever controls it. Looked for only when a battle
    // is on the battlefield.
    let battles = attackable_battles(cx, &players);
    let planeswalkers = attackable_planeswalkers(cx, &players);
    // CR 508.1d, 508.1h: what each attack costs, where a static ability makes attacking cost something.
    let targets: Vec<Target> = players
        .iter()
        .map(|&p| Target::Player(p))
        .chain(planeswalkers.iter().chain(&battles).map(|&id| Target::Object(id)))
        .collect();
    let costs = crate::static_rules::attack_costs(cx, &eligible, &targets);
    // CR 508.1d: a cost to attack is never a requirement, so a creature that attacks each combat if able need not attack
    // where every attack open to it costs something.
    let requirements: Vec<ObjectId> = eligible
        .iter()
        .copied()
        .filter(|&id| cx.chars(id).must_attack && targets.iter().any(|&t| AttackCost::free(&costs, id, t)))
        .collect();
    let can_pay = if costs.is_empty() { 0 } else { actions::available(cx, active).most_unrestricted() };
    let kind = RequestKind::DeclareAttackers { eligible, players, planeswalkers, battles, requirements, costs, can_pay };
    let attacks = loop {
        // A declaration whose costs are not paid is taken back (CR 508.1, 733): the game returns to the moment before it,
        // the mana abilities activated while paying undone, and the attackers are declared again.
        let mark = cx.s().mark();
        let log_mark = cx.g.log.len();
        let answer = cx.ask_core(active, kind.clone(), false)?;
        let Answer::Attack(attacks) = answer else { return Ok(()) };
        // CR 508.1f: the chosen creatures become tapped, which is no cost, before any cost is paid (CR 508.1g-j).
        for &(id, _) in &attacks {
            if !cx.chars(id).has(Keyword::Vigilance) {
                ops::tap(cx, id);
            }
        }
        let RequestKind::DeclareAttackers { costs, .. } = &kind else { unreachable!() };
        // CR 508.1h: the total cost to attack, locked in now; mana abilities activated while paying it (CR 508.1i) change
        // nothing of it. CR 508.1j: it is paid all at once.
        let total = AttackCost::total(costs, &attacks);
        if !total.is_zero() {
            let paying: Vec<ObjectId> = attacks.iter().filter(|&&(a, t)| !AttackCost::free(costs, a, t)).map(|a| a.0).collect();
            let source = costs
                .iter()
                .find(|c| attacks.contains(&(c.attacker, c.target)))
                .map(|c| c.source)
                .expect("a cost to attack has a source");
            let helpers = actions::Helpers { attackers: &paying, ..actions::Helpers::NONE };
            match actions::pay_mana_with(cx, active, source, total, &helpers) {
                Ok(_) => {}
                Err(Stop::Cancel) => {
                    cx.st().revert(mark);
                    cx.g.log.truncate(log_mark);
                    continue;
                }
                Err(stop) => return Err(stop),
            }
        }
        break attacks;
    };
    let mut attackers = Vec::new();
    for (id, target) in attacks {
        attackers.push(Attacker { id, target, blocked: false });
    }
    let attacked = !attackers.is_empty();
    cx.st().update_turn(|t| t.attacked = attacked);
    cx.st().set_attackers(attackers.clone());
    // One batch (CR 603.2c): "whenever one or more creatures attack" triggers once; "whenever you attack" after the
    // creatures' own abilities (CR 508.1m).
    let mut events: Vec<Event> = attackers.iter().map(|a| Event::Attacks { attacker: a.id, defender: a.target }).collect();
    if attacked {
        events.push(Event::AttackersDeclared { player: active, count: attackers.len() as u8 });
    }
    cx.emit_batch(&events, None);
    Ok(())
}

/// Whether `blocker` may block `attacker` (CR 509.1b): a creature that can't block (decayed's too, CR 702.147a), one that
/// can block only creatures with flying, flying (CR 702.9b), shadow (CR 702.28b), protection (CR 702.16f). Menace (CR 702.110b) is about the
/// whole declaration, not a pair.
fn can_block(cx: &Cx, blocker: ObjectId, attacker: ObjectId, rule_sources: &[ObjectId]) -> bool {
    let a = cx.chars(attacker);
    let b = cx.chars(blocker);
    if a.has(Keyword::CantBeBlocked) || b.has(Keyword::CantBlock) || b.has(Keyword::Decayed) || (b.has(Keyword::CanBlockOnlyCreaturesWithFlying) && !a.has(Keyword::Flying)) {
        return false;
    }
    if a.has(Keyword::Flying) && !b.has(Keyword::Flying) && !b.has(Keyword::Reach) {
        return false;
    }
    if a.has(Keyword::Shadow) != b.has(Keyword::Shadow) {
        return false;
    }
    // "Can't be blocked by creatures with power 2 or less" (CR 509.1b): a creature with no power can't block it either.
    if a.has(Keyword::CantBeBlockedByPowerTwoOrLess) && b.power.is_none_or(|p| p <= 2) {
        return false;
    }
    // CR 509.1b: a static ability that limits who may block whom, of the blocker, of the attacker, or of another permanent
    // ("creatures you control with power 2 or less can't be blocked by ...", Delney, Streetwise Lookout).
    for &id in rule_sources {
        let obj = cx.s().obj(id);
        let card = obj.copy.map_or(obj.card, |c| c.card);
        if !cx.card(card).block_rules.iter().all(|rule| rule(cx, id, blocker, attacker)) {
            return false;
        }
    }
    !a.protected_from_by(&b, cx.s().obj(blocker).controller)
}

/// "Can't block" (CR 509.1b): from an effect on this creature, or the turn's rule that no creature can.
fn cant_block(cx: &Cx, id: ObjectId) -> bool {
    let s = cx.s();
    s.turn.no_blocks_turn == s.turn.number
        || s.effects.iter().any(|e| match e.kind {
            EffectKind::CantBlock => e.affected == id,
            // "Creatures target player controls can't block this turn": a rule about the player, reaching later creatures too.
            EffectKind::CreaturesCantBlock(p) => s.obj(id).controller == p,
            _ => false,
        })
}

fn declare_blockers(cx: &mut Cx) -> R<()> {
    let attackers: Vec<ObjectId> = cx.s().attackers.iter().map(|a| a.id).collect();
    if attackers.is_empty() {
        return Ok(());
    }
    let Some(defender) = defending_player(cx, cx.s().attackers[0].target) else { return Ok(()) };
    let eligible: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| cx.s().obj(id).controller == defender && cx.chars(id).is(CardType::Creature) && !cx.s().obj(id).tapped)
        .filter(|&id| !cant_block(cx, id))
        .collect();
    // The permanents whose static abilities limit who may block whom (CR 509.1b), and which have not lost them (CR 613.1f).
    let rule_sources: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            cx.cards().block_rules[obj.copy.map_or(obj.card, |c| c.card).0 as usize] && !cx.s().lost_all_abilities(id)
        })
        .collect();
    let mut allowed = Vec::new();
    for &b in &eligible {
        for &a in &attackers {
            if can_block(cx, b, a, &rule_sources) {
                allowed.push((b, a));
            }
        }
    }
    // Menace (CR 702.110b) and "except by three or more creatures" (CR 509.1b): an attacker that can't be blocked by
    // fewer than so many creatures is offered only if that many could.
    let menace: Vec<ObjectId> = attackers
        .iter()
        .copied()
        .filter(|&a| allowed.iter().filter(|p| p.1 == a).count() < cx.chars(a).minimum_blockers() as usize)
        .collect();
    allowed.retain(|p| !menace.contains(&p.1));
    if !allowed.is_empty() {
        let answer = cx.ask_core(defender, RequestKind::DeclareBlockers { eligible, attackers, allowed }, false)?;
        if let Answer::Block(blocks) = answer {
            let mut list = Vec::new();
            let mut attackers = cx.s().attackers.clone();
            for (blocker, attacker) in blocks {
                list.push(Block { blocker, attacker });
                if let Some(a) = attackers.iter_mut().find(|a| a.id == attacker) {
                    a.blocked = true;
                }
            }
            cx.st().set_attackers(attackers);
            // CR 509.1g: the blockers are declared together, each one a "blocks" event for abilities that watch it.
            let blocks: Vec<Event> = list.iter().map(|b| Event::Blocks { blocker: b.blocker, attacker: b.attacker }).collect();
            cx.st().set_blocks(list);
            if !blocks.is_empty() {
                cx.emit_batch(&blocks, None);
            }
        }
    }
    // CR 509.1h: the attackers nothing blocked are unblocked, announced together.
    let unblocked: Vec<Event> = cx
        .s()
        .attackers
        .iter()
        .filter(|a| !a.blocked)
        .map(|a| Event::Unblocked { attacker: a.id, defender: a.target })
        .collect();
    if !unblocked.is_empty() {
        cx.emit_batch(&unblocked, None);
    }
    Ok(())
}

/// Combat damage (CR 510): every attacker and blocker deals its damage at once. A blocked attacker divides its
/// damage among its blockers in order, lethal to each before the next (CR 510.1c's default).
/// Whether a creature deals combat damage in this damage step: in a first-strike step only creatures with first
/// strike or double strike; in the regular step those without first strike, and those with double strike again.
fn deals_now(cx: &Cx, id: ObjectId, first_strike_step: bool) -> bool {
    let c = cx.chars(id);
    let first = c.has(Keyword::FirstStrike) || c.has(Keyword::DoubleStrike);
    if first_strike_step { first } else { !c.has(Keyword::FirstStrike) || c.has(Keyword::DoubleStrike) }
}

fn combat_damage(cx: &mut Cx) -> R<()> {
    // CR 510.4: with a first or double striker in combat, there is a first-strike damage step before the regular one.
    let in_combat: Vec<ObjectId> =
        cx.s().attackers.iter().map(|a| a.id).chain(cx.s().blocks.iter().map(|b| b.blocker)).collect();
    let round = cx.s().turn.damage_round;
    let first_strike_step = round == 0
        && in_combat.iter().any(|&id| {
            live(cx.s(), id) && {
                let c = cx.chars(id);
                c.has(Keyword::FirstStrike) || c.has(Keyword::DoubleStrike)
            }
        });
    cx.st().update_turn(|t| t.damage_round = if first_strike_step { 1 } else { 2 });

    // CR 510.1c-d: an attacker blocked by more than one creature, or with trample and more than lethal, divides its
    // damage as its controller chooses.
    let mut divisions = Vec::new();
    for attacker in cx.s().attackers.clone() {
        if live(cx.s(), attacker.id) && attacker.blocked && deals_now(cx, attacker.id, first_strike_step)
            && let Some(d) = division(cx, &attacker)
        {
            divisions.push(d);
        }
    }
    let mut chosen: Vec<(ObjectId, Option<ObjectId>, u16)> = Vec::new();
    if !divisions.is_empty() {
        let active = cx.s().turn.active;
        if let Answer::Shares(shares) = cx.ask_core(active, RequestKind::AssignCombatDamage { divisions }, false)? {
            chosen = shares;
        }
    }

    let mut assignments: Vec<(ObjectId, Target, u16)> = Vec::new();
    for attacker in cx.s().attackers.clone() {
        if !live(cx.s(), attacker.id) || !deals_now(cx, attacker.id, first_strike_step) {
            continue;
        }
        if chosen.iter().any(|s| s.0 == attacker.id) {
            for &(source, to, amount) in chosen.iter().filter(|s| s.0 == attacker.id) {
                let target = match to {
                    Some(creature) => Target::Object(creature),
                    None => attacker.target,
                };
                assignments.push((source, target, amount));
            }
            continue;
        }
        let power = crate::static_rules::combat_damage_of(cx, attacker.id);
        let blockers: Vec<ObjectId> = cx
            .s()
            .blocks
            .iter()
            .filter(|b| b.attacker == attacker.id && live(cx.s(), b.blocker))
            .map(|b| b.blocker)
            .collect();
        if !attacker.blocked {
            assignments.push((attacker.id, attacker.target, power));
            continue;
        }
        if blockers.is_empty() {
            // CR 510.1c: a blocked attacker whose blockers are all gone assigns no combat damage - unless it has trample,
            // which then assigns all of it to the player or planeswalker it attacks (CR 702.19e).
            if cx.chars(attacker.id).has(Keyword::Trample) {
                assignments.push((attacker.id, attacker.target, power));
            }
            continue;
        }
        let mut left = power;
        for (i, &b) in blockers.iter().enumerate() {
            let chars = cx.chars(b);
            let lethal = (chars.toughness.unwrap_or(0) - cx.s().obj(b).damage as i16).max(0) as u16;
            let share = if i + 1 == blockers.len() { left } else { left.min(lethal) };
            assignments.push((attacker.id, Target::Object(b), share));
            left -= share;
        }
    }
    for block in cx.s().blocks.clone() {
        if live(cx.s(), block.blocker) && live(cx.s(), block.attacker) && deals_now(cx, block.blocker, first_strike_step) {
            let power = crate::static_rules::combat_damage_of(cx, block.blocker);
            assignments.push((block.blocker, Target::Object(block.attacker), power));
        }
    }
    // CR 510.2: all of it is dealt at once, and announced at once (CR 603.2c).
    let mut batch = ops::DamageBatch::default();
    let events: Vec<Event> = assignments
        .into_iter()
        .filter_map(|(source, target, amount)| ops::deal_damage_with(cx, source, target, amount, true, &mut batch))
        .collect();
    batch.gain_lifelink(cx);
    cx.emit_batch(&events, None);
    Ok(())
}

/// The damage division a blocked attacker has a choice in (the C# `AttackerDivision`): more than one blocker, or
/// trample and more damage than is lethal to its one blocker.
fn division(cx: &Cx, attacker: &Attacker) -> Option<DamageDivision> {
    let power = crate::static_rules::combat_damage_of(cx, attacker.id);
    let blockers: Vec<ObjectId> =
        cx.s().blocks.iter().filter(|b| b.attacker == attacker.id && live(cx.s(), b.blocker)).map(|b| b.blocker).collect();
    if power == 0 || blockers.is_empty() {
        return None;
    }
    let trample = cx.chars(attacker.id).has(Keyword::Trample);
    let deathtouch = cx.chars(attacker.id).has(Keyword::Deathtouch);
    let lethal: Vec<u16> = blockers
        .iter()
        .map(|&b| {
            if deathtouch {
                return 1;
            }
            let c = cx.chars(b);
            (c.toughness.unwrap_or(0) - cx.s().obj(b).damage as i16).max(0) as u16
        })
        .collect();
    if blockers.len() == 1 && (!trample || power <= lethal[0]) {
        return None;
    }
    // The suggestion: lethal to each in order, the rest to the last blocker, or with trample to the defender.
    let mut left = power;
    let mut suggested = Vec::new();
    for (i, &l) in lethal.iter().enumerate() {
        let share = if i + 1 == lethal.len() && !trample { left } else { left.min(l) };
        suggested.push(share);
        left -= share;
    }
    Some(DamageDivision {
        source: attacker.id,
        damage: power,
        creatures: blockers,
        lethal,
        may_assign_to_defender: trample,
        defender: if trample { attacker.target } else { Target::None },
        suggested,
        suggested_to_defender: if trample { left } else { 0 },
    })
}

/// Cleanup (CR 514): discard to hand size, then damage wears off and "until end of turn" effects end.
fn cleanup(cx: &mut Cx) -> R<()> {
    let active = cx.s().turn.active;
    // CR 514.1: "sacrifice it at the beginning of the next cleanup step", for the permanents whose card says so, each by its
    // controller, together.
    let doomed: Vec<ObjectId> = cx
        .s()
        .battlefield()
        .iter()
        .copied()
        .filter(|&id| {
            let obj = cx.s().obj(id);
            cx.card(obj.copy.map_or(obj.card, |c| c.card)).sacrifice_at_cleanup.is_some_and(|rule| rule(&*cx, id))
        })
        .collect();
    if !doomed.is_empty() {
        ops::sacrifice(cx, &doomed);
    }
    let hand = cx.s().zone(Zone::hand(active)).to_vec();
    if hand.len() > 7 && !crate::actions::no_maximum_hand_size(cx, active) {
        let count = (hand.len() - 7) as u8;
        if let Answer::Cards(cards) = cx.ask_core(active, RequestKind::DiscardToHandSize { count, hand }, false)? {
            ops::discard_together(cx, &cards);
        }
    }
    // CR 514.2: all damage marked on permanents is removed, except where a static ability says it isn't ("damage isn't
    // removed from this creature during cleanup steps"). Having been dealt deathtouch damage matters only to the next
    // check of state-based actions (CR 704.5h), so that record goes either way.
    let damaged: Vec<ObjectId> =
        cx.s().battlefield().iter().copied().filter(|&id| cx.s().obj(id).damage > 0 || cx.s().obj(id).deathtouched).collect();
    if !damaged.is_empty() {
        let stays = crate::static_rules::damage_stays(cx, &damaged);
        for id in damaged {
            let keep = stays.contains(&id);
            cx.st().update_obj(id, |o| {
                if !keep {
                    o.damage = 0;
                }
                o.deathtouched = false;
            });
        }
    }
    // "This turn" delayed abilities end with it (CR 603.7b).
    for at in (0..cx.s().delayed.len()).rev() {
        if cx.s().delayed[at].when == DelayedWhen::ThisTurn {
            cx.st().delayed_remove(at);
        }
    }
    if cx.s().effects.iter().any(|e| e.duration == Duration::EndOfTurn) {
        // A permanent whose control was only taken until end of turn goes back (CR 613.1b).
        let controlled: Vec<ObjectId> = cx
            .s()
            .effects
            .iter()
            .filter(|e| e.duration == Duration::EndOfTurn && matches!(e.kind, EffectKind::Control(_)))
            .map(|e| e.affected)
            .collect();
        // A permanent that was only a copy until end of turn is what it was again (CR 707.2, 611.2b): the latest copy effect
        // first, so the oldest one's prior values are what is left.
        let restored: Vec<(ObjectId, Option<Copiable>)> = cx
            .s()
            .effects
            .iter()
            .rev()
            .filter(|e| e.duration == Duration::EndOfTurn)
            .filter_map(|e| match e.kind {
                EffectKind::RestoreCopy(prior) if live(cx.s(), e.affected) => Some((e.affected, prior)),
                _ => None,
            })
            .collect();
        cx.st().effects_retain(|e| e.duration != Duration::EndOfTurn);
        for (id, prior) in restored {
            cx.st().update_obj(id, |o| o.copy = prior);
        }
        for id in controlled {
            ops::refresh_control(cx, id);
        }
    }
    // Permissions to play cards end with the turn they name (CR 514.2), "this turn" or "your next turn"; one for a
    // card that has since moved is gone with the old object (CR 400.7).
    if !cx.s().permissions.is_empty() {
        let number = cx.s().turn.number;
        let moved: Vec<ObjectId> =
            cx.s().permissions.iter().map(|p| p.card).filter(|&c| cx.s().obj(c).became.is_some()).collect();
        cx.st().permissions_retain(|p| {
            !moved.contains(&p.card)
                && match p.grant.expiry {
                    Expiry::Never => true,
                    Expiry::EndOfTurn => false,
                    Expiry::EndOfYourNextTurn => !(p.player == active && number > p.turn),
                    Expiry::UntilYourNextEndStep => true,
                }
        });
    }
    Ok(())
}

/// An effect ends the turn (CR 724.1), as the spell or ability with that instruction resolves. What follows the
/// resolution - no more priority in this step, the step's end with combat's, the cleanup step - is `Turn::ending`'s, read
/// by `priority` and `next_step`.
pub fn end_the_turn(cx: &mut Cx) -> R<()> {
    let player = cx.s().turn.active;
    cx.record(Event::TurnEnded { player });
    // CR 724.1a: abilities that triggered before this began and are not yet on the stack cease to exist. Those that
    // trigger from here on wait for the cleanup step (724.1f).
    while let Some(last) = cx.s().triggers.len().checked_sub(1) {
        cx.st().trigger_remove(last);
    }
    // CR 724.1b: every object on the stack is exiled, the resolving one among them, all at once. An ability on the stack
    // is no card, and is gone; a copy of a spell, exiled, ceases to exist as the state-based actions are checked
    // (CR 704.5e, 707.10a). Exiled is not countered.
    let spells: Vec<ops::Moving> = cx
        .s()
        .stack
        .iter()
        .rev()
        .filter(|i| i.kind == StackKind::Spell)
        .map(|i| ops::Moving::to(i.object, Zone::exile(cx.s().obj(i.object).owner)))
        .collect();
    for at in (0..cx.s().stack.len()).rev() {
        if cx.s().stack[at].kind != StackKind::Spell {
            cx.st().stack_remove(at);
        }
    }
    if !spells.is_empty() {
        ops::move_together(cx, &spells, ZoneReason::Exile);
    }
    // CR 724.1c: state-based actions, with no player given priority and no triggered ability put on the stack.
    crate::settle::state_based_actions_alone(cx)?;
    // CR 724.1d: the step ends once this resolution is over, and the game skips straight to the cleanup step.
    cx.st().update_turn(|t| t.ending = true);
    Ok(())
}

/// The next player in turn order after `p` who has not lost (CR 800.4a).
fn next_in_game(s: &GameState, p: PlayerId) -> PlayerId {
    let mut next = s.next_player(p);
    while s.player(next).lost && next != p {
        next = s.next_player(next);
    }
    next
}

/// A priority round: state-based actions and triggers first, then the player with priority acts or passes; when
/// everyone has passed in succession the top of the stack resolves, or with an empty stack the step ends (CR 117).
pub fn priority(cx: &mut Cx) -> R<Flow> {
    // CR 724.1c-d: an effect has ended the turn. No player gets priority again in this step, and the abilities that
    // triggered since wait for the cleanup step (724.1f).
    if cx.s().turn.ending {
        return Ok(Flow::Done);
    }
    if cx.s().unsettled || !cx.s().triggers.is_empty() {
        cx.st().frame_push(Frame::new(FrameKind::Settle));
        return Ok(Flow::Again);
    }
    if cx.s().is_over() {
        return Ok(Flow::Done);
    }
    // A player who has lost has left the game and is not given priority (CR 800.4a): it goes to the next player still in.
    let mut player = cx.s().turn.priority;
    if cx.s().player(player).lost {
        player = next_in_game(cx.s(), player);
        cx.st().update_turn(|t| t.priority = player);
    }
    let options = actions::priority_options(cx, player);
    match cx.ask_core(player, RequestKind::Priority { options }, false)? {
        Answer::Act(action) => {
            let mut frame = Frame::new(FrameKind::Act { player });
            frame.answers.push(Answer::Act(action));
            cx.st().frame_push(frame);
            Ok(Flow::Again)
        }
        _ => {
            let players = cx.s().apnap().count() as u8;
            let passes = cx.s().turn.passes + 1;
            if passes < players {
                let next = next_in_game(cx.s(), player);
                cx.st().update_turn(|t| {
                    t.passes = passes;
                    t.priority = next;
                });
                return Ok(Flow::Again);
            }
            let active = cx.s().turn.active;
            cx.st().update_turn(|t| {
                t.passes = 0;
                t.priority = active;
            });
            if cx.s().stack.is_empty() {
                return Ok(Flow::Done);
            }
            cx.st().frame_push(Frame::new(FrameKind::Resolve));
            Ok(Flow::Again)
        }
    }
}

#[allow(dead_code)]
fn _unused(cx: &Cx) -> bool {
    layers::live(cx.s(), ObjectId(0))
}
