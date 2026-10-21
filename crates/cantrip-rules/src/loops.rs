//! Loops (CR 104.4b, 732). "If a game ... somehow enters a 'loop' of mandatory actions, repeating a sequence of events with
//! no way to stop, the game is a draw. Loops that contain an optional action don't result in a draw" (CR 104.4b; 732.4,
//! 732.5). The rules here find the loops that are a draw, make a player stop a loop several players keep going by turns
//! (CR 732.3), and leave every other loop a player keeps going by choice to go on.
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
//! **A loop its players keep going** (CR 732). Once a step has a great deal in it (`BUSY_STEP` events, which a step outside
//! a loop rarely has), every question asked in it is kept by its exact position, with the answers given to it there, and
//! every player's choices to act are counted (an action at priority, a "may" taken, anything chosen of several but passing,
//! declining or choosing nothing). A priority asked again in the same position, with nothing asking the history in between,
//! is a round of a loop; if two or more players chose to act in it, it is a fragmented loop, and the active player (or, if
//! they are not involved, the first involved in turn order) must make a different game choice (CR 732.3): from then on
//! they are not offered, at a question in a position they have answered before, what they answered there - an action at
//! priority, a target, card, mode or player chosen alone - and a "may" they took there is declined for them, until they
//! answer such a question otherwise. A pass or a decline is never refused (CR 732.5 lets a player be held to an action a
//! loop object calls for, but no rule requires it). A loop only one player keeps going, or one whose players only pass and
//! decline, goes on for as long as they choose: the engine supports no shortcut (CR 732.2), every round is played by hand,
//! and nothing here cuts it off or caps it - a combination may need hundreds of rounds. A seat nobody drives stops by
//! itself (the C# client's `LoopGuard`), a person by the match clock, and a game that reaches `MAX_GAME_EVENTS` events is a
//! draw (`Game::run`; `docs/rules-exceptions.md`, RX-1).
//!
//! **What the watch keeps.** Each question of a busy step once (`GameState::loop_keys`, found by its key in a hashed index,
//! journalled an entry at a time), with the answers given to it and, for a priority, the round it was last asked in: a loop
//! played by hand for any number of rounds costs it nothing more per round. Past `KEPT` questions it starts over, so a loop
//! is found again within two of its rounds.
//!
//! A normal game pays a comparison per priority and a count per read of the history: the fingerprinting starts only after
//! a long run of priorities in which nobody could do anything, or in a step far longer than a game outside a loop has.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use cantrip_api::history::{note_read, take_reads};
use cantrip_api::*;
use cantrip_state::{GameState, LastKnown, LoopRound, LoopWatch};

use crate::Cx;

/// Priorities given with something on the stack and no real choice in between before the positions are fingerprinted:
/// enough that a few triggers resolving with nothing to respond with never pay for it.
pub const WATCH_AFTER: u32 = 16;

/// Events in a step before its questions are kept with their positions: far more than a step outside a loop has.
pub const BUSY_STEP: u32 = 1024;

/// The most questions the watch keeps in a step: far past any loop's round. Past it the watch starts over.
const KEPT: usize = 1 << 18;

/// The most positions a run of mandatory actions is fingerprinted at before the run is started over (its positions are
/// compared again from there).
const SEEN: usize = 1024;

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
    /// Ask it, as it now is (with what a player who must choose otherwise answered it before taken out), `key` its place in
    /// the watch when it is kept.
    Ask { kind: RequestKind, key: Option<u64> },
    /// A "may" the player must now decline (CR 732.3), answered without asking.
    Answered(Answer),
}

