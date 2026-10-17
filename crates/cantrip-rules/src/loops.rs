//! Loops (CR 104.4b, 732). "If a game ... somehow enters a 'loop' of mandatory actions, repeating a sequence of events with
//! no way to stop, the game is a draw. Loops that contain an optional action don't result in a draw" (CR 104.4b); and a
//! loop a player could stop is shortcut (CR 732.2): its players choose how many times it goes round, and then one of them
//! must make a different choice (CR 732.3). The rules here find both, and see that every loop ends.
//!
//! **A loop of mandatory actions.** A player who is asked a question with more than one legal answer has a real choice (an
//! optional action, or a different one), and any such question clears the watch of a run of mandatory actions. A question
//! with one legal answer is no choice: a lone target, the only host an Aura can enter attached to, and priority with nothing
//! to do but pass, or tap a mana ability whose whole cost is {T} when no spell or ability that more mana would pay for could
//! be cast or activated (`actions::could_act_with_more_mana`: lands that return untapped each time round could otherwise
//! float mana until it is paid for). Every priority given with something on the stack and no real choice since the last is
//! counted; past `WATCH_AFTER` of them each priority is judged in full and its position fingerprinted, and a fingerprint
//! seen again means the game has come back to where it was, through mandatory actions alone, and would go round forever:
//! the game is a draw.
//!
//! **What a position is.** Whatever the game's future depends on, up to the identity of objects (CR 400.7: an exiled card
//! that returns is a new object, so object ids and timestamps grow round a loop although nothing else changes): each object
//! is numbered by where it is, a reference to an object that has left is followed to what it became (with what its row and
//! last-known information still say), and timestamps are replaced by their order. Every amount is in it - life totals,
//! poison, counters on objects, damage, mana in pools, the cards in each library - so a loop that changes one each time
//! round never comes back to the same position: it goes on until the amount runs out (a player at 0 life, CR 704.5a; a
//! draw from an empty library, CR 704.5b) or reaches what a card waits for. The event history is not in the fingerprint, as
//! no two rounds have the same one; instead the history counts every time it is read (`cantrip_api::history::note_read`),
//! and a read clears the positions seen so far: a position comes round again only if nothing asked the past in between, so
//! that "the third time this turn", "only once each turn" and "if you've cast two spells this turn" can never be what tells
//! two rounds apart. A question about the past whose answer no later event of the turn can change ("it has triggered once
//! this turn already") is asked through `settled_read`, which counts it only while it could still change.
//!
//! **A loop that changes something each time round, or that reads the past.** The position less its amounts is its
//! shape. A position at priority whose shape comes round again, with every amount gone the way no rule ends the game or a
//! permanent by (life gained, counters added, mana floated: never life lost, poison or damage added, counters removed),
//! is a loop going nowhere but up; `ROUNDS` times round, more than any number a card waits for, and with no player choosing
//! to act, it is a loop of mandatory actions with no way to stop it, and a draw (CR 104.4b) - and so is a loop with nothing
//! but the history between its rounds. One whose amounts go down is left to run until they run out. That watch begins only
//! once a step has a great deal in it (`BUSY_STEP` events), which a step outside a loop rarely has (a long combination turn
//! may; it then only pays for the watch).
//!
//! **A loop a player could stop** (CR 732). In such a step every question asked is kept with the shape of its position and
//! the answers given to it there. A loop whose players choose to act each time round (an ability activated, a "may" taken,
//! Animate Dead's choice of a creature card made the same way) is not a draw (CR 104.4b's last sentence). No player names
//! how many times it goes round (CR 732.2a): the engine takes `ROUNDS_TO_CHOOSE` rounds as the number they chose, and then
//! the player chosen as CR 732.3 chooses one - the active player if they are among those acting, else the next of them in
//! turn order - must make a different choice for the rest of the step: offered the same question in the same shape of
//! position, the answers they gave it before are no longer among its answers. Another of them is chosen if it goes on. What
//! is made to differ is only ever an action taken or a thing chosen; a pass or a decline is never refused (the engine's
//! policy: CR 732.5 lets a player be held to an action a loop object calls for, but no rule requires it, and taking a "may"
//! that would not stop the loop - drawing a card each time round - would only harm them). A loop in which the players only
//! pass or decline - holding an instant that would stop it, or an ability no object of the loop calls for, which no player
//! can be made to use (CR 732.5) - goes round `ROUNDS` times and is then a loop of mandatory actions (CR 732.4), a draw.
//!
//! **That every loop ends.** A run of mandatory actions `FORCED_LIMIT` priorities long whose life totals, poison and
//! libraries have gone nowhere for half of it is a loop that never comes back to a position (an amount or a list growing
//! for ever), a draw, unless the stack or the board shrank in it too (a long chain that ends); and a step with `STEP_LIMIT`
//! events in it is past any number of times the players could have meant to go round, so its players may only pass, and
//! decline what they may, from then on, and every answer counts as passing. A step twice that long - a loop through choices
//! the engine can't refuse that never comes back to a shape - is judged endless, and the game a draw: the engine's last
//! word, not a rule's.
//!
//! A normal game pays a comparison per priority and a count per read of the history: the fingerprinting starts only after
//! a long run of priorities in which nobody could do anything, or in a step far longer than a game outside a loop has.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cantrip_api::history::{note_read, take_reads};
use cantrip_api::*;
use cantrip_state::{GameState, LastKnown, LoopKey, LoopRound, LoopWatch};

use crate::Cx;

/// Priorities given with something on the stack and no real choice in between before the positions are fingerprinted:
/// enough that a few triggers resolving with nothing to respond with never pay for it.
pub const WATCH_AFTER: u32 = 16;

