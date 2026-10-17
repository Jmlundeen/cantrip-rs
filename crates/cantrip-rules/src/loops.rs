//! A loop of mandatory actions (CR 104.4b, 732.4): "If a game ... somehow enters a 'loop' of mandatory actions, repeating a
//! sequence of events with no way to stop, the game is a draw. Loops that contain an optional action don't result in a
//! draw."
//!
//! The rules find it the way the definition reads. A player who is asked a question with more than one legal answer has a
//! real choice (an optional action, or a different one), and any such question clears the watch: a loop it is part of is
//! one a player can stop, and is left to the players (CR 732.2-732.3). A question with one legal answer is no choice: a
//! lone target, the only host an Aura can enter attached to, and priority with nothing to do but pass, or tap a mana ability
//! whose whole cost is {T} when no spell or ability that more mana would pay for could be cast or activated
//! (`actions::could_act_with_more_mana`: lands that return untapped each time round could otherwise float mana until it
//! is paid for). Every priority given with something on the stack and no real choice since the last is counted; past
//! `WATCH_AFTER` of them each priority is judged in full and its position fingerprinted, and a fingerprint seen again
//! means the game has come back to where it was, through mandatory actions alone, and would go round forever: the game is
//! a draw.
//!
//! The fingerprint is the position up to the identity of objects (CR 400.7: an exiled card that returns is a new object, so
//! object ids and timestamps grow round a loop although nothing else changes): each object is numbered by where it is,
//! a reference to an object that has left is followed to what it became (with what its row and last-known information
//! still say), and timestamps are replaced by their order. The event history and the request counter are not part of it
//! (`docs/known-issues.md`, KI-54). Anything else that differs round the loop keeps it from matching, which errs towards
//! no draw.
//!
//! A normal game pays one comparison per priority: the fingerprinting starts only after a long run of priorities in which
//! nobody could do anything, which a game outside a loop almost never has.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cantrip_api::*;
use cantrip_state::{LastKnown, LoopWatch};

use crate::Cx;

/// Priorities given with something on the stack and no real choice in between before the positions are fingerprinted:
/// enough that a few triggers resolving with nothing to respond with never pay for it.
pub const WATCH_AFTER: u16 = 16;

/// Whether `kind` has exactly one legal answer, so that answering it is no choice (CR 104.4b). Errs towards "a choice":
/// any question not known to have one answer counts as one. Priority is judged by `priority_forced`.
pub fn forced(kind: &RequestKind) -> bool {
    match kind {
        RequestKind::ChooseTarget { candidates, optional, least, count, any_number, .. } => {
            !optional && !any_number && least == count && *count as usize == candidates.len()
        }
        RequestKind::ChooseCards { eligible, count, up_to, .. } => !up_to && *count as usize == eligible.len(),
        RequestKind::ChooseModes { eligible, count, at_most, repeat, .. } => !repeat && *at_most == 0 && *count as usize == eligible.len(),
        RequestKind::ChoosePlayer { candidates, .. } => candidates.len() == 1,
        RequestKind::OrderTriggers { abilities, .. } => abilities.len() < 2,
        _ => false,
    }
}

/// Whether priority, offering `options` besides passing, is no choice (CR 104.4b): nothing but mana abilities whose cost is
/// {T} alone, which only add mana.
fn priority_forced(cx: &Cx, options: &[ActionOption]) -> bool {
    // Anything but an activation is a choice, found without computing a characteristic.
    if options.iter().any(|o| o.kind != ActionKind::ActivateAbility || o.index >= PLAYER_MANA_ABILITY) {
        return false;
    }
    options.iter().all(|o| {
        o.kind == ActionKind::ActivateAbility
            && o.index < PLAYER_MANA_ABILITY
            && o.source.is_some_and(|id| {
                cx.chars(id).abilities().nth(o.index as usize).is_some_and(|ability| match cx.cards().ability(ability) {
                    AbilityDef::Activated(a) => a.is_mana_ability() && a.cost.is_tap_only(),
                    _ => false,
                })
            })
    })
}

/// A question other than priority is asked: one with more than one legal answer clears the watch.
pub fn asked(cx: &mut Cx, kind: &RequestKind) {
    if !matches!(kind, RequestKind::Priority { .. }) && !forced(kind) {
        clear(cx);
    }
}