/// A question other than priority is about to be asked (or asked again, on a re-run): one with more than one legal answer
/// clears the watch of mandatory actions; in a step being watched it is kept with its position, and a player who must make
/// a different choice (CR 732.3) is not offered what they answered it before. Priority is `at_priority`'s. A question
/// that can be answered without asking (`may_answer`: not a staged one) may be answered here; it is then given as any
/// answer is, and `replayed` with the frame. A question `replayed` (a frame played out again with the answers it had) is
/// kept but never changed, its answer given to it as it was first asked: a player who must choose otherwise and chose so
/// there no longer must.
pub fn asked(cx: &mut Cx, player: PlayerId, kind: RequestKind, may_answer: bool, replayed: Option<&Answer>) -> Asked {
    if matches!(kind, RequestKind::Priority { .. }) || forced(&kind) {
        return Asked::Ask { kind, key: None };
    }
    clear_forced(cx);
    if !busy(cx) {
        return Asked::Ask { kind, key: None };
    }
    let key = {
        let canon = Canon::new(cx.s());
        question_key(&canon, position(cx.s(), &canon), player, cx.frame, cx.cursor, &kind)
    };
    let at = entry(cx.st(), key);
    if !designated(cx.s(), player) {
        return Asked::Ask { kind, key: Some(key) };
    }
    let canon = Canon::new(cx.s());
    if let Some(answer) = replayed {
        let given = &cx.s().loop_keys[at].given;
        let this = answer_hash(&canon, answer);
        if !given.is_empty() && !given.contains(&this) {
            let watch = LoopWatch { designated: cx.s().loop_watch.designated & !bit(player), ..cx.s().loop_watch.clone() };
            cx.st().set_loop_watch(watch);
        }
        return Asked::Ask { kind, key: Some(key) };
    }
    match otherwise(&canon, kind, &cx.s().loop_keys[at].given, may_answer) {
        Asked::Ask { kind, .. } => Asked::Ask { kind, key: Some(key) },
        // Declining is the different choice (CR 732.3); and noted again if the frame is played out again with it.
        answered => {
            let watch = LoopWatch { designated: cx.s().loop_watch.designated & !bit(player), ..cx.s().loop_watch.clone() };
            cx.st().set_loop_watch(watch);
            answered
        }
    }
}

/// The question with the answers in `given` taken out, where what is left is still a question with an answer: a target,
/// card, mode or player chosen alone; and a "may" taken before is declined without asking, if `may_answer`. A player made
/// to choose otherwise is only ever made to choose another of the same, or to decline: declining is never refused. Any
/// other question is asked as it is.
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

/// Whether giving `answer` to a question of `kind` is choosing to act (CR 732.3): anything but passing, declining, or
/// choosing nothing, to a question with more than one legal answer.
fn acts(kind: &RequestKind, answer: &Answer) -> bool {
    !forced(kind)
        && match answer {
            Answer::Pass | Answer::No | Answer::Cancel => false,
            Answer::Targets(t) => !t.is_empty(),
            Answer::Cards(c) => !c.is_empty(),
            Answer::Attack(a) => !a.is_empty(),
            Answer::Block(b) => !b.is_empty(),
            _ => true,
        }
}

/// What an answer about to be given to a kept question (request `id`) comes to for the watch: the question's key, the
/// answer as the watch knows it, and whether giving it is choosing to act (`acts`). Read before the answer is played out,
/// in the position it was asked in. Priority is `at_priority`'s, as it is played out again.
pub fn answering(s: &GameState, id: u32, request: &Request, answer: &Answer) -> Option<(u64, u64, bool)> {
    let (asked, key) = s.loop_watch.asking?;
    if asked != id || matches!(request.kind, RequestKind::Priority { .. }) {
        return None;
    }
    let canon = Canon::new(s);
    Some((key, answer_hash(&canon, answer), acts(&request.kind, answer)))
}

/// The answer noted by `answering` is given, by `player`: it is kept with its question, a player who chose to act is
/// counted (`LoopWatch::acts`), and a player who had to choose otherwise and has (CR 732.3) no longer must.
pub fn answered(s: &mut GameState, player: PlayerId, (key, answer, acted): (u64, u64, bool)) {
    let at = entry(s, key);
    note(s, at, player, answer, acted);
}