/// The longest run of mandatory actions that is let go on without coming back to a position, in priorities: a loop that
/// changes something round after round without end (CR 104.4b).
pub const FORCED_LIMIT: u32 = 1 << 14;

/// The times a loop that changes nothing but the history, or only amounts no rule ends anything by, comes round before it is
/// a draw: more than any number a card counts to (a hundred counters, fifty life, the seventh card drawn this turn).
pub const ROUNDS: u16 = 256;

/// The times a loop its players choose to go on with comes round before one of them must choose otherwise (CR 732.2a,
/// 732.3): the number of times they are taken to have chosen.
pub const ROUNDS_TO_CHOOSE: u16 = 32;

/// Events in a step before its questions are kept with their positions: far more than a step outside a loop has.
pub const BUSY_STEP: u32 = 1024;

/// Events in a step past which its players may only pass and decline (see the module).
pub const STEP_LIMIT: u32 = 1 << 18;

/// The most positions or questions the watch keeps: past it, a loop that long is not looked for (it is started over).
const KEPT: usize = 1024;

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

/// A question about the past whose answer, once `settled` says so, no later event of this turn can change (a count that
/// has reached its limit, "the first card drawn this turn" once one has been): asked of the history, and counted as a read
/// of it (`note_read`) only while it could still change. A loop within the turn can't be told apart by a settled answer.
pub fn settled_read<T>(cx: &Cx, ask: impl FnOnce(&History) -> T, settled: impl FnOnce(&T) -> bool) -> T {
    let history = History { records: &cx.g.log, turn: cx.s().turn.number };
    let answer = ask(&history);
    if !settled(&answer) {
        note_read();
    }
    answer
}

/// What a question comes to once the watch has seen it.
pub enum Asked {
    /// Ask it, as it now is (with the answers a player who must choose otherwise gave before taken out), `key` its place in
    /// the watch when it is kept.
    Ask { kind: RequestKind, key: Option<u64> },
    /// It has one answer left, which is given without asking.
    Answered(Answer),
}

/// A question other than priority is about to be asked (or asked again, on a re-run): one with more than one legal answer
/// clears the watch of mandatory actions; in a step being watched it is kept with its position, and a player who must make
/// a different choice (CR 732.3) is not offered what they answered it before. Priority is `at_priority`'s. A question
/// that can be answered without asking (`may_answer`) may be answered here. A question being `replaying` (a frame run again
/// with the answers it had) is kept but never changed: its answer was given to it as it was first asked.
pub fn asked(cx: &mut Cx, player: PlayerId, kind: RequestKind, may_answer: bool, replaying: bool) -> Asked {
    if let RequestKind::Priority { .. } = kind {
        let key = cx.s().loop_watch.priority_key;
        return Asked::Ask { kind, key };
    }
    if forced(&kind) {
        return Asked::Ask { kind, key: None };
    }
    clear_forced(cx);
    let Some(records) = busy(cx) else { return Asked::Ask { kind, key: None } };
    let mut watch = cx.s().loop_watch.clone();
    let asked = {
        let canon = Canon::new(cx.s());
        let key = question_key(&canon, &position(cx.s(), &canon).shape, player, cx.frame, cx.cursor, &kind);
        let at = keep(&mut watch, key).0;
        if !replaying && (watch.designated & bit(player) != 0 || records >= STEP_LIMIT) {
            // Past the limit, every "may" is declined (see the module).
            let given = if records >= STEP_LIMIT { vec![answer_hash(&canon, &Answer::Yes)] } else { watch.keys[at].given.clone() };
            match otherwise(&canon, kind, &given, may_answer) {
                Asked::Ask { kind, .. } => Asked::Ask { kind, key: Some(key) },
                answered => answered,
            }
        } else {
            Asked::Ask { kind, key: Some(key) }
        }
    };
    cx.st().set_loop_watch(watch);
    asked
}

/// The question with the answers in `given` taken out, where what is left is still a question with an answer: a target,
/// card, mode or player chosen alone (`picks`); and a "may" taken before is declined without asking, if `may_answer`. A
/// player made to choose otherwise is only ever made to choose less, or another of the same: declining is never refused.
/// Any other question is asked as it is.
fn otherwise(canon: &Canon, mut kind: RequestKind, given: &[u64], may_answer: bool) -> Asked {
    let out = |a: Answer| given.contains(&answer_hash(canon, &a));
    match &mut kind {
        RequestKind::ChooseTarget { candidates, least, count: 1, any_number: false, .. } => {
            let left: Vec<Target> = candidates.iter().copied().filter(|&t| !out(Answer::Targets(vec![t]))).collect();
            if left.len() >= (*least as usize).max(1) || *least == 0 {
                *candidates = left;
            }
        }
        RequestKind::ChooseCards { eligible, count: 1, .. } => {
            let left: Vec<ObjectId> = eligible.iter().copied().filter(|&c| !out(Answer::Cards(vec![c]))).collect();
            if !left.is_empty() {
                *eligible = left;
            }
        }
        RequestKind::ChoosePlayer { candidates, .. } => {
            let left: Vec<PlayerId> = candidates.iter().copied().filter(|&p| !out(Answer::Player(p))).collect();
            if !left.is_empty() {
                *candidates = left;
            }
        }
        RequestKind::ChooseModes { eligible, count: 1, at_most: 0, repeat: false, .. } => {
            let left: Vec<u8> = eligible.iter().copied().filter(|&m| !out(Answer::Modes(vec![m]))).collect();
            if !left.is_empty() {
                *eligible = left;
            }
        }
        RequestKind::May { .. } if may_answer && out(Answer::Yes) => return Asked::Answered(Answer::No),
        _ => {}
    }
    Asked::Ask { kind, key: None }
}

