//! The core: rules authority and timing (docs/engine-design.md, "Layers: core, shared mechanics, cards").
//!
//! The engine is a step function. [`Game::run`] works the agenda until a frame asks something or the game ends;
//! [`Game::answer`] gives the answer and runs on. Between the two, the game is plain data: copy it with `clone`.

pub mod tree;
mod actions;
mod attach;
mod day_night;
mod designations;
mod layers;
mod ops;
mod replace;
mod resolve;
mod settle;
mod speed;
mod static_rules;
mod turn;
mod validate;

use cantrip_api::*;
use cantrip_state::{Frame, FrameKind, Mark, Marks, Rng};

pub use actions::{activation_costs, cast_costs, describe, exile_cast_texts, granted_cast_cost, life_for_cast, granted_cast_total, special_action_costs, sacrifices_for_reduction};
pub use validate::{check as check_answer, distribution_problem, name_problem};
pub use layers::{chars, live, summoning_sick};

/// One game: the state, what has happened, and the card table it is played with.
pub struct Game {
    pub state: GameState,
    /// The event history (`cantrip_api::history`): every event since the game began, in order, with its serial,
    /// turn, step and cause. Cut back only with the journal, when a run is redone.
    pub log: Vec<Record>,
    /// The game log as the search tree, and the node the working state is at (`tree`).
    pub tree: tree::Tree,
    pub at: tree::NodeId,
    pub cards: &'static CardTable,
    /// Check that every rollback restores the state exactly (slow; for tests).
    pub verify: bool,
}

/// A whole copy of a game, its path's journal and the tree with it: one that can go on, or move through the tree,
/// exactly as the original would.
impl Clone for Game {
    fn clone(&self) -> Game {
        Game {
            state: self.state.clone_with_journal(),
            log: self.log.clone(),
            tree: self.tree.clone(),
            at: self.at,
            cards: self.cards,
            verify: self.verify,
        }
    }
}

/// What dealing a game needs.
#[derive(Clone, Debug, Default)]
pub struct Setup {
    /// Each seat's deck, top of the library first.
    pub decks: Vec<Vec<CardId>>,
    /// Each seat's sideboard: the cards it owns outside the game (CR 100.4, 400.11a, 108.3b), which effects that bring a
    /// card into the game from outside it choose among. A seat with no entry has none.
    pub sideboards: Vec<Vec<CardId>>,
    /// In a Limited game, the basic land cards the event supplies: a Limited sideboard holds any number of each (Magic
    /// Tournament Rules; North Wind Avatar's ruling), so each seat always has one of each outside the game. Empty otherwise.
    pub basic_lands_outside: Vec<CardId>,
    pub seed: u64,
    /// Shuffle the libraries and play the real start of a game (CR 103): who goes first, mulligans. Without it the
    /// libraries stay in the order given, the first seat goes first and both opening hands are dealt and kept, as a
    /// scenario wants.
    pub shuffle: bool,
    /// With `shuffle` off: still ask who goes first and offer mulligans (a scenario's `[Opening]`), this seat choosing.
    pub opening_chooser: Option<PlayerId>,
}

/// An answer the engine refused. The game is as it was, the request still outstanding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rejected(pub String);

/// The card code stopped: a question it is waiting on, or a cast given up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Stop {
    Suspend,
    /// The player gave up the spell or ability being announced (CR 733.1): everything the frame did is undone.
    Cancel,
}

impl From<Suspend> for Stop {
    fn from(_: Suspend) -> Stop {
        Stop::Suspend
    }
}

pub(crate) type R<T> = Result<T, Stop>;

/// How a frame's run ended.
pub(crate) enum Flow {
    /// Its work is done: take it off the agenda.
    Done,
    /// It stays, to run again from its start with no answers (a priority round, between passes).
    Again,
}