/// `player` gives `answer` to the kept question at `at` (see `answered`).
fn note(s: &mut GameState, at: usize, player: PlayerId, answer: u64, acted: bool) {
    let given = &s.loop_keys[at].given;
    let differs = !given.is_empty() && !given.contains(&answer);
    let mut watch = s.loop_watch.clone();
    if differs {
        watch.designated &= !bit(player);
    }
    if acted && let Some(n) = watch.acts.get_mut(player.0 as usize) {
        *n = n.saturating_add(1);
    }
    if watch != s.loop_watch {
        s.set_loop_watch(watch);
    }
    s.give_loop_answer(at, answer);
}

/// A question with request id `id` and watch key `key` is now waiting.
pub fn waiting(s: &mut GameState, id: u32, key: Option<u64>) {
    if let Some(key) = key {
        let watch = LoopWatch { asking: Some((id, key)), ..s.loop_watch.clone() };
        s.set_loop_watch(watch);
    }
}

fn bit(p: PlayerId) -> u8 {
    1u8.checked_shl(p.0 as u32).unwrap_or(0)
}

fn designated(s: &GameState, p: PlayerId) -> bool {
    s.loop_watch.designated & bit(p) != 0
}

/// The step the game is in, as the watch names it.
fn step_id(s: &GameState) -> u32 {
    s.turn.number << 8 | s.turn.step_raw as u32
}

/// Whether this step, with the history `log` events long, is being watched for loops of choices (`BUSY_STEP`).
fn busy_at(s: &GameState, log: usize) -> bool {
    let w = &s.loop_watch;
    w.step == step_id(s) && (log as u32).saturating_sub(w.step_log) > BUSY_STEP
}

fn busy(cx: &Cx) -> bool {
    busy_at(cx.s(), cx.g.log.len())
}

/// The kept question `key`'s entry, made if it has none. Past `KEPT` of them the watch starts over.
fn entry(s: &mut GameState, key: u64) -> usize {
    if let Some(at) = s.loop_key_entry(key, KEPT) {
        return at;
    }
    s.loop_keys_clear();
    s.loop_key_entry(key, KEPT).unwrap_or_default()
}

/// The run of mandatory actions is broken: a real choice.
fn clear_forced(cx: &mut Cx) {
    let w = &cx.s().loop_watch;
    if w.forced != 0 || !w.seen.is_empty() {
        let mut watch = w.clone();
        watch.forced = 0;
        watch.seen.clear();
        cx.st().set_loop_watch(watch);
    }
}