fn clear(cx: &mut Cx) {
    let w = &cx.s().loop_watch;
    if w.forced != 0 || !w.seen.is_empty() {
        cx.st().set_loop_watch(LoopWatch::default());
    }
}

/// `player` is about to be given priority, with `options` besides passing. Whether the game has just come round a loop
/// of mandatory actions and is a draw (CR 104.4b): the draw is then made, and nobody is asked anything.
pub fn at_priority(cx: &mut Cx, player: PlayerId, options: &[ActionOption]) -> bool {
    if cx.s().stack.is_empty() || !priority_forced(cx, options) {
        clear(cx);
        return false;
    }
    let mut watch = cx.s().loop_watch.clone();
    if watch.forced < WATCH_AFTER {
        watch.forced += 1;
        cx.st().set_loop_watch(watch);
        return false;
    }
    // Tapping lands is no choice only if the mana could buy nothing: with a spell or an ability that more mana would pay for,
    // a player going round a loop could float mana until it is paid for, and stop the loop (CR 104.4b's last sentence). Asked
    // only once the cheap count says a loop may be under way, and of every priority from then on, so a loop is found only if
    // every priority of a whole round of it passes.
    if !options.is_empty() && crate::actions::could_act_with_more_mana(cx, player) {
        clear(cx);
        return false;
    }
    let print = fingerprint(cx);
    // The same priority asked again (a run redone with its answers) is the same question, not a loop.
    if watch.seen.last() == Some(&print) {
        return false;
    }
    if watch.seen.contains(&print) {
        cx.st().set_loop_watch(LoopWatch::default());
        cx.st().set_outcome(Outcome::Draw);
        cx.record(Event::GameDrawn { reason: DrawReason::MandatoryLoop });
        return true;
    }
    watch.seen.push(print);
    cx.st().set_loop_watch(watch);
    false
}