impl Game {
    pub fn new(cards: &'static CardTable, setup: &Setup) -> Game {
        let players = setup.decks.len();
        let mut state = GameState::new(players, setup.seed);
        state.outside_supply = setup.basic_lands_outside.clone();
        for (seat, deck) in setup.decks.iter().enumerate() {
            let p = PlayerId(seat as u8);
            let library = deck.iter().map(|&card| (card, Zone::library(p)));
            let sideboard = setup.sideboards.get(seat).into_iter().flatten().chain(&setup.basic_lands_outside).map(|&card| (card, Zone::sideboard(p)));
            for (card, zone) in library.chain(sideboard).collect::<Vec<_>>() {
                let id = state.push_obj(Obj {
                    card,
                    owner: p,
                    controller: p,
                    zone,
                    token: false,
                    tapped: false,
                    controlled_since: 0,
                    damage: 0,
                    deathtouched: false,
                    counters: [0; CounterKind::COUNT],
                    timestamp: 0,
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
                state.zone_push(zone, id);
            }
        }

        let real = setup.shuffle || setup.opening_chooser.is_some();
        if setup.shuffle {
            for p in state.seats() {
                turn::shuffle(&mut state, Zone::library(p));
            }
        }
        let chooser = match (setup.opening_chooser, setup.shuffle) {
            (Some(p), _) => p,
            (None, true) => PlayerId(state.random(players as u32) as u8),
            (None, false) => PlayerId(0),
        };
        state.update_turn(|t| {
            t.active = chooser;
            t.starting = chooser;
            t.priority = chooser;
        });

        let mut game = Game { state, log: Vec::new(), tree: tree::Tree::new(), at: tree::NodeId::ROOT, cards, verify: false };
        if real {
            game.state.frame_push(Frame::new(FrameKind::Opening));
        } else {
            // A scenario's deal: seven each, in turn order, kept.
            for p in game.state.apnap().collect::<Vec<_>>() {
                let mut cx = Cx { g: &mut game, frame: usize::MAX, cursor: 0, mana_source: None };
                for _ in 0..7 {
                    ops::draw(&mut cx, p);
                }
                game.state.update_player(p, |pl| pl.kept = true);
            }
            game.log.clear();
            // CR 103.6a: a card in a hand that may begin the game on the battlefield is asked before the first turn.
            let begins = game.state.seats().any(|p| {
                game.state.zone(Zone::hand(p)).iter().any(|&id| cards.get(game.state.obj(id).card).opening_hand.is_some())
            });
            if begins {
                game.state.frame_push(Frame::new(FrameKind::Opening));
            }
        }
        game.state.journal.entries.clear();
        game.state.journal.base = 0;
        // The journal of the path is the tree's record of its nodes' writes (`tree`): it is never dropped.
        game.state.journal.holds = 1;
        game
    }

    /// The question the game is waiting on, if any.
    pub fn pending(&self) -> Option<&Request> {
        self.state.agenda.last().and_then(|f| f.pending.as_ref())
    }

    pub fn is_over(&self) -> bool {
        self.state.is_over()
    }

    /// Runs until the game asks something or ends.
    pub fn run(&mut self) {
        loop {
            if self.state.is_over() {
                return;
            }
            if self.state.agenda.is_empty() {
                turn::next_step(self);
                continue;
            }
            let at = self.state.agenda.len() - 1;
            if self.state.agenda[at].pending.is_some() {
                return;
            }

            let mark = self.state.mark();
            let log_mark = self.log.len() as u32;
            {
                let frame = &mut self.state.agenda[at];
                frame.mark = mark.0;
                frame.log_mark = log_mark;
            }
            let before = if self.verify { Some(self.state.clone()) } else { None };

            let kind = self.state.agenda[at].kind;
            let flow = {
                let mut cx = Cx { g: self, frame: at, cursor: 0, mana_source: None };
                match kind {
                    FrameKind::Opening => turn::opening(&mut cx),
                    FrameKind::BeginStep => turn::begin_step(&mut cx),
                    FrameKind::Priority => turn::priority(&mut cx),
                    FrameKind::Settle => settle::settle(&mut cx),
                    FrameKind::Act { player } => actions::act(&mut cx, player),
                    FrameKind::Resolve => resolve::resolve(&mut cx),
                }
            };

            // A question was asked and the run went on regardless: its work is thrown away and redone when it is answered.
            let flow = match flow {
                Ok(_) if self.state.agenda.get(at).is_some_and(|f| f.pending.is_some()) => {
                    // The run went on past a question it could not stop at (a replacement effect asking which
                    // applies first): it is redone with the answer, so what it did is undone now, and the question
                    // is asked of the position it began from.
                    let pending = self.state.agenda[at].pending.clone();
                    self.state.revert(mark);
                    self.log.truncate(log_mark as usize);
                    self.state.update_frame(at, |f| f.pending = pending);
                    Err(Stop::Suspend)
                }
                other => other,
            };
            match flow {
                Ok(Flow::Done) => {
                    self.state.frame_remove(at);
                }
                Ok(Flow::Again) => {
                    if !self.state.agenda[at].answers.is_empty() {
                        self.state.update_frame(at, |f| f.answers.clear());
                    }
                }
                Err(Stop::Suspend) => {
                    if let Some(before) = before {
                        // Rolling back the run must give back exactly the state it started from.
                        let mut probe = self.state.clone_with_journal();
                        probe.rollback(mark);
                        assert!(probe == before, "a rollback did not restore the state: some write was not journalled");
                    }
                    return;
                }
                Err(Stop::Cancel) => {
                    // Everything the announcement did is undone (CR 733.1), and the frame goes.
                    self.state.revert(mark);
                    self.log.truncate(log_mark as usize);
                    self.state.frame_remove(at);
                }
            }
            self.forget();
        }
    }

    /// Answers the outstanding request and runs on.
    pub fn answer(&mut self, request: u32, player: PlayerId, answer: Answer) -> Result<(), Rejected> {
        let Some(at) = self.state.agenda.len().checked_sub(1) else {
            return Err(Rejected("nothing is being asked".into()));
        };
        let Some(pending) = self.state.agenda[at].pending.clone() else {
            return Err(Rejected("nothing is being asked".into()));
        };
        if pending.id != request {
            return Err(Rejected(format!("request {request} is not outstanding (request {} is)", pending.id)));
        }
        if pending.player != player {
            return Err(Rejected(format!("request {request} is not for {player:?}")));
        }
        // An answer given here before leads to the node it made: the working state goes there, nothing is replayed.
        if let Some(child) = self.tree.child(self.at, request, player, &answer) {
            self.goto(child);
            return Ok(());
        }
        validate::check(self, &pending, &answer)?;
        let here = self.marks();
        self.tree.close(self.at, here);
        // A run answered without stages is redone from its start, so the events it had recorded are thrown away and
        // the child's begin where the run did; the journal only grows (a revert appends the inverse entries).
        let child_start = if self.state.agenda[at].staged { here } else { (here.0, self.state.agenda[at].log_mark as usize) };
        let child = self.tree.add(self.at, (request, player, answer.clone()), child_start);

        let frame = &self.state.agenda[at];
        if frame.staged {
            let next = frame.next_stage;
            self.state.update_frame(at, |f| {
                f.pending = None;
                f.answers = vec![answer];
                f.stage = next;
            });
        } else {
            let mark = Mark(frame.mark);
            let log_mark = frame.log_mark as usize;
            let mut answers = frame.answers.clone();
            answers.push(answer);
            self.state.revert(mark);
            self.log.truncate(log_mark);
            self.state.update_frame(at, |f| f.answers = answers);
        }
        self.run();
        self.at = child;
        if self.verify {
            // Up to the node the answer was given at and back: the working state must come back exactly.
            let (state, log) = (self.state.clone(), self.log.clone());
            let parent = self.tree.parent(child).expect("a parent");
            self.goto(parent);
            self.goto(child);
            assert!(self.state == state && self.log == log, "moving through the game tree did not restore the position");
        }
        Ok(())
    }

    /// Where the working state's path ends now: the journal mark and the history's length.
    fn marks(&self) -> (Mark, usize) {
        (self.state.mark(), self.log.len())
    }

    /// Moves the working state to `target` through the tree (`tree`): undoes up to the common ancestor and redoes down
    /// to it. No state is copied.
    pub fn goto(&mut self, target: tree::NodeId) {
        if target == self.at {
            return;
        }
        let here = self.marks();
        self.tree.close(self.at, here);
        let from = self.tree.path(self.at);
        let to = self.tree.path(target);
        let shared = from.iter().zip(&to).take_while(|(a, b)| a == b).count();
        for &node in from[shared..].iter().rev() {
            let (mark, events_from) = self.tree.start(node);
            let (undos, redos) = self.state.step_back(mark);
            let events = self.log.split_off(events_from);
            self.tree.put_away(node, undos, redos, events);
        }
        for &node in &to[shared..] {
            let (undos, redos, events) = self.tree.take_back(node);
            self.state.step_forward(undos, redos);
            self.log.extend(events);
        }
        self.at = target;
    }

    /// Drops journal entries nothing can roll back to: everything before the oldest run still waiting, when no
    /// checkpoint is held.
    fn forget(&mut self) {
        if self.state.journal.holds > 0 {
            return;
        }
        let oldest = self.state.agenda.iter().filter(|f| f.pending.is_some() || !f.answers.is_empty()).map(|f| f.mark).min();
        let keep = oldest.unwrap_or_else(|| self.state.mark().0);
        self.state.forget_before(Mark(keep));
    }

    /// The node the game is at, for a search to come back to with [`Game::goto`].
    pub fn checkpoint(&self) -> tree::NodeId {
        self.at
    }

    /// Back to a node a search left from: [`Game::goto`].
    pub fn rollback(&mut self, to: tree::NodeId) {
        self.goto(to);
    }

    /// An object's characteristics.
    pub fn chars(&self, id: ObjectId) -> Chars {
        layers::chars(self.cards, &self.state, &self.log, id)
    }

    /// The name an object shows (its copiable values' name).
    pub fn name(&self, id: ObjectId) -> &'static str {
        self.chars(id).name
    }

    /// Whether `player` can't gain life right now (CR 119.7): an effect or a static ability on the battlefield says so.
    pub fn cant_gain_life(&self, player: PlayerId) -> bool {
        static_rules::cant_gain_life(self, self.cards, player)
    }

    /// Whether `creature` may attack as though it didn't have defender right now (CR 702.3b, 609.4): an effect or a static
    /// ability on the battlefield says so. Whether it has defender at all is asked of its characteristics.
    pub fn may_attack_despite_defender(&self, creature: ObjectId) -> bool {
        static_rules::may_attack_despite_defender(self, self.cards, creature)
    }

    /// Whether the static abilities on the battlefield let `creature` attack (CR 508.1c): "this creature can't attack unless
    /// ..." (Patchwork Beastie). A resolved "can't attack" effect is asked of the effects.
    pub fn attack_rules_allow(&self, creature: ObjectId) -> bool {
        static_rules::attack_rules_allow(self, self.cards, creature)
    }

    /// The sources of the ordinary mana in `player`'s pool that a rule tells apart from other mana (CR 106.3): mana the player
    /// spends only by choosing it, which the pool shows as that source's.
    pub fn mana_told_apart(&self, player: PlayerId) -> Vec<ObjectId> {
        static_rules::mana_told_apart(self, self.cards, player)
    }

    /// Whether `player` may look at the top card of their library any time (CR 401.2), as a permanent they control says.
    pub fn may_look_at_library_top(&self, player: PlayerId) -> bool {
        actions::may_look_at_library_top(self, self.cards, player)
    }
}

/// The view of the game a frame's run has: what card code is handed as `&mut dyn Game`.
pub(crate) struct Cx<'a> {
    pub g: &'a mut Game,
    /// The frame being run.
    pub frame: usize,
    /// How many of its answers have been taken.
    pub cursor: usize,
    /// The source of the mana ability running (CR 106.3, 605): what produces the mana it adds. None outside one, where the
    /// mana a resolving spell or ability adds is that spell's or the ability's source's.
    pub mana_source: Option<ObjectId>,
}