/// The answers a question offers that choose one thing - a target, card, player or mode chosen alone - as the watch knows
/// them (`answer_hash`): what `otherwise` can take out. None for any other question.
fn picks(canon: &Canon, kind: &RequestKind) -> Option<Vec<u64>> {
    let hash = |a: Answer| answer_hash(canon, &a);
    Some(match kind {
        RequestKind::ChooseTarget { candidates, count: 1, any_number: false, .. } => candidates.iter().map(|&t| hash(Answer::Targets(vec![t]))).collect(),
        RequestKind::ChooseCards { eligible, count: 1, .. } => eligible.iter().map(|&c| hash(Answer::Cards(vec![c]))).collect(),
        RequestKind::ChoosePlayer { candidates, .. } => candidates.iter().map(|&p| hash(Answer::Player(p))).collect(),
        RequestKind::ChooseModes { eligible, count: 1, at_most: 0, repeat: false, .. } => eligible.iter().map(|&m| hash(Answer::Modes(vec![m]))).collect(),
        _ => return None,
    })
}

/// What an answer about to be given to a kept question (request `id`) comes to for the watch: the question's key, the
/// answer as the watch knows it, and whether giving it is choosing to act - doing something at priority, taking a "may", or
/// choosing one thing of several when another is still untried there. Passing, declining, and a choice every other answer
/// to which has been tried already, are no action that could keep a loop going that another would stop. Read before the
/// answer is played out, in the position it was asked in.
pub fn answering(s: &GameState, id: u32, request: &Request, answer: &Answer) -> Option<(u64, u64, bool)> {
    let (asked, key) = s.loop_watch.asking?;
    if asked != id {
        return None;
    }
    let canon = Canon::new(s);
    let this = answer_hash(&canon, answer);
    let given = s.loop_watch.keys.iter().find(|k| k.key == key).map_or(&[][..], |k| &k.given[..]);
    let acts = !forced(&request.kind)
        && match (&request.kind, answer) {
            (_, Answer::Pass | Answer::No) => false,
            // A "may" asked in a staged ability can't be declined for its player (`Game::ask_then`): no action, as any
            // other question `otherwise` can't change.
            (RequestKind::May { .. }, _) => !s.agenda.last().is_some_and(|f| f.staged),
            (RequestKind::Priority { .. }, _) => true,
            (_, Answer::Targets(t)) if t.is_empty() => false,
            (_, Answer::Cards(c)) if c.is_empty() => false,
            (kind, _) => picks(&canon, kind).is_some_and(|p| p.iter().any(|&h| h != this && !given.contains(&h))),
        };
    Some((key, this, acts))
}

/// The answer noted by `answering` is given, by `player`, to a question of this `kind`: it is kept with its question, and a
/// player who chose to act is among those involved in the loop (CR 732.3). Priority is played out again from the frame's
/// start, its watch with it, so a player acting at priority joins `involved` only as that priority is watched again
/// (`acted`).
pub fn answered(s: &mut GameState, player: PlayerId, priority: bool, (key, answer, acts): (u64, u64, bool)) {
    let mut watch = s.loop_watch.clone();
    let at = entry(&mut watch, key);
    if let Some(k) = watch.keys.get_mut(at)
        && !k.given.contains(&answer)
    {
        k.given.push(answer);
    }
    if acts && priority {
        watch.acted |= bit(player);
    } else if acts {
        watch.involved |= bit(player);
    }
    s.set_loop_watch(watch);
}

/// A question with request id `id` and watch key `key` is now waiting.
pub fn waiting(s: &mut GameState, id: u32, key: Option<u64>) {
    if let Some(key) = key {
        let mut watch = s.loop_watch.clone();
        watch.asking = Some((id, key));
        watch.priority_key = None;
        s.set_loop_watch(watch);
    }
}

fn bit(p: PlayerId) -> u8 {
    1u8.checked_shl(p.0 as u32).unwrap_or(0)
}

/// The step the game is in, as the watch names it.
fn step_id(s: &GameState) -> u32 {
    s.turn.number << 8 | s.turn.step_raw as u32
}

/// The events recorded in this step, if it is being watched for loops of choices (`BUSY_STEP`).
fn busy(cx: &Cx) -> Option<u32> {
    let w = &cx.s().loop_watch;
    let records = (cx.g.log.len() as u32).saturating_sub(w.step_log);
    (w.step == step_id(cx.s()) && records > BUSY_STEP).then_some(records)
}

/// The question `key` is asked: its entry in the watch (made if it has none), and whether it was asked before.
fn keep(watch: &mut LoopWatch, key: u64) -> (usize, bool) {
    let at = entry(watch, key);
    let k = &mut watch.keys[at];
    k.asked = k.asked.saturating_add(1);
    (at, k.asked > 1)
}

/// The watch's entry for `key`, made if it has none.
fn entry(watch: &mut LoopWatch, key: u64) -> usize {
    if let Some(at) = watch.keys.iter().position(|k| k.key == key) {
        return at;
    }
    if watch.keys.len() >= KEPT {
        watch.keys.clear();
        watch.round = None;
    }
    watch.keys.push(LoopKey { key, asked: 0, given: Vec::new() });
    watch.keys.len() - 1
}

/// The run of mandatory actions is broken: a real choice.
fn clear_forced(cx: &mut Cx) {
    let w = &cx.s().loop_watch;
    if w.forced != 0 || !w.seen.is_empty() || !w.far.is_empty() {
        let mut watch = w.clone();
        watch.forced = 0;
        watch.seen.clear();
        watch.far.clear();
        cx.st().set_loop_watch(watch);
    }
}

/// A frame of the rules has run: if it asked the history anything, the positions seen before can't be compared with any
/// after (see the module).
pub fn after_frame(s: &mut GameState) {
    if take_reads() > 0 && !s.loop_watch.seen.is_empty() {
        let mut watch = s.loop_watch.clone();
        watch.seen.clear();
        s.set_loop_watch(watch);
    }
}