/// The position up to the identity of objects and the order of timestamps (see the module).
pub fn fingerprint(cx: &Cx) -> u64 {
    let s = cx.s();
    // Each object in a zone, numbered by where it is.
    let mut number = vec![u32::MAX; s.objects.len()];
    let mut n = 0u32;
    for zone in &s.zones {
        for &id in zone {
            number[id.index()] = n;
            n += 1;
        }
    }
    // What an object that left still says (CR 608.2h: an ability of it reads it as it last was): its row and its last-known
    // information, without the ids and timestamps that differ round a loop.
    let departed = |id: ObjectId| -> u64 {
        let mut o = *s.obj(id);
        o.timestamp = 0;
        o.became = None;
        o.attached_to = None;
        o.entered_with = None;
        o.prepared_from = None;
        o.exiled = None;
        let known = s.last_known(id).map(|k| LastKnown { id: ObjectId::NONE, ..*k });
        let mut h = DefaultHasher::new();
        (o, known).hash(&mut h);
        h.finish()
    };
    let canon = |id: ObjectId| -> u64 {
        if id == ObjectId::NONE {
            return u64::MAX;
        }
        // An object that left is named by what it became, how many moves ago, and what it still says.
        let mut at = id;
        for moves in 0..16u64 {
            if at.index() >= number.len() {
                break;
            }
            if number[at.index()] != u32::MAX {
                let here = moves << 32 | number[at.index()] as u64;
                return if moves == 0 { here } else { here ^ departed(id).rotate_left(17) };
            }
            match s.obj(at).became {
                Some(next) => at = next,
                None => break,
            }
        }
        departed(id)
    };
    let target = |t: Target| -> (u8, u64) {
        match t {
            Target::None => (0, 0),
            Target::Player(p) => (1, p.0 as u64),
            Target::Object(id) => (2, canon(id)),
            Target::Ability(serial) => (3, s.stack.iter().position(|i| i.serial == serial).map_or(u64::MAX, |at| at as u64)),
        }
    };
    let members = |span: cantrip_state::Span| -> Vec<u64> {
        let start = span.start as usize;
        s.members.get(start..start + span.len as usize).unwrap_or(&[]).iter().map(|&id| canon(id)).collect()
    };
    // Timestamps by their order (CR 613.7).
    let mut stamps: Vec<u32> = s.zones.iter().flatten().map(|&id| s.obj(id).timestamp).collect();
    stamps.extend(s.effects.iter().map(|e| e.timestamp));
    stamps.extend(s.counter_stamps.iter().map(|c| c.timestamp));
    stamps.sort_unstable();
    stamps.dedup();
    let rank = |t: u32| stamps.binary_search(&t).unwrap_or(usize::MAX);

    let mut h = DefaultHasher::new();
    s.turn.hash(&mut h);
    s.players.hash(&mut h);
    s.restricted_mana.hash(&mut h);
    for m in &s.mana_from {
        (m.player, m.restriction, canon(m.source), m.lasting, m.pool).hash(&mut h);
    }
    for c in &s.counter_stamps {
        (canon(c.object), c.kind, rank(c.timestamp)).hash(&mut h);
    }
    for zone in &s.zones {
        zone.len().hash(&mut h);
        for &id in zone {
            let mut o = *s.obj(id);
            let refs = (
                o.attached_to.map(canon),
                o.entered_with.map(canon),
                o.prepared_from.map(canon),
                o.exiled.map(|e| (canon(e.by), e.from, e.until_leaves, e.encoded)),
            );
            o.timestamp = rank(o.timestamp) as u32;
            o.became = None;
            o.attached_to = None;
            o.entered_with = None;
            o.prepared_from = None;
            o.exiled = None;
            (o, refs).hash(&mut h);
        }
    }
    for item in &s.stack {
        let mut i = *item;
        let refs = (canon(i.object), i.targets().iter().map(|&t| target(t)).collect::<Vec<_>>(), target(i.subject), members(i.members));
        i.object = ObjectId::NONE;
        i.targets = [Target::None; cantrip_state::MAX_TARGETS];
        i.subject = Target::None;
        i.members = cantrip_state::Span::default();
        i.serial = 0;
        i.since = 0;
        (i, refs).hash(&mut h);
    }
    for e in &s.effects {
        (canon(e.source), e.controller, rank(e.timestamp), canon(e.affected), e.kind, e.duration).hash(&mut h);
    }
    for p in &s.permissions {
        (canon(p.card), p.player, p.grant, p.turn).hash(&mut h);
    }
    for t in &s.triggers {
        (canon(t.source), t.ability, t.controller, target(t.subject), t.data, members(t.members), t.count).hash(&mut h);
    }
    for d in &s.delayed {
        (d.when, canon(d.source), d.controller, d.ability, target(d.subject), members(d.members)).hash(&mut h);
    }
    for a in &s.attackers {
        (canon(a.id), target(a.target), a.blocked).hash(&mut h);
    }
    for b in &s.blocks {
        (canon(b.blocker), canon(b.attacker)).hash(&mut h);
    }
    s.extra_turns.hash(&mut h);
    s.rng.hash(&mut h);
    for f in &s.agenda {
        f.kind.hash(&mut h);
    }
    h.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;

    fn never(_: &dyn Query, _: ObjectId, _: &Event) -> Option<Fired> {
        None
    }

    fn nothing(_: &mut dyn cantrip_api::Game, _: &StackItem) -> Result<(), Suspend> {
        Ok(())
    }

    static CARD: CardDef = CardDef {
        name: "Loop Test",
        types: TypeSet::of(&[CardType::Enchantment]),
        abilities: &[triggered(when(EventMask::ZONE_CHANGE, never), nothing, "")],
        ..CardDef::BLANK
    };

    /// A game with one Loop Test on the battlefield and its ability on the stack.
    fn game() -> Game {
        let table: &'static CardTable = Box::leak(Box::new(CardTable::new([&CARD])));
        let mut game = Game::new(table, &crate::Setup { decks: vec![Vec::new(), Vec::new()], ..Default::default() });
        let card = table.id_of(&CARD);
        let id = game.state.push_obj(Obj::new(card, PlayerId(0), Zone::BATTLEFIELD));
        game.state.zone_push(Zone::BATTLEFIELD, id);
        let item = crate::actions::stack_item(StackKind::Triggered, id, AbilityRef { card, index: 0 }, PlayerId(0), &[]);
        game.state.stack_push(item);
        game
    }

    fn cx(game: &mut Game) -> Cx<'_> {
        Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None }
    }

    /// The permanent leaves and comes back: a new object (CR 400.7), at a later timestamp, and its ability triggers again,
    /// the ability's source the object that left.
    fn flicker(game: &mut Game) {
        let old = game.state.battlefield()[0];
        let item = game.state.stack_pop().expect("the ability");
        game.state.zone_remove(Zone::BATTLEFIELD, old);
        let t = game.state.timestamp();
        let mut obj = Obj::new(game.state.obj(old).card, PlayerId(0), Zone::BATTLEFIELD);
        obj.timestamp = t;
        let new = game.state.push_obj(obj);
        game.state.update_obj(old, |o| o.became = Some(new));
        game.state.zone_push(Zone::BATTLEFIELD, new);
        game.state.stack_push(StackItem { object: old, serial: 0, ..item });
    }

    /// CR 104.4b, 400.7: a position that has come round again is the same whatever its objects are numbered and whatever
    /// their timestamps, but anything else that differs - a life total - makes it another.
    #[test]
    fn a_position_reached_again_with_new_objects_has_the_same_fingerprint() {
        let mut game = game();
        flicker(&mut game);
        let before = fingerprint(&cx(&mut game));
        flicker(&mut game);
        assert_eq!(fingerprint(&cx(&mut game)), before, "a new object of the same card where the old one was");
        game.state.update_player(PlayerId(1), |p| p.life -= 1);
        assert_ne!(fingerprint(&cx(&mut game)), before, "a life total changed");
    }

    /// CR 104.4b: a question with one legal answer is no choice; one with more is.
    #[test]
    fn only_a_question_with_one_legal_answer_is_forced() {
        let choose = |candidates: Vec<Target>, optional: bool| RequestKind::ChooseTarget {
            source: ObjectId(0),
            purpose: Purpose::Targets,
            candidates,
            optional,
            least: 1,
            count: 1,
            any_number: false,
            label: "",
            slot_player: None,
            total: None,
        };
        assert!(forced(&choose(vec![Target::Object(ObjectId(1))], false)));
        assert!(!forced(&choose(vec![Target::Object(ObjectId(1))], true)), "choosing none is another answer");
        assert!(!forced(&choose(vec![Target::Object(ObjectId(1)), Target::Object(ObjectId(2))], false)));
        // "One or two targets" with two candidates: choosing one of them is another answer (CR 601.2c).
        let one_or_two = RequestKind::ChooseTarget {
            source: ObjectId(0),
            purpose: Purpose::Targets,
            candidates: vec![Target::Object(ObjectId(1)), Target::Object(ObjectId(2))],
            optional: false,
            least: 1,
            count: 2,
            any_number: false,
            label: "",
            slot_player: None,
            total: None,
        };
        assert!(!forced(&one_or_two));
        assert!(!forced(&RequestKind::May { source: ObjectId(0), purpose: Purpose::Optional }));
    }

    /// CR 104.4b, 732.4: the same position at priority, with no choice made since, is a draw; a real choice in between
    /// (an optional action, CR 104.4b's last sentence) starts the watch over.
    #[test]
    fn a_position_repeated_through_forced_priorities_alone_is_a_draw() {
        let mut game = game();
        let mut drawn = false;
        for _ in 0..WATCH_AFTER {
            flicker(&mut game);
            // Somebody chooses between two players each time round: never a draw.
            let two = RequestKind::ChoosePlayer { source: ObjectId(0), candidates: vec![PlayerId(0), PlayerId(1)], verb: "", amount: 0 };
            asked(&mut cx(&mut game), &two);
            for p in [PlayerId(0), PlayerId(1)] {
                game.state.update_turn(|t| t.priority = p);
                drawn |= at_priority(&mut cx(&mut game), p, &[]);
            }
        }
        assert!(!drawn && game.state.outcome == Outcome::InProgress);
        // Each player passes in turn, round after round.
        'round: for _ in 0..WATCH_AFTER {
            flicker(&mut game);
            for p in [PlayerId(0), PlayerId(1)] {
                game.state.update_turn(|t| t.priority = p);
                if at_priority(&mut cx(&mut game), p, &[]) {
                    drawn = true;
                    break 'round;
                }
            }
        }
        assert!(drawn && game.state.outcome == Outcome::Draw);
        assert!(game.log.iter().any(|r| r.event == Event::GameDrawn { reason: DrawReason::MandatoryLoop }));
    }

    /// Nothing on the stack: passing in succession ends the step (CR 117.4), so no loop is watched for.
    #[test]
    fn an_empty_stack_is_never_a_loop() {
        let mut game = game();
        game.state.stack_pop();
        for _ in 0..WATCH_AFTER * 2 {
            assert!(!at_priority(&mut cx(&mut game), PlayerId(0), &[]));
        }
        assert_eq!(game.state.loop_watch, LoopWatch::default());
    }
}