impl<'a> Cx<'a> {
    pub fn s(&self) -> &GameState {
        &self.g.state
    }

    pub fn st(&mut self) -> &mut GameState {
        &mut self.g.state
    }

    pub fn cards(&self) -> &'static CardTable {
        self.g.cards
    }

    pub fn ask_core(&mut self, player: PlayerId, kind: RequestKind, may_cancel: bool) -> R<Answer> {
        let frame = &self.g.state.agenda[self.frame];
        if self.cursor < frame.answers.len() {
            let answer = frame.answers[self.cursor].clone();
            self.cursor += 1;
            return Ok(answer);
        }
        // A question is already waiting: an operation that cannot carry the suspension out (a zone change asking
        // which replacement applies first) went on with the first, and the run is abandoned and replayed once the
        // answer is in (see `Game::run`). No second question replaces it.
        if frame.pending.is_some() {
            return Err(Stop::Suspend);
        }
        let id = self.g.state.request_id();
        let at = self.frame;
        self.g.state.update_frame(at, |f| f.pending = Some(Request { id, player, kind, may_cancel }));
        Err(Stop::Suspend)
    }

    /// A reflexive triggered ability triggers (CR 603.12): `count` is the number its "that many" counts, if it has one.
    fn reflexive(&mut self, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target, count: Option<u16>) {
        // A reflexive triggered ability is an ordinary one (CR 603.12): its intervening "if" must hold for it to trigger at all
        // (CR 603.4), not only as it resolves.
        if let AbilityDef::Triggered(TriggeredDef { condition: Some(condition), .. }) = self.g.cards.ability(ability)
            && !condition(&*self, &Asker { source, controller })
        {
            return;
        }
        self.g.state.trigger_push(cantrip_state::PendingTrigger {
            source,
            ability,
            controller,
            subject,
            data: [0; 2],
            members: cantrip_state::Span::default(),
            count,
            since: None,
        });
    }

    /// An event happened: the triggers it causes are found, then it is recorded (so a trigger or replacement
    /// deciding about it sees the history as it was just before it, `cantrip_api::history`).
    pub fn emit(&mut self, event: Event) {
        settle::collect_triggers(self, &event, None);
        self.record(event);
    }

    /// Events that happened together (CR 603.2c): triggers are collected over the whole batch, so a "one or more"
    /// ability triggers once. `before` is the battlefield before them, for look-back abilities.
    pub fn emit_batch(&mut self, events: &[Event], before: Option<&[ObjectId]>) {
        settle::collect_batch(self, events, before);
        for &e in events {
            self.record(e);
        }
    }

    /// Records an event in the history without looking for triggers: the records no ability triggers on.
    pub fn record(&mut self, event: Event) {
        let turn = self.g.state.turn;
        let cause = self.cause();
        let serial = self.g.log.len() as u32;
        self.g.log.push(Record { serial, turn: turn.number, step: turn.step(), active: turn.active, cause, event });
    }

    /// The controller of the spell or ability whose effect is running (CR 101.2): none for a player's own action, the
    /// turn's, or the rules'.
    pub fn effect_controller(&self) -> Option<PlayerId> {
        match self.cause() {
            Cause::Spell(_) | Cause::Ability { .. } => self.g.state.stack.last().map(|i| i.controller),
            _ => None,
        }
    }

    /// The player doing what the running frame does (CR 122.6a): who a player's action is, or who controls the spell or
    /// ability that is resolving. None for the rules' own doing.
    pub fn actor(&self) -> Option<PlayerId> {
        match self.cause() {
            Cause::Player(p) => Some(p),
            Cause::Spell(_) | Cause::Ability { .. } => self.g.state.stack.last().map(|i| i.controller),
            _ => None,
        }
    }

    /// What the frame being run is doing, as an event's cause.
    pub(crate) fn cause(&self) -> Cause {
        let Some(frame) = self.g.state.agenda.get(self.frame) else { return Cause::Rules };
        match frame.kind {
            FrameKind::Act { player } => Cause::Player(player),
            FrameKind::BeginStep | FrameKind::Opening => Cause::Turn,
            FrameKind::Resolve => match self.resolving() {
                Some(item) if item.kind == StackKind::Spell => Cause::Spell(item.object),
                Some(item) => Cause::Ability { source: item.object, ability: item.ability },
                None => Cause::Rules,
            },
            _ => Cause::Rules,
        }
    }

    /// The source of mana added now (CR 106.3): the mana ability's source while one runs, else the spell resolving or the
    /// source of the ability resolving; none for a player's own mana ability (CR 605.1a) or anything else.
    pub(crate) fn mana_source(&self) -> ObjectId {
        self.mana_source.or_else(|| self.resolving().map(|i| i.object)).unwrap_or(ObjectId::NONE)
    }

    /// The spell or ability resolving, when the frame being run is its resolution: the top of the stack.
    pub(crate) fn resolving(&self) -> Option<&StackItem> {
        let frame = self.g.state.agenda.get(self.frame)?;
        if frame.kind != FrameKind::Resolve {
            return None;
        }
        self.g.state.stack.last()
    }

}