/// The game is a draw: a loop of mandatory actions (CR 104.4b, 732.4).
fn draw(cx: &mut Cx) {
    let step = cx.s().loop_watch.step;
    cx.st().set_loop_watch(LoopWatch { step, ..LoopWatch::default() });
    cx.st().set_outcome(Outcome::Draw);
    cx.record(Event::GameDrawn { reason: DrawReason::MandatoryLoop });
}

/// `player` is about to be given priority, with `options` besides passing. `None` if the game has just come round a loop
/// of mandatory actions and is a draw (CR 104.4b): the draw is then made, and nobody is asked anything. Otherwise the
/// options to offer: all of them, but for what a player who must make a different choice (CR 732.3) did here before.
pub fn at_priority(cx: &mut Cx, player: PlayerId, mut options: Vec<ActionOption>) -> Option<Vec<ActionOption>> {
    // The step being watched for loops of choices begins with its first priority.
    let step = step_id(cx.s());
    if cx.s().loop_watch.step != step {
        let mut watch = cx.s().loop_watch.clone();
        watch.step = step;
        watch.step_log = cx.g.log.len() as u32;
        watch.keys.clear();
        watch.round = None;
        watch.involved = 0;
        watch.designated = 0;
        watch.asking = None;
        watch.priority_key = None;
        watch.acted = 0;
        cx.st().set_loop_watch(watch);
    }
    let mut seen_position = None;
    if mandatory(cx, player, &options, &mut seen_position) {
        return None;
    }
    let Some(records) = busy(cx) else {
        if cx.s().loop_watch.priority_key.is_some() {
            let watch = LoopWatch { priority_key: None, ..cx.s().loop_watch.clone() };
            cx.st().set_loop_watch(watch);
        }
        return Some(options);
    };
    if records >= 2 * STEP_LIMIT {
        draw(cx);
        return None;
    }
    let mut watch = cx.s().loop_watch.clone();
    let active = cx.s().turn.active;
    let order: Vec<PlayerId> = cx.s().turn_order_from(active).collect();
    let (drawn, options) = {
        let canon = Canon::new(cx.s());
        let here = seen_position.take().unwrap_or_else(|| position(cx.s(), &canon));
        let key = question_key(&canon, &here.shape, player, cx.frame, cx.cursor, &RequestKind::Priority { options: Vec::new() });
        watch.priority_key = Some(key);
        let (at, again) = keep(&mut watch, key);
        // A loop that has gone elsewhere is followed from the next position that comes round.
        if let Some(round) = &mut watch.round
            && round.key != key
        {
            round.since = round.since.saturating_add(1);
            if round.since as usize > KEPT / 2 {
                watch.round = None;
            }
        }
        // Past the limit nobody may act any more but by questions nothing can change: what is left is passing.
        if records >= STEP_LIMIT {
            watch.involved = 0;
            watch.acted = 0;
        }
        let drawn = again && round_comes(&mut watch, key, here.amounts, &order);
        watch.involved |= std::mem::take(&mut watch.acted);
        if records >= STEP_LIMIT {
            options.clear();
        } else if watch.designated & bit(player) != 0 {
            let given = &watch.keys[at].given;
            options.retain(|o| !given.contains(&option_hash(&canon, o)));
        }
        (drawn, options)
    };
    cx.st().set_loop_watch(watch);
    if drawn {
        draw(cx);
        return None;
    }
    Some(options)
}

/// A position at priority, `key`, has come round again in a step being watched, with these `amounts`; `order` is the
/// players in turn order from the active player. Whether the loop is one of mandatory actions and the game a draw.
fn round_comes(watch: &mut LoopWatch, key: u64, amounts: Vec<(u32, i32)>, order: &[PlayerId]) -> bool {
    let Some(round) = &mut watch.round else {
        watch.round = Some(LoopRound { key, amounts, ..LoopRound::default() });
        return false;
    };
    if round.key != key {
        return false;
    }
    round.since = 0;
    let mut drawn = false;
    if gone_nowhere(&round.amounts, &amounts) {
        round.rounds = round.rounds.saturating_add(1);
        round.passes = if watch.involved == 0 { round.passes.saturating_add(1) } else { 0 };
        // Round and round with nobody choosing to act: no player can be made to stop it (CR 732.5), and it is a loop of
        // mandatory actions (CR 732.4).
        drawn = round.passes >= ROUNDS;
        if watch.involved != 0 && round.rounds >= ROUNDS_TO_CHOOSE {
            // CR 732.3: the active player, if they are among those acting in the loop, or else the first of them in turn
            // order, must make a different choice.
            if let Some(&p) = order.iter().find(|&&p| watch.involved & bit(p) != 0 && watch.designated & bit(p) == 0) {
                watch.designated |= bit(p);
            }
            round.rounds = 0;
        }
    } else {
        round.rounds = 0;
        round.passes = 0;
    }
    round.amounts = amounts;
    watch.involved = 0;
    drawn
}