/// A frame of the rules has run, the history now `log` events long: if it asked the history anything, the positions seen
/// before can't be compared with any after (see the module).
pub fn after_frame(s: &mut GameState, log: usize) {
    if take_reads() > 0 && (!s.loop_watch.seen.is_empty() || busy_at(s, log)) {
        let mut watch = s.loop_watch.clone();
        watch.seen.clear();
        watch.reads = watch.reads.saturating_add(1);
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
        let watch = LoopWatch {
            step,
            step_log: cx.g.log.len() as u32,
            reads: 0,
            acts: [0; 8],
            designated: 0,
            asking: None,
            ..cx.s().loop_watch.clone()
        };
        cx.st().set_loop_watch(watch);
        cx.st().loop_keys_clear();
    }
    let mut seen_position = None;
    if mandatory(cx, player, &options, &mut seen_position) {
        return None;
    }
    if !busy(cx) {
        return Some(options);
    }
    // The answer this priority had, when it is played out again with it (`Game::answer`): what the watch notes of it is
    // noted here, where it stays.
    let replayed = cx.s().agenda.get(cx.frame).and_then(|f| f.answers.get(cx.cursor)).cloned();
    let canon = Canon::new(cx.s());
    let here = seen_position.unwrap_or_else(|| position(cx.s(), &canon));
    let key = question_key(&canon, here, player, cx.frame, cx.cursor, &RequestKind::Priority { options: Vec::new() });
    drop(canon);
    let at = entry(cx.st(), key);
    let mut watch = cx.s().loop_watch.clone();
    // The same position, with nothing asking the history since it was last here: a round of a loop. Two players or more
    // chose to act in it: a fragmented loop (CR 732.3), which one of them must now stop.
    let round = cx.s().loop_keys[at].round;
    if round.asked && round.reads == watch.reads {
        let involved = |p: PlayerId| watch.acts.get(p.0 as usize).zip(round.acts.get(p.0 as usize)).is_some_and(|(now, then)| now > then);
        let order: Vec<PlayerId> = cx.s().turn_order_from(cx.s().turn.active).filter(|&p| involved(p)).collect();
        if order.len() >= 2 {
            watch.designated |= bit(order[0]);
        }
    }
    cx.st().set_loop_round(at, LoopRound { asked: true, reads: watch.reads, acts: watch.acts });
    if watch != cx.s().loop_watch {
        cx.st().set_loop_watch(watch);
    }
    if designated(cx.s(), player) {
        let canon = Canon::new(cx.s());
        let given = &cx.s().loop_keys[at].given;
        options.retain(|o| !given.contains(&option_hash(&canon, o)));
    }
    if let Some(answer) = replayed {
        let this = answer_hash(&Canon::new(cx.s()), &answer);
        note(cx.st(), at, player, this, matches!(answer, Answer::Act(_)));
    }
    Some(options)
}

/// The watch of a run of mandatory actions, at a priority: whether it has come round a loop and the game is a draw. The
/// fingerprint of the position, if it took one, is left in `seen_position`.
fn mandatory(cx: &mut Cx, player: PlayerId, options: &[ActionOption], seen_position: &mut Option<u64>) -> bool {
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
    let here = position(cx.s(), &Canon::new(cx.s()));
    // The same priority asked again (a run redone with its answers) is the same question, not a loop.
    if watch.seen.last() == Some(&here) {
        *seen_position = Some(here);
        return false;
    }
    if read {
        watch.seen.clear();
        if busy(cx) {
            watch.reads = watch.reads.saturating_add(1);
        }
    }
    if watch.seen.contains(&here) {
        draw(cx);
        return true;
    }
    if watch.seen.len() >= SEEN {
        watch.seen.clear();
    }
    watch.seen.push(here);
    cx.st().set_loop_watch(watch);
    *seen_position = Some(here);
    false
}

// --- Positions -----------------------------------------------------------------------------------------------------------

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
        o.last_known = None;
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

/// The answer an option at priority would be, as the watch knows it.
fn option_hash(canon: &Canon, option: &ActionOption) -> u64 {
    answer_hash(canon, &Answer::Act(Action::of(option)))
}

/// A question as the watch keeps it: the position it is asked in (`position`), who is asked, where in the rules (the frame,
/// and how many answers it has had), and what is asked, its objects named by the position.
fn question_key(canon: &Canon, position: u64, player: PlayerId, frame: usize, cursor: usize, kind: &RequestKind) -> u64 {
    let mut h = DefaultHasher::new();
    (position, player, frame, cursor, std::mem::discriminant(kind)).hash(&mut h);
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
    position(cx.s(), &Canon::new(cx.s()))
}