impl Query for Game {
    fn state(&self) -> &GameState {
        &self.state
    }

    fn events(&self) -> &[Record] {
        &self.log
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.cards.id_of(def)
    }

    fn card(&self, id: CardId) -> &'static CardDef {
        self.cards.get(id)
    }

    fn chars(&self, id: ObjectId) -> Chars {
        layers::chars(self.cards, &self.state, &self.log, id)
    }

    fn lost_all_abilities(&self, id: ObjectId) -> bool {
        layers::lost_all_abilities(self.cards, &self.state, &self.log, id)
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        layers::can_target(self.cards, &self.state, &self.log, asker, target)
    }
}

impl Query for Cx<'_> {
    fn state(&self) -> &GameState {
        &self.g.state
    }

    fn events(&self) -> &[Record] {
        &self.g.log
    }

    fn card_id(&self, def: &'static CardDef) -> CardId {
        self.g.cards.id_of(def)
    }

    fn card(&self, id: CardId) -> &'static CardDef {
        self.g.cards.get(id)
    }

    fn chars(&self, id: ObjectId) -> Chars {
        layers::chars(self.g.cards, &self.g.state, &self.g.log, id)
    }

    fn lost_all_abilities(&self, id: ObjectId) -> bool {
        layers::lost_all_abilities(self.g.cards, &self.g.state, &self.g.log, id)
    }

    fn can_target(&self, asker: &Asker, target: Target) -> bool {
        layers::can_target(self.g.cards, &self.g.state, &self.g.log, asker, target)
    }
}