/// The watch of a run of mandatory actions, at a priority: whether it has come round a loop and the game is a draw. The
/// position it fingerprinted, if it did, is left in `seen_position`.
fn mandatory(cx: &mut Cx, player: PlayerId, options: &[ActionOption], seen_position: &mut Option<Position>) -> bool {
    if cx.s().stack.is_empty() || !priority_forced(cx, options) {
        clear_forced(cx);
        return false;
    }
    let mut watch = cx.s().loop_watch.clone();
    watch.forced = watch.forced.saturating_add(1);
    if watch.forced <= WATCH_AFTER {
        cx.st().set_loop_watch(watch);
        return false;
    }
    // Tapping lands is no choice only if the mana could buy nothing: with a spell or an ability that more mana would pay for,
    // a player going round a loop could float mana until it is paid for, and stop the loop (CR 104.4b's last sentence). Asked
    // only once the cheap count says a loop may be under way, and of every priority from then on, so a loop is found only if
    // every priority of a whole round of it passes.
    if !options.is_empty() && crate::actions::could_act_with_more_mana(cx, player) {
        clear_forced(cx);
        return false;
    }
    // The history asked since the last position was seen - deciding what may be done here among it - and the positions seen
    // before can't be compared with this one.
    let read = take_reads() > 0;
    let canon = Canon::new(cx.s());
    let here = position(cx.s(), &canon);
    // The same priority asked again (a run redone with its answers) is the same question, not a loop.
    if watch.seen.last() == Some(&here.exact) {
        *seen_position = Some(here);
        return false;
    }
    if read {
        watch.seen.clear();
    }
    if watch.seen.contains(&here.exact) {
        draw(cx);
        return true;
    }
    if watch.seen.len() >= KEPT {
        watch.seen.clear();
    }
    watch.seen.push(here.exact);
    // A run that long has never come back to a position: something in it grows without end. If nothing that ends a game -
    // a life total, poison, a library - has moved towards its end for half of it, nothing will.
    let players = players_amounts(cx.s());
    if watch.forced == FORCED_LIMIT / 2 {
        watch.far = players;
    } else if watch.forced >= FORCED_LIMIT {
        let far: Vec<(u32, i32)> = watch.far.chunks(2).map(|c| (c[0] as u32, c[1])).collect();
        if gone_nowhere(&far, &players.chunks(2).map(|c| (c[0] as u32, c[1])).collect::<Vec<_>>()) {
            draw(cx);
            return true;
        }
        watch.forced = FORCED_LIMIT / 2;
        watch.far = players;
    }
    cx.st().set_loop_watch(watch);
    *seen_position = Some(here);
    false
}

// --- Positions -----------------------------------------------------------------------------------------------------------

/// What an amount is, as `gone_nowhere` judges its change: the top bits of its code (`code`).
mod amount {
    pub const LIFE: u32 = 0;
    pub const POISON: u32 = 1;
    pub const ENERGY: u32 = 2;
    pub const EXPERIENCE: u32 = 3;
    pub const LIBRARY: u32 = 4;
    pub const STACK: u32 = 5;
    pub const OBJECTS: u32 = 6;
    pub const POOL: u32 = 8;
    pub const LASTING: u32 = 16;
    pub const RESTRICTED: u32 = 24;
    pub const FROM: u32 = 32;
    pub const DAMAGE: u32 = 48;
    pub const COUNTER: u32 = 64;
}

fn code(what: u32, slot: usize) -> u32 {
    what << 20 | (slot as u32 & 0xF_FFFF)
}

/// Whether an amount went from `old` to `new` only the way no rule ends a game or a permanent by: a life total, energy,
/// experience, mana or a counter up (or not at all), poison, damage or -1/-1 counters down (or not at all). An amount that
/// moves the other way is heading for where it ends something (CR 704.5a-c, 704.5f-g, 704.5i, 714.4 ...).
fn up_only(code: u32, old: i32, new: i32) -> bool {
    use amount::*;
    match code >> 20 {
        POISON | DAMAGE => new <= old,
        k if k >= COUNTER => {
            if CounterKind::ALL.get((k - COUNTER) as usize) == Some(&CounterKind::MinusOne) {
                new <= old
            } else {
                new >= old
            }
        }
        _ => new >= old,
    }
}

/// Whether every amount of `new` (sorted by code, as `position` lists them) moved from `old` only `up_only`.
fn gone_nowhere(old: &[(u32, i32)], new: &[(u32, i32)]) -> bool {
    let (mut i, mut j) = (0, 0);
    while i < old.len() || j < new.len() {
        let (code, a, b) = match (old.get(i), new.get(j)) {
            (Some(&(c, a)), Some(&(d, b))) if c == d => {
                i += 1;
                j += 1;
                (c, a, b)
            }
            (Some(&(c, a)), Some(&(d, _))) if c < d => {
                i += 1;
                (c, a, 0)
            }
            (Some(&(c, a)), None) => {
                i += 1;
                (c, a, 0)
            }
            (_, Some(&(d, b))) => {
                j += 1;
                (d, 0, b)
            }
            (None, None) => unreachable!(),
        };
        if !up_only(code, a, b) {
            return false;
        }
    }
    true
}

/// The players' life totals, poison and library sizes, and the size of the stack and of every zone together, as code and
/// amount in turn (for `LoopWatch::far`).
fn players_amounts(s: &GameState) -> Vec<i32> {
    let mut out = Vec::new();
    for (i, p) in s.players.iter().enumerate() {
        let library = s.zone(Zone::library(PlayerId(i as u8))).len() as i32;
        for (what, n) in [(amount::LIFE, p.life), (amount::POISON, p.poison as i32), (amount::LIBRARY, library)] {
            out.push(code(what, i) as i32);
            out.push(n);
        }
    }
    // A chain of mandatory resolutions that ends - a stack or a board shrinking - is going somewhere too.
    let objects: usize = s.zones.iter().map(|z| z.len()).sum();
    for (what, n) in [(amount::STACK, s.stack.len()), (amount::OBJECTS, objects)] {
        out.push(code(what, 0) as i32);
        out.push(n as i32);
    }
    out
}

/// A position: its fingerprint, its shape (the fingerprint with every amount left out), and the amounts.
pub struct Position {
    pub exact: u64,
    pub shape: u64,
    /// Each amount by its code (`code`), sorted; an amount of zero on an object is left out.
    pub amounts: Vec<(u32, i32)>,
}

/// The objects of a position numbered by where they are (see the module).
struct Canon<'a> {
    s: &'a GameState,
    number: Vec<u32>,
}