/// The position's fingerprint (see the module).
fn position(s: &GameState, canon: &Canon) -> u64 {
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
        (m.player, m.restriction, canon.id(m.source), m.lasting, m.pool).hash(&mut h);
    }
    for c in &s.counter_stamps {
        (canon.id(c.object), c.kind, rank(c.timestamp)).hash(&mut h);
    }
    for zone in &s.zones {
        zone.len().hash(&mut h);
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
            (o, refs).hash(&mut h);
        }
    }
    for item in &s.stack {
        let mut i = *item;
        let refs = (
            canon.id(i.object),
            s.targets(item).iter().map(|&t| canon.target(t)).collect::<Vec<_>>(),
            s.spec_of(item).to_vec(),
            s.shares(item).to_vec(),
            canon.target(i.subject),
            canon.members(i.members),
        );
        i.object = ObjectId::NONE;
        // The targets, their slots and shares are in `refs`, read through the arena: not where the arena keeps them.
        i.inline_targets = [Target::None; cantrip_state::INLINE_TARGETS];
        i.inline_spec_of = [0; cantrip_state::INLINE_TARGETS];
        i.shares = 0;
        i.spill = 0;
        i.subject = Target::None;
        i.members = cantrip_state::Span::default();
        i.serial = 0;
        i.since = 0;
        (i, refs).hash(&mut h);
    }
    for e in &s.effects {
        (canon.id(e.source), e.controller, rank(e.timestamp), canon.id(e.affected), e.kind, e.duration).hash(&mut h);
    }
    for p in &s.permissions {
        // The permanent a "for as long as you control" permission is about, by where it is, as every object is.
        let source = p.grant.expiry.controlled().map(|id| canon.id(id));
        let grant = PlayGrant { expiry: if source.is_some() { Expiry::WhileControls(ObjectId::NONE) } else { p.grant.expiry }, ..p.grant };
        (canon.id(p.card), p.player, grant, source, p.turn).hash(&mut h);
    }
    for t in &s.triggers {
        (canon.id(t.source), t.ability, t.controller, canon.target(t.subject), t.data, canon.members(t.members), t.count).hash(&mut h);
    }
    for d in &s.delayed {
        (d.when, canon.id(d.source), d.controller, d.ability, canon.target(d.subject), canon.members(d.members)).hash(&mut h);
    }
    for a in &s.attackers {
        (canon.id(a.id), canon.target(a.target), a.blocked).hash(&mut h);
    }
    for b in &s.blocks {
        (canon.id(b.blocker), canon.id(b.attacker)).hash(&mut h);
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
        Cx { g: game, frame: usize::MAX, cursor: 0, mana_source: None, entering: None, lineage: Vec::new(), asked_at: None, doer: None }
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
            let _ = asked(&mut cx(&mut game), PlayerId(0), two, true, None);
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
            let log = game.log.len();
            crate::loops::after_frame(&mut game.state, log);
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

    /// CR 732: a question asked in the middle of a run that goes on past it (a replacement's) is registered with the loop watch,
    /// and winding the run back to where it was asked (`Cx::asked_at`) keeps that registration.
    #[test]
    fn a_question_a_run_goes_on_past_is_still_watched_once_the_run_is_wound_back() {
        // A step busy enough to be watched for loops of choices (`BUSY_STEP`), with a frame running.
        let mut game = game();
        game.state.frame_push(cantrip_state::Frame::new(cantrip_state::FrameKind::Resolve));
        let mut watch = game.state.loop_watch.clone();
        watch.step = step_id(&game.state);
        watch.step_log = 0;
        game.state.set_loop_watch(watch);
        let source = game.state.battlefield()[0];
        let mut cx = Cx { frame: 0, ..cx(&mut game) };
        for _ in 0..=BUSY_STEP {
            cx.record(Event::Drew { player: PlayerId(0), card: source });
        }
        // A question asked mid-run (a replacement's "may"), and the run goes on past it before it is wound back (`Game::run`).
        let asked = cx.ask_core(PlayerId(0), RequestKind::May { source, purpose: Purpose::Optional }, false);
        assert!(matches!(asked, Err(crate::Stop::Suspend)));
        let (back_to, _) = cx.asked_at.expect("where it was asked");
        cx.st().update_player(PlayerId(0), |p| p.life -= 3);
        cx.g.state.revert(back_to);
        let id = game.state.agenda[0].pending.as_ref().expect("the question").id;
        assert_eq!(game.state.player(PlayerId(0)).life, 20, "what the run did after the question is undone");
        assert!(matches!(game.state.loop_watch.asking, Some((asked, _)) if asked == id), "the watch still waits for its answer");
    }

    /// A busy step (`BUSY_STEP` events in it) with the Loop Test's ability on the stack.
    fn busy_game() -> Game {
        let mut game = game();
        let mut watch = game.state.loop_watch.clone();
        watch.step = step_id(&game.state);
        watch.step_log = 0;
        game.state.set_loop_watch(watch);
        let source = game.state.battlefield()[0];
        let mut cx = cx(&mut game);
        for _ in 0..=BUSY_STEP {
            cx.record(Event::Drew { player: PlayerId(0), card: source });
        }
        game
    }

    /// A kept question of a player's choice between two players (each `cursor` its own), answered acting or not.
    fn choose(game: &mut Game, player: PlayerId, cursor: usize, acting: bool) {
        let two = RequestKind::ChoosePlayer { source: ObjectId(0), candidates: vec![PlayerId(0), PlayerId(1)], verb: "", amount: 0 };
        let Asked::Ask { key: Some(key), .. } = asked(&mut Cx { cursor, ..cx(game) }, player, two, true, None) else { panic!("kept") };
        let answer = answer_hash(&Canon::new(&game.state), &Answer::Player(PlayerId(1)));
        answered(&mut game.state, player, (key, answer, acting));
    }

    /// CR 732.3: a priority asked again in the same position is a round of a loop. With only the active player choosing to
    /// act in it, it goes on; once both players have, it is a fragmented loop, and the active player must choose otherwise.
    #[test]
    fn a_loop_two_players_act_in_makes_the_active_player_choose_otherwise() {
        let mut game = busy_game();
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        choose(&mut game, PlayerId(0), 1, true);
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        assert_eq!(game.state.loop_watch.designated, 0, "one player kept the loop going");
        choose(&mut game, PlayerId(1), 2, false);
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        assert_eq!(game.state.loop_watch.designated, 0, "declining is no action");
        choose(&mut game, PlayerId(1), 2, true);
        choose(&mut game, PlayerId(0), 1, true);
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        assert_eq!(game.state.loop_watch.designated, 1, "the active player");
        assert_eq!(game.state.outcome, Outcome::InProgress, "never a draw");
        // Asked the choice they made before, they are offered the other player alone; choosing it, they chose otherwise.
        let two = RequestKind::ChoosePlayer { source: ObjectId(0), candidates: vec![PlayerId(0), PlayerId(1)], verb: "", amount: 0 };
        let other = answer_hash(&Canon::new(&game.state), &Answer::Player(PlayerId(0)));
        let Asked::Ask { kind, key: Some(key) } = asked(&mut Cx { cursor: 1, ..cx(&mut game) }, PlayerId(0), two, true, None) else { panic!("kept") };
        assert!(matches!(kind, RequestKind::ChoosePlayer { candidates, .. } if candidates == vec![PlayerId(0)]));
        answered(&mut game.state, PlayerId(0), (key, other, true));
        assert_eq!(game.state.loop_watch.designated, 0, "a different choice ends it");
    }

    /// A history read between two visits of a position makes them different rounds (see the module): no fragmented loop.
    #[test]
    fn a_position_with_the_history_read_since_is_no_round_of_a_loop() {
        cantrip_api::history::take_reads();
        let mut game = busy_game();
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        choose(&mut game, PlayerId(0), 1, true);
        choose(&mut game, PlayerId(1), 2, true);
        cantrip_api::history::note_read();
        let log = game.log.len();
        after_frame(&mut game.state, log);
        assert!(at_priority(&mut cx(&mut game), PlayerId(0), Vec::new()).is_some());
        assert_eq!(game.state.loop_watch.designated, 0);
    }
}