impl cantrip_api::Game for Cx<'_> {
    fn ask(&mut self, player: PlayerId, kind: RequestKind) -> Result<Answer, Suspend> {
        // CR 101.2: a discard or sacrifice an opponent's spell or ability can't cause is not offered as a choice.
        if let RequestKind::ChooseCards { purpose: purpose @ (Purpose::Discard | Purpose::Sacrifice), .. } = &kind
            && let Some(causer) = self.effect_controller()
            && !cantrip_api::may_cause_discard_or_sacrifice(&*self, causer, player, *purpose == Purpose::Sacrifice)
        {
            return Ok(Answer::Cards(Vec::new()));
        }
        self.ask_core(player, kind, false).map_err(|_| Suspend)
    }

    fn stage(&self) -> u16 {
        self.g.state.agenda[self.frame].stage
    }

    fn staged_answer(&self) -> Option<Answer> {
        let frame = &self.g.state.agenda[self.frame];
        if frame.staged { frame.answers.first().cloned() } else { None }
    }

    fn scratch(&self) -> [u32; 4] {
        self.g.state.agenda[self.frame].scratch
    }

    fn set_scratch(&mut self, scratch: [u32; 4]) {
        let at = self.frame;
        self.g.state.update_frame(at, |f| f.scratch = scratch);
    }

    fn ask_then(&mut self, next: u16, player: PlayerId, kind: RequestKind) -> Suspend {
        let id = self.g.state.request_id();
        let at = self.frame;
        self.g.state.update_frame(at, |f| {
            f.staged = true;
            f.next_stage = next;
            f.pending = Some(Request { id, player, kind, may_cancel: false });
        });
        Suspend
    }

    fn draw(&mut self, player: PlayerId, count: u32) {
        ops::draw_cards(self, player, count);
    }

    fn discard(&mut self, card: ObjectId) {
        ops::discard(self, card);
    }

    fn discard_together(&mut self, cards: &[ObjectId]) -> Vec<ObjectId> {
        ops::discard_together(self, cards)
    }

    fn create_tokens_together(&mut self, tokens: &[(CardId, PlayerId)]) -> Result<Vec<ObjectId>, Suspend> {
        ops::create_tokens_batch(self, tokens, false, None, &[]).map_err(|_| Suspend)
    }

    fn mill(&mut self, player: PlayerId, count: u32) -> Vec<ObjectId> {
        ops::mill(self, player, count)
    }

    fn mill_together(&mut self, mills: &[(PlayerId, u32)]) -> Vec<ObjectId> {
        ops::mill_together(self, mills)
    }

    fn damage(&mut self, source: ObjectId, target: Target, amount: u16) -> u16 {
        ops::damage(self, source, target, amount, false)
    }

    fn damage_together(&mut self, damage: &[(ObjectId, Target, u16)]) -> Vec<u16> {
        ops::damage_together(self, damage)
    }

    fn each_opponent_loses_life(&mut self, player: PlayerId, amount: i32) -> i32 {
        ops::each_opponent_loses_life(self, player, amount)
    }

    fn destroy_all(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId> {
        ops::destroy_together(self, permanents, ZoneReason::Destroy)
    }

    fn destroy_all_no_regen(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId> {
        ops::destroy_together(self, permanents, ZoneReason::DestroyNoRegen)
    }

    fn sacrifice(&mut self, permanents: &[ObjectId]) -> Vec<ObjectId> {
        ops::sacrifice(self, permanents)
    }

    fn return_to_hand(&mut self, objects: &[ObjectId]) -> Vec<ObjectId> {
        ops::return_to_hand(self, objects)
    }

    fn bring_into_hand(&mut self, cards: &[ObjectId]) -> Vec<ObjectId> {
        ops::bring_into_hand(self, cards)
    }

    fn put_into_library(&mut self, objects: &[ObjectId], place: LibraryPlace) -> Vec<ObjectId> {
        ops::put_into_library(self, objects, place)
    }

    fn exile_all(&mut self, objects: &[ObjectId]) -> Vec<ObjectId> {
        ops::exile_all(self, objects)
    }

    fn put_into_graveyard(&mut self, objects: &[ObjectId]) -> Vec<ObjectId> {
        ops::put_into_graveyard(self, objects)
    }

    fn move_to(&mut self, object: ObjectId, to: Zone) -> ObjectId {
        ops::move_to(self, object, to)
    }

    fn exile_with(&mut self, cards: &[ObjectId], by: ObjectId, until_leaves: bool) -> Vec<ObjectId> {
        ops::exile_with(self, cards, by, until_leaves)
    }

    fn link_exiled(&mut self, cards: &[ObjectId], by: ObjectId) {
        ops::link_exiled(self, cards, by);
    }

    fn exile_face_down(&mut self, cards: &[ObjectId], by: Option<ObjectId>) -> Vec<ObjectId> {
        ops::exile_face_down(self, cards, by)
    }

    fn encode(&mut self, card: ObjectId, creature: ObjectId) -> Option<ObjectId> {
        ops::encode(self, card, creature)
    }

    fn look_at(&mut self, player: PlayerId, cards: &[ObjectId]) {
        ops::look_at(self, player, cards);
    }

    fn put_onto_battlefield_face_down(&mut self, object: ObjectId, controller: PlayerId) -> Result<ObjectId, Suspend> {
        ops::put_onto_battlefield_face_down(self, object, controller).map_err(|_| Suspend)
    }

    fn turn_face_up(&mut self, permanent: ObjectId, player: PlayerId) {
        ops::turn_face_up(self, permanent, player);
    }

    fn unlock_door(&mut self, permanent: ObjectId, door: u8, player: PlayerId) {
        ops::unlock_door(self, permanent, door, player);
    }

    fn remember_chosen_name(&mut self, permanent: ObjectId, name: CardId) {
        ops::remember_chosen_name(self, permanent, name);
    }

    fn move_together(&mut self, moves: &[(ObjectId, Zone)], reason: ZoneReason) -> Vec<ObjectId> {
        let moves: Vec<ops::Moving> = moves.iter().map(|&(id, to)| ops::Moving::to(id, to)).collect();
        ops::move_together(self, &moves, reason)
    }

    fn put_onto_battlefield(&mut self, object: ObjectId, controller: PlayerId, tapped: bool) -> Result<ObjectId, Suspend> {
        ops::put_onto_battlefield(self, &[(object, controller, tapped)], ZoneReason::Other)
            .map(|v| v.first().copied().unwrap_or(object))
            .map_err(|_| Suspend)
    }

    fn put_onto_battlefield_attacking(&mut self, object: ObjectId, controller: PlayerId, defender: Target) -> Result<ObjectId, Suspend> {
        ops::put_onto_battlefield_attacking(self, object, controller, defender).map_err(|_| Suspend)
    }

    fn put_onto_battlefield_with_aura(&mut self, object: ObjectId, controller: PlayerId, aura: ObjectId) -> Result<ObjectId, Suspend> {
        let new = self.put_onto_battlefield(object, controller, false)?;
        if new != object && live(&self.g.state, aura) && self.g.state.obj(aura).zone.is(ZoneKind::Battlefield) {
            self.g.state.update_obj(new, |o| o.entered_with = Some(aura));
            ops::attach(self, aura, new);
        }
        Ok(new)
    }

    fn sacrifice_and_return(&mut self, sacrificed: &[ObjectId], returned: &[(ObjectId, PlayerId)]) -> Result<(), Suspend> {
        ops::sacrifice_and_return(self, sacrificed, returned).map_err(|_| Suspend)
    }

    fn put_all_onto_battlefield_as(&mut self, objects: &[(ObjectId, PlayerId, bool)]) -> Result<Vec<ObjectId>, Suspend> {
        ops::put_onto_battlefield(self, objects, ZoneReason::Other).map_err(|_| Suspend)
    }

    fn put_all_onto_battlefield(&mut self, objects: &[(ObjectId, PlayerId)]) -> Result<Vec<ObjectId>, Suspend> {
        let all: Vec<(ObjectId, PlayerId, bool)> = objects.iter().map(|&(id, p)| (id, p, false)).collect();
        ops::put_onto_battlefield(self, &all, ZoneReason::Other).map_err(|_| Suspend)
    }

    fn put_onto_battlefield_with_counters(&mut self, object: ObjectId, controller: PlayerId, tapped: bool, counters: &[(CounterKind, i16)]) -> Result<ObjectId, Suspend> {
        ops::put_onto_battlefield_with(self, &[(object, controller, tapped)], ZoneReason::Other, counters)
            .map(|v| v.first().copied().unwrap_or(object))
            .map_err(|_| Suspend)
    }

    fn create_tokens_with_counters(&mut self, def: CardId, controller: PlayerId, count: u32, tapped: bool, counters: &[(CounterKind, i16)]) -> Result<Vec<ObjectId>, Suspend> {
        let tokens: Vec<(CardId, PlayerId)> = (0..count).map(|_| (def, controller)).collect();
        ops::create_tokens_batch(self, &tokens, tapped, None, counters).map_err(|_| Suspend)
    }

    fn create_token_copies(&mut self, of: ObjectId, controller: PlayerId, count: u32) -> Result<Vec<ObjectId>, Suspend> {
        let values = self.copiable_values(of);
        ops::create_token_copies(self, values, controller, count).map_err(|_| Suspend)
    }

    fn create_token_copies_of(&mut self, values: Copiable, controller: PlayerId, count: u32) -> Result<Vec<ObjectId>, Suspend> {
        ops::create_token_copies(self, values, controller, count).map_err(|_| Suspend)
    }

    fn put_onto_battlefield_transformed(&mut self, object: ObjectId, controller: PlayerId, tapped: bool) -> Result<ObjectId, Suspend> {
        ops::put_onto_battlefield_as(self, object, controller, tapped, true, ZoneReason::Other).map_err(|_| Suspend)
    }

    fn transform(&mut self, permanent: ObjectId) {
        ops::transform(self, permanent, None);
    }

    fn transform_by(&mut self, permanent: ObjectId, item: &StackItem) {
        ops::transform(self, permanent, Some(item));
    }

    fn create_emblem(&mut self, def: CardId, player: PlayerId) -> ObjectId {
        ops::create_emblem(self, def, player)
    }

    fn create_tokens(&mut self, def: CardId, controller: PlayerId, count: u32, tapped: bool) -> Result<Vec<ObjectId>, Suspend> {
        ops::create_tokens(self, def, controller, count, tapped).map_err(|_| Suspend)
    }

    fn copy_spell(&mut self, spell: ObjectId, copier: PlayerId, new_targets: bool) -> Result<Option<ObjectId>, Suspend> {
        ops::copy_spell(self, spell, copier, new_targets).map_err(|_| Suspend)
    }

    fn copy_ability(&mut self, ability: Target, copier: PlayerId, new_targets: bool) -> Result<Option<u32>, Suspend> {
        ops::copy_ability(self, ability, copier, new_targets).map_err(|_| Suspend)
    }

    fn delayed_trigger_about(&mut self, when: DelayedWhen, source: ObjectId, controller: PlayerId, ability: AbilityRef, objects: &[ObjectId]) {
        let members = self.g.state.members_push(objects);
        let since = self.g.log.len() as u32;
        self.g.state.delayed_push(cantrip_state::Delayed { when, source, controller, ability, subject: Target::None, members, since });
    }

    fn delayed_trigger(&mut self, when: DelayedWhen, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target) {
        let since = self.g.log.len() as u32;
        self.g.state.delayed_push(cantrip_state::Delayed { when, source, controller, ability, subject, members: cantrip_state::Span::default(), since });
    }

    fn reflexive_trigger(&mut self, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target) {
        self.reflexive(source, controller, ability, subject, None);
    }

    fn reflexive_trigger_counted(&mut self, source: ObjectId, controller: PlayerId, ability: AbilityRef, subject: Target, count: u16) {
        self.reflexive(source, controller, ability, subject, Some(count));
    }

    fn exile_on_leave(&mut self, permanent: ObjectId) {
        self.g.state.update_obj(permanent, |o| o.exile_on_leave = true);
    }

    fn shuffle(&mut self, player: PlayerId) {
        turn::shuffle(&mut self.g.state, Zone::library(player));
    }

    fn roll_die_modified(&mut self, player: PlayerId, sides: u32, modifier: i32) -> Roll {
        let natural = self.g.state.roll_die(sides);
        let result = natural as i32 + modifier;
        self.emit(Event::DieRolled { player, sides: sides as u8, natural: natural as u8, result: result as i16 });
        Roll { natural, result }
    }

    fn choose_at_random(&mut self, count: usize) -> usize {
        if count <= 1 {
            return 0;
        }
        let index = if count <= 20 { self.g.state.roll_die(count as u32) as usize - 1 } else { self.g.state.random(count as u32) as usize };
        self.emit(Event::ChosenAtRandom { count: count as u16, index: index as u16 });
        index
    }

    fn flip_coin(&mut self, player: PlayerId) -> bool {
        let won = self.g.state.roll_die(2) == 1;
        self.emit(Event::CoinFlipped { player, result: if won { CoinFlip::Won } else { CoinFlip::Lost } });
        won
    }

    fn flip_coin_for_heads(&mut self, player: PlayerId) -> bool {
        let heads = self.g.state.roll_die(2) == 1;
        self.emit(Event::CoinFlipped { player, result: if heads { CoinFlip::Heads } else { CoinFlip::Tails } });
        heads
    }

    fn random(&mut self, bound: u32) -> u32 {
        if bound <= 1 {
            return 0;
        }
        self.g.state.random(bound)
    }

    fn place_in_library(&mut self, card: ObjectId, from_top: usize) {
        let zone = self.g.state.obj(card).zone;
        if !zone.is(ZoneKind::Library) || !self.g.state.zone_remove(zone, card) {
            return;
        }
        let at = from_top.min(self.g.state.zone(zone).len());
        self.g.state.zone_insert(zone, at, card);
    }

    fn become_prepared(&mut self, permanent: ObjectId) {
        ops::become_prepared(self, permanent);
    }

    fn set_class_level(&mut self, permanent: ObjectId, level: u8) {
        if !live(&self.g.state, permanent) || !self.g.state.obj(permanent).zone.is(ZoneKind::Battlefield) {
            return;
        }
        let was = self.g.state.obj(permanent).class_level.max(1);
        self.g.state.update_obj(permanent, |o| o.class_level = level);
        if level != was {
            self.emit(Event::ClassLevel { object: permanent, level });
        }
    }

    fn arrange_library(&mut self, player: PlayerId, top: &[ObjectId], bottom: &[ObjectId]) {
        ops::arrange_library(self, player, top, bottom);
    }

    fn bottom_in_random_order(&mut self, player: PlayerId, cards: &[ObjectId]) {
        ops::bottom_in_random_order(self, player, cards);
    }

    fn add_combat_phase(&mut self) {
        self.g.state.update_turn(|t| t.extra_combats = t.extra_combats.saturating_add(1));
    }

    fn end_the_turn(&mut self) -> Result<(), Suspend> {
        turn::end_the_turn(self).map_err(|_| Suspend)
    }

    fn lose_game(&mut self, player: PlayerId) {
        ops::lose_game(self, player);
    }

    fn give_experience(&mut self, player: PlayerId, amount: u32) {
        self.g.state.update_player(player, |p| p.experience = p.experience.saturating_add(amount.min(255) as u8));
    }

    fn give_poison(&mut self, player: PlayerId, amount: u32) {
        ops::give_poison(self, player, amount);
    }

    fn become_copy(&mut self, source: ObjectId, permanent: ObjectId, values: Copiable, duration: Duration) {
        ops::become_copy(self, source, permanent, values, duration);
    }

    fn win_game(&mut self, player: PlayerId) {
        ops::win_game(self, player);
    }

    fn add_extra_turns(&mut self, player: PlayerId, count: u32) {
        if self.g.state.player(player).lost {
            return;
        }
        let mut turns = self.g.state.extra_turns.clone();
        turns.extend(std::iter::repeat(player).take(count as usize));
        self.g.state.set_extra_turns(turns);
    }

    fn gave_gift(&mut self, player: PlayerId, to: PlayerId, gift: Gift) {
        self.emit(Event::GaveGift { player, to, gift });
    }

    fn searched(&mut self, player: PlayerId, library: PlayerId) {
        self.emit(Event::Searched { player, library });
    }

    fn scried(&mut self, player: PlayerId, count: u16, looked: u16) {
        self.emit(Event::Scried { player, count, looked });
    }

    fn surveilled(&mut self, player: PlayerId, count: u16, looked: u16) {
        self.emit(Event::Surveilled { player, count, looked });
    }

    fn attach(&mut self, attachment: ObjectId, host: ObjectId) {
        ops::attach(self, attachment, host);
    }

    fn unattach(&mut self, attachment: ObjectId) {
        ops::unattach(self, attachment);
    }

    fn tap(&mut self, permanent: ObjectId) {
        ops::tap(self, permanent);
    }

    fn untap(&mut self, permanent: ObjectId) {
        ops::untap(self, permanent);
    }

    fn exert(&mut self, permanent: ObjectId) {
        ops::exert(self, permanent);
    }

    fn monstrosity(&mut self, permanent: ObjectId, counters: i16) {
        ops::monstrosity(self, permanent, counters);
    }

    fn become_saddled(&mut self, permanent: ObjectId) {
        ops::saddle(self, permanent);
    }

    fn solve(&mut self, permanent: ObjectId) {
        ops::solve(self, permanent);
    }

    fn suspect(&mut self, permanent: ObjectId) {
        ops::suspect(self, permanent);
    }

    fn fight(&mut self, first: ObjectId, second: ObjectId) {
        ops::fight(self, first, second);
    }

    fn create_tokens_attacking(&mut self, tokens: &[(CardId, PlayerId)], defender: Target) -> Result<Vec<ObjectId>, Suspend> {
        ops::create_tokens_attacking(self, tokens, defender).map_err(|_| Suspend)
    }

    fn create_tokens_worded(&mut self, tokens: &[(CardId, PlayerId)], tapped: bool, attacking: Option<Target>, words: Words) -> Result<Vec<ObjectId>, Suspend> {
        let created = ops::create_tokens_worded(self, tokens, tapped || attacking.is_some(), None, &[], words, None).map_err(|_| Suspend)?;
        if let Some(defender) = attacking {
            ops::attack_with_tokens(self, &created, defender);
        }
        Ok(created)
    }

    fn create_token_attached(&mut self, def: CardId, controller: PlayerId, host: ObjectId) -> Result<Vec<ObjectId>, Suspend> {
        ops::create_token_attached(self, def, controller, host).map_err(|_| Suspend)
    }

    fn gain_life(&mut self, player: PlayerId, amount: i32) {
        ops::change_life(self, player, amount);
    }

    fn lose_life(&mut self, player: PlayerId, amount: i32) {
        ops::change_life(self, player, -amount);
    }

    fn set_life_total(&mut self, player: PlayerId, total: i32) {
        let life = self.g.state.player(player).life;
        ops::change_life(self, player, total - life);
    }

    fn add_mana(&mut self, player: PlayerId, mana: Mana, amount: u8) {
        let source = self.mana_source();
        self.g.state.add_mana(player, mana, amount, source, None, false);
    }

    fn become_day(&mut self) {
        day_night::become_designation(self, DayNight::Day);
    }

    fn become_night(&mut self) {
        day_night::become_designation(self, DayNight::Night);
    }

    fn note_done(&mut self, source: ObjectId, ability: AbilityRef) {
        let player = self.g.state.obj(source).controller;
        self.record(Event::Done { source, ability, player });
    }

    fn add_mana_until_end_of_combat(&mut self, player: PlayerId, mana: Mana, amount: u8) {
        let source = self.mana_source();
        self.g.state.add_mana(player, mana, amount, source, None, true);
    }

    fn add_restricted_mana(&mut self, player: PlayerId, mana: Mana, amount: u8, restriction: ManaRestriction) {
        let source = self.mana_source();
        self.g.state.add_mana(player, mana, amount, source, Some(restriction), false);
    }

    fn add_counters(&mut self, object: ObjectId, kind: CounterKind, amount: i16) {
        ops::add_counters(self, object, kind, amount);
    }

    fn add_effect(&mut self, source: ObjectId, affected: ObjectId, kind: EffectKind, duration: Duration) {
        // An object that has changed zones is a new object the effect knows nothing of (CR 400.7, 611.2c), and "for as long
        // as [the source] remains on the battlefield" is over before it begins once the source has left (CR 611.2b).
        if !(live(&self.g.state, affected) || kind.outlives_affected()) {
            return;
        }
        if duration == Duration::WhileSourceOnBattlefield
            && !(live(&self.g.state, source) && self.g.state.obj(source).zone.is(ZoneKind::Battlefield))
        {
            return;
        }
        // "For as long as it has a [kind] counter on it" whose condition is already false does nothing (CR 611.2b).
        if !duration.holds_for(self.g.state.obj(affected)) {
            return;
        }
        let controller = self.g.state.obj(source).controller;
        let timestamp = self.g.state.timestamp();
        self.g.state.effect_push(ContinuousEffect { source, controller, timestamp, affected, kind, duration });
        // A static ability given to a permanent that affects other objects makes it a source of static abilities.
        if let EffectKind::AddAbility(ability) = kind
            && matches!(self.cards().ability(ability), AbilityDef::Static(s) if !s.own)
        {
            self.g.state.see_statics();
        }
        if matches!(kind, EffectKind::Control(_)) {
            ops::refresh_control(self, affected);
        }
    }

    fn add_player_effect(&mut self, controller: PlayerId, source: ObjectId, kind: EffectKind, duration: Duration) {
        debug_assert!(kind.outlives_affected(), "a player's effect is about the player, not an object");
        let timestamp = self.g.state.timestamp();
        self.g.state.effect_push(ContinuousEffect { source, controller, timestamp, affected: source, kind, duration });
    }

    fn counter(&mut self, spell: ObjectId) {
        resolve::counter(self, spell);
    }

    fn counter_to(&mut self, spell: ObjectId, to: CounterTo) {
        resolve::counter_to(self, spell, to);
    }

    fn change_controller(&mut self, spell: ObjectId, player: PlayerId) {
        ops::change_spell_controller(self, spell, player);
    }

    fn change_target(&mut self, of: Target, chooser: PlayerId) -> Result<(), Suspend> {
        ops::change_target(self, of, chooser).map_err(|_| Suspend)
    }

    fn choose_new_targets(&mut self, spell: ObjectId, chooser: PlayerId) -> Result<(), Suspend> {
        ops::choose_new_targets(self, spell, chooser).map_err(|_| Suspend)
    }

    fn remove_spell(&mut self, spell: ObjectId, to: CounterTo) {
        resolve::remove_spell(self, spell, to);
    }

    fn counter_item(&mut self, object: ObjectId, hint: usize) {
        resolve::counter_item(self, object, hint);
    }

    fn reveal(&mut self, player: PlayerId, cards: &[ObjectId]) {
        for &card in cards {
            self.emit(Event::Revealed { player, card });
        }
    }

    fn target_legal(&self, item: &StackItem, slot: usize) -> bool {
        resolve::target_legal(self.g.cards, &self.g.state, &self.g.log, item, slot)
    }

    fn may_pay(&mut self, payer: PlayerId, source: ObjectId, subject: ObjectId, cost: Cost, question: &'static Question) -> Result<bool, Suspend> {
        actions::may_pay(self, payer, source, subject, &cost, question).map_err(|_| Suspend)
    }

    fn may_pay_x(&mut self, payer: PlayerId, source: ObjectId, subject: ObjectId, cost: Cost, question: &'static Question) -> Result<Option<u16>, Suspend> {
        actions::may_pay_x(self, payer, source, subject, &cost, question).map_err(|_| Suspend)
    }

    fn could_cast_during_resolution(&self, card: ObjectId) -> bool {
        actions::could_cast_during_resolution(self, card)
    }

    fn could_play_land_during_resolution(&self, card: ObjectId) -> bool {
        actions::could_play_land_during_resolution(self, card)
    }

    fn play_land_during_resolution(&mut self, card: ObjectId) -> Result<Option<ObjectId>, Suspend> {
        actions::play_land_during_resolution(self, card).map_err(|_| Suspend)
    }

    fn cast_during_resolution(&mut self, card: ObjectId, instead: Option<ManaCost>) -> Result<bool, Suspend> {
        actions::cast_during_resolution(self, card, CastTerms { instead, ..Default::default() }).map_err(|_| Suspend)
    }

    fn cast_ways_during_resolution(&self, card: ObjectId) -> Vec<CastWay> {
        actions::cast_ways_during_resolution(self, card)
    }

    fn cast_during_resolution_as(&mut self, card: ObjectId, terms: CastTerms) -> Result<bool, Suspend> {
        actions::cast_during_resolution(self, card, terms).map_err(|_| Suspend)
    }

    fn create_copy_in_exile(&mut self, of: ObjectId, player: PlayerId) -> ObjectId {
        ops::copy_card_into(self, of, player, Zone::exile(player))
    }

    fn allow_play(&mut self, card: ObjectId, player: PlayerId, grant: PlayGrant) {
        let turn = self.g.state.turn.number;
        self.g.state.permission_push(PlayPermission { card, player, grant, turn });
    }

    fn could_cast_transformed_during_resolution(&self, card: ObjectId) -> bool {
        actions::could_cast_transformed_during_resolution(self, card)
    }

    fn cast_transformed_during_resolution(&mut self, card: ObjectId) -> Result<bool, Suspend> {
        actions::cast_transformed_during_resolution(self, card).map_err(|_| Suspend)
    }

    fn creatures_cant_block(&mut self) {
        let number = self.g.state.turn.number;
        self.g.state.update_turn(|t| t.no_blocks_turn = number);
    }

    fn become_monarch(&mut self, player: PlayerId) {
        designations::become_monarch(self, player);
    }

    fn take_initiative(&mut self, player: PlayerId) {
        designations::take_initiative(self, player);
    }

    fn venture(&mut self, player: PlayerId) -> Result<(), Suspend> {
        designations::venture(self, player).map_err(|_| Suspend)
    }

    fn increase_speed(&mut self, player: PlayerId, amount: u8) {
        speed::increase(self, player, amount);
    }

    fn damage_cant_be_prevented(&mut self) {
        let number = self.g.state.turn.number;
        self.g.state.update_turn(|t| t.no_prevention_turn = number);
    }

    fn allow_extra_loyalty_activations(&mut self, permanent: ObjectId, times: u8) {
        let player = self.g.state.obj(permanent).controller;
        let ability = AbilityRef { card: self.g.state.obj(permanent).card, index: actions::EXTRA_LOYALTY_ALLOWED };
        for _ in 0..times {
            self.record(Event::AbilityActivated { source: permanent, ability, player });
        }
    }

    fn change_energy(&mut self, player: PlayerId, delta: i32) {
        ops::change_energy(self, player, delta);
    }

    fn proliferate(&mut self, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
        ops::proliferate(self, player, source).map_err(|_| Suspend)
    }

    fn set_copy(&mut self, object: ObjectId, copy: Copiable) {
        self.g.state.update_obj(object, |o| o.copy = Some(copy));
        if self.cards().statics[copy.card.0 as usize] {
            self.g.state.see_statics();
        }
    }
}

/// The ContinuousEffect type card code builds through `add_effect`.
pub use cantrip_state::ContinuousEffect;

#[allow(dead_code)]
fn _assert_rng_is_state(_: Rng) {}