impl<'a> Canon<'a> {
    fn new(s: &'a GameState) -> Canon<'a> {
        let mut number = vec![u32::MAX; s.objects.len()];
        let mut n = 0u32;
        for zone in &s.zones {
            for &id in zone {
                number[id.index()] = n;
                n += 1;
            }
        }
        Canon { s, number }
    }

    /// What an object that left still says (CR 608.2h: an ability of it reads it as it last was): its row and its
    /// last-known information, without the ids and timestamps that differ round a loop.
    fn departed(&self, id: ObjectId) -> u64 {
        let s = self.s;
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
    }

    /// An object as the position names it: by where it is, or, for one that left, by what it became, how many moves ago, and
    /// what it still says.
    fn id(&self, id: ObjectId) -> u64 {
        if id == ObjectId::NONE {
            return u64::MAX;
        }
        let mut at = id;
        for moves in 0..16u64 {
            if at.index() >= self.number.len() {
                break;
            }
            if self.number[at.index()] != u32::MAX {
                let here = moves << 32 | self.number[at.index()] as u64;
                return if moves == 0 { here } else { here ^ self.departed(id).rotate_left(17) };
            }
            match self.s.obj(at).became {
                Some(next) => at = next,
                None => break,
            }
        }
        self.departed(id)
    }

    fn target(&self, t: Target) -> (u8, u64) {
        match t {
            Target::None => (0, 0),
            Target::Player(p) => (1, p.0 as u64),
            Target::Object(id) => (2, self.id(id)),
            Target::Ability(serial) => (3, self.s.stack.iter().position(|i| i.serial == serial).map_or(u64::MAX, |at| at as u64)),
        }
    }

    fn members(&self, span: cantrip_state::Span) -> Vec<u64> {
        let start = span.start as usize;
        self.s.members.get(start..start + span.len as usize).unwrap_or(&[]).iter().map(|&id| self.id(id)).collect()
    }
}

/// An answer as the watch knows it: its objects named by the position (`Canon`), so that the same answer to the same
/// question is the same however round a loop the objects have been made anew.
fn answer_hash(canon: &Canon, answer: &Answer) -> u64 {
    let mut h = DefaultHasher::new();
    match answer {
        Answer::Act(a) => (1u8, a.kind, a.source.map(|id| canon.id(id)), a.index, a.mana, a.restriction).hash(&mut h),
        Answer::Targets(ts) => (2u8, ts.iter().map(|&t| canon.target(t)).collect::<Vec<_>>()).hash(&mut h),
        Answer::Cards(ids) => (3u8, ids.iter().map(|&id| canon.id(id)).collect::<Vec<_>>()).hash(&mut h),
        other => (0u8, other).hash(&mut h),
    }
    h.finish()
}

/// The answer that takes a priority option, as `answer_hash` knows it.
fn option_hash(canon: &Canon, option: &ActionOption) -> u64 {
    answer_hash(canon, &Answer::Act(Action::of(option)))
}

/// A question as the watch keeps it: the shape of the position it is asked in, who is asked, where in the rules (the frame,
/// and how many answers it has had), and what is asked, its objects named by the position.
fn question_key(canon: &Canon, shape: &u64, player: PlayerId, frame: usize, cursor: usize, kind: &RequestKind) -> u64 {
    let mut h = DefaultHasher::new();
    (shape, player, frame, cursor, std::mem::discriminant(kind)).hash(&mut h);
    let ids = |ids: &[ObjectId]| ids.iter().map(|&id| canon.id(id)).collect::<Vec<_>>();
    match kind {
        RequestKind::ChooseTarget { source, candidates, least, count, label, .. } => {
            (canon.id(*source), candidates.iter().map(|&t| canon.target(t)).collect::<Vec<_>>(), least, count, label).hash(&mut h)
        }
        RequestKind::ChooseCards { source, eligible, count, .. } => (canon.id(*source), ids(eligible), count).hash(&mut h),
        RequestKind::ChoosePlayer { source, candidates, .. } => (canon.id(*source), candidates).hash(&mut h),
        RequestKind::ChooseModes { source, eligible, count, .. } => (canon.id(*source), eligible, count).hash(&mut h),
        RequestKind::May { source, .. } => canon.id(*source).hash(&mut h),
        _ => {}
    }
    h.finish()
}

/// The position up to the identity of objects and the order of timestamps (see the module).
#[cfg(test)]
fn fingerprint(cx: &Cx) -> u64 {
    position(cx.s(), &Canon::new(cx.s())).exact
}

/// Two hashes taken together: the fingerprint, and the shape, which leaves the amounts out.
struct Pair {
    exact: DefaultHasher,
    shape: DefaultHasher,
}

impl Pair {
    fn both<T: Hash + ?Sized>(&mut self, t: &T) {
        t.hash(&mut self.exact);
        t.hash(&mut self.shape);
    }
}

fn position(s: &GameState, canon: &Canon) -> Position {
    use amount::*;
    // Timestamps by their order (CR 613.7).
    let mut stamps: Vec<u32> = s.zones.iter().flatten().map(|&id| s.obj(id).timestamp).collect();
    stamps.extend(s.effects.iter().map(|e| e.timestamp));
    stamps.extend(s.counter_stamps.iter().map(|c| c.timestamp));
    stamps.sort_unstable();
    stamps.dedup();
    let rank = |t: u32| stamps.binary_search(&t).unwrap_or(usize::MAX);

    let mut h = Pair { exact: DefaultHasher::new(), shape: DefaultHasher::new() };
    let mut amounts = Vec::new();
    h.both(&s.turn);
    for (i, p) in s.players.iter().enumerate() {
        p.hash(&mut h.exact);
        let mut shape = *p;
        (shape.life, shape.poison, shape.energy, shape.experience) = (0, 0, 0, 0);
        (shape.pool, shape.lasting) = Default::default();
        shape.hash(&mut h.shape);
        for (what, n) in [(LIFE, p.life), (POISON, p.poison as i32), (ENERGY, p.energy as i32), (EXPERIENCE, p.experience as i32)] {
            amounts.push((code(what, i), n));
        }
        for m in 0..6 {
            amounts.push((code(POOL + m as u32, i), p.pool.0[m] as i32));
            amounts.push((code(LASTING + m as u32, i), p.lasting.0[m] as i32));
        }
    }
    for (i, r) in s.restricted_mana.iter().enumerate() {
        r.hash(&mut h.exact);
        (r.player, r.restriction).hash(&mut h.shape);
        for m in 0..6 {
            amounts.push((code(RESTRICTED + m as u32, i), r.pool.0[m] as i32));
        }
    }
    for (i, m) in s.mana_from.iter().enumerate() {
        let what = (m.player, m.restriction, canon.id(m.source), m.lasting);
        (what, m.pool).hash(&mut h.exact);
        what.hash(&mut h.shape);
        for k in 0..6 {
            amounts.push((code(FROM + k as u32, i), m.pool.0[k] as i32));
        }
    }
    for c in &s.counter_stamps {
        h.both(&(canon.id(c.object), c.kind, rank(c.timestamp)));
    }
    let mut n = 0usize;
    for zone in &s.zones {
        h.both(&zone.len());
        for &id in zone {
            let mut o = *s.obj(id);
            let refs = (
                o.attached_to.map(|id| canon.id(id)),
                o.entered_with.map(|id| canon.id(id)),
                o.prepared_from.map(|id| canon.id(id)),
                o.exiled.map(|e| (canon.id(e.by), e.from, e.until_leaves, e.encoded)),
            );
            o.timestamp = rank(o.timestamp) as u32;
            o.became = None;
            o.attached_to = None;
            o.entered_with = None;
            o.prepared_from = None;
            o.exiled = None;
            (o, refs).hash(&mut h.exact);
            if o.damage != 0 {
                amounts.push((code(DAMAGE, n), o.damage as i32));
            }
            for (k, &count) in o.counters.iter().enumerate() {
                if count != 0 {
                    amounts.push((code(COUNTER + k as u32, n), count as i32));
                }
            }
            o.damage = 0;
            o.counters = [0; CounterKind::COUNT];
            (o, refs).hash(&mut h.shape);
            n += 1;
        }
    }
    for item in &s.stack {
        let mut i = *item;
        let refs = (canon.id(i.object), i.targets().iter().map(|&t| canon.target(t)).collect::<Vec<_>>(), canon.target(i.subject), canon.members(i.members));
        i.object = ObjectId::NONE;
        i.targets = [Target::None; cantrip_state::MAX_TARGETS];
        i.subject = Target::None;
        i.members = cantrip_state::Span::default();
        i.serial = 0;
        i.since = 0;
        h.both(&(i, refs));
    }
    for e in &s.effects {
        h.both(&(canon.id(e.source), e.controller, rank(e.timestamp), canon.id(e.affected), e.kind, e.duration));
    }
    for p in &s.permissions {
        h.both(&(canon.id(p.card), p.player, p.grant, p.turn));
    }
    for t in &s.triggers {
        h.both(&(canon.id(t.source), t.ability, t.controller, canon.target(t.subject), t.data, canon.members(t.members), t.count));
    }
    for d in &s.delayed {
        h.both(&(d.when, canon.id(d.source), d.controller, d.ability, canon.target(d.subject), canon.members(d.members)));
    }
    for a in &s.attackers {
        h.both(&(canon.id(a.id), canon.target(a.target), a.blocked));
    }
    for b in &s.blocks {
        h.both(&(canon.id(b.blocker), canon.id(b.attacker)));
    }
    h.both(&s.extra_turns);
    h.both(&s.rng);
    for f in &s.agenda {
        h.both(&f.kind);
    }
    amounts.sort_unstable();
    Position { exact: h.exact.finish(), shape: h.shape.finish(), amounts }
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
            let _ = asked(&mut cx(&mut game), PlayerId(0), two, true, false);
            for p in [PlayerId(0), PlayerId(1)] {
                game.state.update_turn(|t| t.priority = p);
                drawn |= at_priority(&mut cx(&mut game), p, Vec::new()).is_none();
            }
        }
        assert!(!drawn && game.state.outcome == Outcome::InProgress);
        // Each player passes in turn, round after round.
        'round: for _ in 0..WATCH_AFTER {
            flicker(&mut game);
            for p in [PlayerId(0), PlayerId(1)] {
                game.state.update_turn(|t| t.priority = p);
                if at_priority(&mut cx(&mut game), p, Vec::new()).is_none() {
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
            assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        }
        assert!(game.state.loop_watch.forced == 0 && game.state.loop_watch.seen.is_empty());
    }

    /// Both players pass once round the loop; whether the game was drawn on the way.
    fn round(game: &mut Game, between: impl Fn(&mut Game)) -> bool {
        flicker(game);
        let mut drawn = false;
        for p in [PlayerId(0), PlayerId(1)] {
            between(game);
            crate::loops::after_frame(&mut game.state);
            game.state.update_turn(|t| t.priority = p);
            drawn |= at_priority(&mut cx(game), p, Vec::new()).is_none();
        }
        drawn
    }

    /// CR 104.4b, 603.2h: a forced loop whose rounds differ only in their history is the same position round after round,
    /// but a question the history is asked between them ("is this the sixth time this turn?") may answer otherwise on a later
    /// round, so the rounds can't be found the same. A read of the history clears the positions seen; without one, the same
    /// rounds are a draw.
    #[test]
    fn a_loop_whose_history_is_read_between_rounds_is_not_found_by_its_positions() {
        cantrip_api::history::take_reads();
        let mut game = game();
        for _ in 0..WATCH_AFTER * 4 {
            let read = |_: &mut Game| {
                let records: [Record; 0] = [];
                let _ = History::new(&records, 1).this_turn().len();
            };
            assert!(!round(&mut game, read), "the history was asked: the rounds could differ");
        }
        assert_eq!(game.state.outcome, Outcome::InProgress);
        let drawn = (0..4).any(|_| round(&mut game, |_| {}));
        assert!(drawn && game.state.outcome == Outcome::Draw, "the same rounds with nothing asking the past are a draw");
    }

    /// A question about the past settled for the rest of the turn is no read that tells rounds apart; one that is not yet
    /// settled is.
    #[test]
    fn a_settled_question_about_the_past_is_not_counted_as_a_read() {
        let mut game = game();
        cantrip_api::history::take_reads();
        let c = cx(&mut game);
        assert!(settled_read(&c, |_| true, |&done| done));
        assert_eq!(cantrip_api::history::take_reads(), 0);
        assert!(!settled_read(&c, |_| false, |&done| done));
        assert_eq!(cantrip_api::history::take_reads(), 1);
    }

    /// CR 704.5a-c, 704.5f-g, 704.5i: an amount going the way that ends a game or a permanent is progress towards the end of
    /// a loop; one going the other way, or nowhere, is not.
    #[test]
    fn only_amounts_going_the_way_nothing_ends_by_are_going_nowhere() {
        use amount::*;
        let life = code(LIFE, 0);
        let poison = code(POISON, 1);
        let damage = code(DAMAGE, 3);
        let plus = code(COUNTER + CounterKind::ALL.iter().position(|&k| k == CounterKind::PlusOne).unwrap() as u32, 3);
        let minus = code(COUNTER + CounterKind::ALL.iter().position(|&k| k == CounterKind::MinusOne).unwrap() as u32, 3);
        assert!(gone_nowhere(&[(life, 20)], &[(life, 20)]));
        assert!(gone_nowhere(&[(life, 20)], &[(life, 21)]), "life gained");
        assert!(!gone_nowhere(&[(life, 20)], &[(life, 19)]), "life lost");
        assert!(!gone_nowhere(&[(poison, 0)], &[(poison, 1)]), "poison");
        assert!(!gone_nowhere(&[], &[(damage, 1)]), "damage marked");
        assert!(gone_nowhere(&[], &[(plus, 1)]), "a +1/+1 counter put on");
        assert!(!gone_nowhere(&[(plus, 2)], &[(plus, 1)]), "a counter removed");
        assert!(!gone_nowhere(&[(plus, 1)], &[]), "the last counter removed");
        assert!(!gone_nowhere(&[], &[(minus, 1)]), "a -1/-1 counter put on");
    }

    /// CR 104.4b: a forced loop that never comes back to a position - a life more each time round - is still endless, and a
    /// draw once it has gone on as long as the rules let a run of mandatory actions go; but not while a life total is going
    /// down, which ends it (CR 704.5a).
    #[test]
    fn a_forced_run_that_grows_for_ever_is_a_draw_unless_a_life_total_is_falling() {
        let mut game = game();
        let mut rounds = 0;
        while !round(&mut game, |g| g.state.update_player(PlayerId(1), |p| p.life += 1)) {
            rounds += 1;
            assert!(rounds < FORCED_LIMIT, "no end");
        }
        assert_eq!(game.state.outcome, Outcome::Draw);
        assert!(rounds * 2 >= FORCED_LIMIT - 2, "not before the limit: {rounds} rounds");

        let mut game = self::game();
        game.state.update_player(PlayerId(1), |p| p.life = 1 << 20);
        for _ in 0..FORCED_LIMIT {
            assert!(!round(&mut game, |g| g.state.update_player(PlayerId(1), |p| p.life -= 1)));
        }
    }

    /// CR 732.3: a player made to choose otherwise is not offered, at the same question, a card they chose there before -
    /// while there is another to choose - and a "may" they took is declined; one they declined is still asked.
    #[test]
    fn a_player_made_to_choose_otherwise_is_offered_only_what_they_did_not_choose() {
        let game = game();
        let canon = Canon::new(&game.state);
        let a = game.state.battlefield()[0];
        let (one, two) = (Target::Object(a), Target::Player(PlayerId(1)));
        let choose = RequestKind::ChooseTarget {
            source: a,
            purpose: Purpose::Targets,
            candidates: vec![one, two],
            optional: false,
            least: 1,
            count: 1,
            any_number: false,
            label: "",
            slot_player: None,
            total: None,
        };
        let given = [answer_hash(&canon, &Answer::Targets(vec![one]))];
        match otherwise(&canon, choose.clone(), &given, true) {
            Asked::Ask { kind: RequestKind::ChooseTarget { candidates, .. }, .. } => assert_eq!(candidates, vec![two]),
            _ => panic!("still a choice of target"),
        }
        let both = [given[0], answer_hash(&canon, &Answer::Targets(vec![two]))];
        match otherwise(&canon, choose, &both, true) {
            Asked::Ask { kind: RequestKind::ChooseTarget { candidates, .. }, .. } => assert_eq!(candidates.len(), 2, "nothing left untried"),
            _ => panic!("still a choice of target"),
        }
        let may = RequestKind::May { source: a, purpose: Purpose::Optional };
        assert!(matches!(otherwise(&canon, may.clone(), &[answer_hash(&canon, &Answer::Yes)], true), Asked::Answered(Answer::No)));
        assert!(matches!(otherwise(&canon, may, &[answer_hash(&canon, &Answer::No)], true), Asked::Ask { .. }));
    }
}
