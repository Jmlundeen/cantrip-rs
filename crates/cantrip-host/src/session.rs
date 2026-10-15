//! One hosted game: the engine, a handle space and an event stream per seat, and the one question outstanding.
//!
//! The engine asks one question at a time and its request ids are never reused, so a request id is the engine's own: an
//! answer to an old id is `StaleReply`, an answer from the wrong seat `NotYourTurnToAnswer`.
//!
//! # What is published, and when
//!
//! A frame that asks a question is re-run from its start once it is answered, and its log is cut back to where the run
//! began (`Game::answer`). So only the log up to the start of the run that is waiting is published; the entries of a
//! run in progress (a spell being announced and paid for) appear when it completes, which is also what the C# engine
//! tells: nothing about an announcement is told until the spell is paid for.

use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;

use cantrip_engine::*;
use serde_json::{Value, json};

use crate::answer::{Untranslatable, translate};
use crate::events::{Proto, Teller, Tracker};
use crate::handles::HandleSpace;
use crate::prompt::{prompt, request_objects};
use crate::protocol::*;
use crate::view::*;

/// A refusal, with the code `PhaseRulesHost` maps to its `ErrorCode`.
pub struct Rejection {
    pub code: &'static str,
    pub detail: String,
}

impl Rejection {
    pub fn new(code: &'static str, detail: impl Into<String>) -> Rejection {
        Rejection { code, detail: detail.into() }
    }
}

pub struct Session {
    game: Game,
    names: Vec<String>,
    decklists: Vec<Vec<ObjectView>>,
    /// One handle space per seat, index 0 for seat 1.
    handles: Vec<HandleSpace>,
    tracker: Tracker,
    /// How far through the engine's log everything has been published.
    published: usize,
    /// Game-wide event numbering, shared by every seat's copy of an event.
    sequence: u64,
    conceded: Vec<bool>,
    /// What happened that no engine event says: the answers to the opening's questions.
    told: Vec<Proto>,
    /// What each seat is being shown beyond its zones, as of the last step.
    extra: Vec<HashSet<ObjectId>>,
    /// The mana abilities that can still be taken back, latest last.
    undoable: Vec<Undoable>,
}

/// A mana ability a player may take back: where the game was before it, and what was told of it since. Taking it back
/// is a courtesy of the table, not a rule, so it is offered only while the activation did nothing but tap, sacrifice,
/// add mana and cost life, and nothing has happened since but more of the same: never past a trigger, a card seen or
/// another player's answer. A Black Lotus is taken back; a Lion's Eye Diamond, whose cost shows the hand, never is.
struct Undoable {
    player: PlayerId,
    /// The permanent whose ability it was, which the client shows the undo on.
    source: ObjectId,
    node: NodeId,
    tracker: Tracker,
    /// Each seat's handles: a sacrificed permanent's handle moved to the card in the graveyard, which taking it back
    /// unmakes.
    handles: Vec<HandleSpace>,
    published: usize,
    /// The sequences of the entries told of it.
    told: std::ops::Range<u64>,
}

/// The source of a mana ability `answer` activates (CR 605.1a), if it activates one that could be taken back: not one
/// whose cost discards, which shows what was in the hand.
fn mana_activation(game: &Game, answer: &Answer) -> Option<ObjectId> {
    let Answer::Act(action) = answer else { return None };
    let source = action.source.filter(|_| action.kind == ActionKind::ActivateAbility)?;
    if action.index >= PLAYER_MANA_ABILITY {
        return Some(source);
    }
    match game.chars(source).abilities().nth(action.index as usize).map(|a| game.cards.ability(a)) {
        Some(AbilityDef::Activated(d)) if d.mana.is_some() && !d.cost.discard_hand && !d.cost.discard_self && d.cost.discard.is_none() => {
            Some(source)
        }
        _ => None,
    }
}

/// Whether these entries are only what a mana ability does: taps, sacrifices, mana, and life paid or lost for it.
fn only_mana(records: &[Record]) -> bool {
    records.iter().all(|r| {
        matches!(
            r.event,
            Event::Tapped { .. }
                | Event::TappedForMana { .. }
                | Event::LifeChange { .. }
                | Event::AbilityActivated { .. }
                | Event::Resolved { .. }
                | Event::Damage { target: Target::Player(_), .. }
                | Event::ZoneChange { reason: ZoneReason::Sacrifice, .. }
        ) && !matches!(r.event, Event::ZoneChange { from, .. } if from.kind() != ZoneKind::Battlefield)
    })
}

/// Dealing a game: the seats' names and decks, and the conditions beyond the seed.
pub struct Deal {
    pub seats: Vec<(String, Vec<String>)>,
    pub seed: u64,
    pub shuffle: bool,
    pub first_player: Option<i32>,
    pub chooser: Option<i32>,
}

impl Session {
    pub fn new(deal: Deal) -> Result<Session, Rejection> {
        if deal.seats.len() < 2 {
            return Err(Rejection::new("InvalidSetup", "a game needs at least two seats"));
        }
        let cards = cards();
        let mut decks = Vec::new();
        let mut decklists = Vec::new();
        for (name, deck) in &deal.seats {
            let mut list = Vec::new();
            let mut shown = Vec::new();
            for card_name in deck {
                let id = card(card_name).ok_or_else(|| Rejection::new("InvalidSetup", format!("{name}'s deck names {card_name}, which is not a card the engine has")))?;
                let front = cards.fronts[id.0 as usize].unwrap_or(id);
                shown.push(printed(cards.get(front)));
                list.push(front);
            }
            decks.push(list);
            decklists.push(shown);
        }
        let chooser = deal.chooser.and_then(player_at);
        let first = deal.first_player.and_then(player_at);
        let opening_chooser = chooser.or(first.filter(|_| !deal.shuffle));
        let setup = Setup { decks, seed: deal.seed, shuffle: deal.shuffle, opening_chooser };
        let mut game = Game::new(cards, &setup);
        game.run();
        let seats = deal.seats.len();
        let mut session = Session {
            tracker: Tracker::new(&game),
            names: deal.seats.iter().map(|(n, _)| n.clone()).collect(),
            decklists,
            handles: (0..seats).map(|_| HandleSpace::new()).collect(),
            published: 0,
            sequence: 0,
            conceded: vec![false; seats],
            told: Vec::new(),
            extra: vec![HashSet::new(); seats],
            undoable: Vec::new(),
            game,
        };
        // A first player named outright is not asked: the question is answered as it is posed.
        if let Some(first) = first.filter(|_| chooser.is_none())
            && let Some(request) = session.game.pending().cloned()
            && matches!(request.kind, RequestKind::ChooseStartingPlayer { .. })
        {
            session.game.answer(request.id, request.player, Answer::Player(first)).map_err(|e| Rejection::new("InvalidSetup", e.0))?;
            session.told.push(Proto::StartingPlayer { chooser: request.player, player: first });
        }
        Ok(session)
    }

    pub fn decklists(&self) -> HashMap<String, Vec<ObjectView>> {
        self.decklists.iter().enumerate().map(|(i, d)| ((i + 1).to_string(), d.clone())).collect()
    }

    pub fn is_over(&self) -> bool {
        self.game.is_over()
    }

    /// The log up to where it can no longer be cut back.
    fn publishable(&self) -> usize {
        let len = self.game.log.len();
        match self.game.state.agenda.last() {
            Some(frame) if frame.pending.is_some() => (frame.log_mark as usize).min(len),
            _ => len,
        }
    }

    fn pending(&self) -> Option<Request> {
        if self.game.is_over() { None } else { self.game.pending().cloned() }
    }

    fn view_for(&mut self, p: PlayerId, mentioned: &HashSet<ObjectId>) -> PlayerView {
        let extra = self.extra[p.index()].clone();
        player_view(&self.game, p, &mut self.handles[p.index()], &self.names, &self.conceded, &extra, mentioned)
    }

    pub fn view(&mut self, seat_number: i32) -> Result<PlayerView, Rejection> {
        let p = player_at(seat_number).filter(|p| p.index() < self.names.len()).ok_or_else(|| Rejection::new("InvalidSetup", "no such seat"))?;
        Ok(self.view_for(p, &HashSet::new()))
    }

    /// Everything one advance of the game produced: the events since the last step as each seat hears them, each seat's
    /// view, and the question outstanding.
    pub fn step(&mut self) -> Value {
        let from = self.published.min(self.game.log.len());
        let bound = self.publishable().max(from);
        let records: Vec<Record> = self.game.log[from..bound].to_vec();
        self.published = bound;
        let mut protos = std::mem::take(&mut self.told);
        let next = self.sequence + 1 + protos.len() as u64;
        protos.extend(self.tracker.collect(&self.game, &records, next));

        let pending = self.pending();
        let revealed: HashSet<ObjectId> = protos
            .iter()
            .flat_map(|p| match p {
                Proto::Revealed { cards, .. } => cards.clone(),
                _ => Vec::new(),
            })
            .collect();
        let seats: Vec<PlayerId> = self.game.state.seats().collect();
        for &p in &seats {
            let mut extra = revealed.clone();
            if let Some(request) = pending.as_ref().filter(|r| r.player == p) {
                extra.extend(request_objects(request));
            }
            self.extra[p.index()] = extra;
        }

        let base = self.sequence;
        let mut views: HashMap<String, PlayerView> = HashMap::new();
        let mut seat_events: HashMap<String, Vec<Value>> = HashMap::new();
        for &p in &seats {
            let extra = self.extra[p.index()].clone();
            let mut teller = Teller { game: &self.game, viewer: p, handles: &mut self.handles[p.index()], extra: &extra, mentioned: HashSet::new() };
            let mut numbered = Vec::new();
            for (i, proto) in protos.iter().enumerate() {
                if let Some(event) = teller.tell(proto) {
                    let mut value = serde_json::to_value(&event).expect("an event serialises");
                    if let Value::Object(map) = &mut value {
                        map.insert("Sequence".into(), json!(base + 1 + i as u64));
                    }
                    numbered.push(value);
                }
            }
            let mentioned = teller.mentioned;
            seat_events.insert(seat(p).to_string(), numbered);
            views.insert(seat(p).to_string(), self.view_for(p, &mentioned));
        }
        self.sequence = base + protos.len() as u64;

        let mut requests = Vec::new();
        if let Some(request) = pending {
            let index = request.player.index();
            let undo = self.undoable.last().filter(|u| u.player == request.player).map(|u| u.source);
            match prompt(&self.game, &request, &mut self.handles[index]) {
                Ok((mut payload, may_cancel)) => {
                    if let Some(source) = undo
                        && let PromptPayload::PriorityPromptMessage(PriorityPrompt { options, .. })
                        | PromptPayload::PayCostsPromptMessage(PayCostsPrompt { options, .. }) = &mut payload
                    {
                        options.push(ActionOptionMessage::new(ActionKindView::UndoManaAbility, self.handles[index].handle(source)));
                    }
                    requests.push(json!({
                    "RequestId": request.id,
                    "Seat": seat(request.player),
                    "Prompt": payload,
                    "MayCancel": may_cancel,
                    }));
                }
                Err(why) => eprintln!("cantrip-native-host: cannot relay {:?}: {why}", request.kind),
            }
        }

        let (outcome, winning_team) = match self.game.state.outcome {
            Outcome::InProgress => (GameOutcomeView::InProgress, 0),
            Outcome::Draw => (GameOutcomeView::Draw, 0),
            Outcome::Won(p) => (GameOutcomeView::TeamWon, seat(p)),
        };
        json!({
            "Requests": requests,
            "Views": views,
            "Events": seat_events,
            "Finished": self.game.is_over(),
            "Outcome": outcome,
            "WinningTeam": winning_team,
        })
    }

    pub fn answer(&mut self, request_id: u32, seat_number: i32, payload: &AnswerPayload) -> Result<Value, Rejection> {
        let Some(request) = self.pending() else {
            return Err(Rejection::new("StaleReply", format!("request {request_id} is not outstanding")));
        };
        if request.id != request_id {
            return Err(Rejection::new("StaleReply", format!("request {request_id} is not outstanding")));
        }
        if seat(request.player) != seat_number {
            return Err(Rejection::new("NotYourTurnToAnswer", format!("request {request_id} belongs to seat {}", seat(request.player))));
        }
        if let AnswerPayload::TakeActionAnswer(TakeActionAnswer { kind: ActionKindView::UndoManaAbility, .. }) = payload {
            return self.undo(request.player);
        }
        let answer = translate(payload, &request, &self.handles[request.player.index()], &self.game).map_err(|e| match e {
            Untranslatable::UnknownHandle(d) => Rejection::new("UnknownHandle", d),
            Untranslatable::Illegal(d) => Rejection::new("IllegalAnswer", d),
        })?;

        let mana = mana_activation(&self.game, &answer);
        let (triggers, stack) = (self.game.state.triggers.len(), self.game.state.stack.len());
        let before_handles = if mana.is_some() { self.handles.clone() } else { Vec::new() };
        let (node, log_from) = (self.game.checkpoint(), self.game.log.len());
        let before = self.game.clone();
        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| self.game.answer(request.id, request.player, answer.clone())));
        match outcome {
            Ok(Ok(())) => {}
            Ok(Err(Rejected(why))) => return Err(Rejection::new("IllegalAnswer", why)),
            Err(_) => {
                self.game = before;
                self.undoable.clear();
                return Err(Rejection::new("IllegalAnswer", "the engine failed on that answer; the game is as it was"));
            }
        }
        let asks_again = self.pending().is_some_and(|next| {
            next.player == request.player && matches!(next.kind, RequestKind::Priority { .. } | RequestKind::PayCosts { .. })
        });
        match mana {
            // A trigger is something the ability did (Chromatic Star's draw), waiting or on the stack, even before anyone
            // has seen what it does.
            Some(source)
                if asks_again
                    && only_mana(&self.game.log[log_from..])
                    && self.game.state.triggers.len() <= triggers
                    && self.game.state.stack.len() <= stack =>
            {
                let first = self.sequence + 1 + self.told.len() as u64;
                self.undoable.push(Undoable {
                    player: request.player,
                    source,
                    node,
                    tracker: self.tracker.clone(),
                    handles: before_handles,
                    published: self.published,
                    told: first..first,
                });
                let step = self.step();
                if let Some(undo) = self.undoable.last_mut() {
                    undo.told.end = self.sequence + 1;
                }
                return Ok(step);
            }
            _ => self.undoable.clear(),
        }
        match (&request.kind, &answer) {
            (RequestKind::ChooseStartingPlayer { .. }, Answer::Player(first)) => {
                self.told.push(Proto::StartingPlayer { chooser: request.player, player: *first });
            }
            (RequestKind::Mulligan { mulligans }, Answer::Yes) => {
                self.told.push(Proto::Mulligan { player: request.player, count: i32::from(*mulligans) + 1 });
            }
            (RequestKind::Mulligan { mulligans }, _) => {
                self.told.push(Proto::Kept { player: request.player, cards: 7 - i32::from(*mulligans) });
            }
            _ => {}
        }
        Ok(self.step())
    }

    /// The seat gives up: it loses, and the game ends if one side is left.
    /// Takes back the latest mana ability `player` activated: the game goes back to the node it was activated at, by
    /// the tree (`Game::goto`), and what was told of it is retracted.
    fn undo(&mut self, player: PlayerId) -> Result<Value, Rejection> {
        if self.undoable.last().is_none_or(|u| u.player != player) {
            return Err(Rejection::new("IllegalAnswer", "there is no mana ability to take back"));
        }
        let undo = self.undoable.pop().expect("checked");
        self.game.goto(undo.node);
        self.tracker = undo.tracker;
        for (space, earlier) in self.handles.iter_mut().zip(undo.handles) {
            space.restore(earlier);
        }
        self.published = undo.published;
        if !undo.told.is_empty() {
            self.told.push(Proto::Retracted { sequences: undo.told.collect() });
        }
        Ok(self.step())
    }

    pub fn concede(&mut self, seat_number: i32) -> Result<Value, Rejection> {
        let p = player_at(seat_number).filter(|p| p.index() < self.names.len()).ok_or_else(|| Rejection::new("NotYourTurnToAnswer", "no such seat"))?;
        if self.game.is_over() {
            return Err(Rejection::new("IllegalAnswer", "the game is over"));
        }
        self.conceded[p.index()] = true;
        self.undoable.clear();
        self.game.state.update_player(p, |pl| pl.lost = true);
        let standing: Vec<PlayerId> = self.game.state.apnap().collect();
        match standing.as_slice() {
            [] => self.game.state.set_outcome(Outcome::Draw),
            [only] => self.game.state.set_outcome(Outcome::Won(*only)),
            _ => {}
        }
        self.told.extend(self.tracker.conceded(p));
        Ok(self.step())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays both seats from the prompts alone, as the bot of `tests/stdio_game.rs` does, until `done` has seen
    /// what it is looking for in the events some seat was told. Returns every event each seat heard.
    fn play_until(session: &mut Session, done: impl Fn(&[Value]) -> bool) -> HashMap<String, Vec<Value>> {
        let mut heard: HashMap<String, Vec<Value>> = HashMap::new();
        let mut step = session.step();
        // A spell the bot cannot pay for is given up on until the next step of the turn, not tried again at once.
        let mut allow_cast = true;
        for _ in 0..2_000 {
            for (seat, list) in step["Events"].as_object().unwrap() {
                heard.entry(seat.clone()).or_default().extend(list.as_array().unwrap().iter().cloned());
            }
            if heard.values().any(|events| done(events)) {
                return heard;
            }
            assert!(!step["Finished"].as_bool().unwrap(), "the game ended first: {heard:?}");
            let request = &step["Requests"][0];
            let prompt: PromptPayload = serde_json::from_value(request["Prompt"].clone()).unwrap();
            let seat = request["Seat"].as_i64().unwrap() as i32;
            let id = request["RequestId"].as_u64().unwrap() as u32;
            if step["Events"].as_object().unwrap().values().any(|list| !of_type(list.as_array().unwrap(), "StepBeganEvent").is_empty()) {
                allow_cast = true;
            }
            let answer = choose(&prompt, &mut allow_cast);
            step = match session.answer(id, seat, &answer) {
                Ok(step) => step,
                Err(_) => session.answer(id, seat, &prompt.default_answer()).ok().expect("the default answer is taken"),
            };
        }
        let kinds: Vec<&str> = heard["1"].iter().map(|e| e["$type"].as_str().unwrap()).collect();
        panic!("the awaited event never came; last events {:?}; pending {}", &kinds[kinds.len().saturating_sub(12)..], step["Requests"][0]["Prompt"]);
    }

    fn choose(prompt: &PromptPayload, allow_cast: &mut bool) -> AnswerPayload {
        match prompt {
            PromptPayload::PriorityPromptMessage(p) => {
                let pick = |kind: ActionKindView, quiet: bool| p.options.iter().find(|o| o.kind == kind && (!quiet || o.produces.is_empty()));
                match pick(ActionKindView::PlayLand, false).or(pick(ActionKindView::CastSpell, false).filter(|_| *allow_cast)).or(pick(ActionKindView::ActivateAbility, true)) {
                    Some(o) => AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(o)),
                    None => AnswerPayload::PassAnswer(PassAnswer {}),
                }
            }
            PromptPayload::PayCostsPromptMessage(p) => {
                let pick = |kind: ActionKindView| p.options.iter().find(|o| o.kind == kind);
                if let Some(o) = pick(ActionKindView::ConfirmCasting) {
                    return AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(o));
                }
                if let Some(o) = p.options.iter().find(|o| o.kind == ActionKindView::ActivateAbility && !o.produces.is_empty()) {
                    let mut answer = TakeActionAnswer::take(o);
                    answer.color = o.color_choices.first().copied();
                    return AnswerPayload::TakeActionAnswer(answer);
                }
                if let Some(o) = pick(ActionKindView::AssignMana) {
                    *allow_cast = false;
                    return AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(o));
                }
                *allow_cast = false;
                prompt.default_answer()
            }
            other => other.default_answer(),
        }
    }

    fn deal(first: &[&str], filler: &str, count: usize) -> Deal {
        let deck = |top: &[&str]| top.iter().map(|c| c.to_string()).chain(std::iter::repeat_n(filler.to_string(), count)).collect::<Vec<_>>();
        Deal { seats: vec![("North".into(), deck(first)), ("South".into(), deck(&[]))], seed: 1, shuffle: false, first_player: Some(1), chooser: None }
    }

    fn of_type<'a>(events: &'a [Value], kind: &str) -> Vec<&'a Value> {
        events.iter().filter(|e| e["$type"] == kind).collect()
    }

    /// A mana ability is taken back: the game returns to before it, the land untapped and its mana gone, and nothing
    /// more is offered to take back. One that finishes paying for a spell is not offered: the spell is cast.
    #[test]
    fn a_mana_ability_is_taken_back() {
        let mut session = Session::new(deal(&["Llanowar Elves"], "Forest", 19)).ok().unwrap();
        let mut step = session.step();
        let mut allow_cast = true;
        let request = |step: &Value| {
            let r = &step["Requests"][0];
            let prompt: PromptPayload = serde_json::from_value(r["Prompt"].clone()).unwrap();
            (r["RequestId"].as_u64().unwrap() as u32, r["Seat"].as_i64().unwrap() as i32, prompt)
        };
        let options = |prompt: &PromptPayload| match prompt {
            PromptPayload::PriorityPromptMessage(p) => p.options.clone(),
            PromptPayload::PayCostsPromptMessage(p) => p.options.clone(),
            _ => Vec::new(),
        };
        let mana = |prompt: &PromptPayload| options(prompt).into_iter().find(|o| o.kind == ActionKindView::ActivateAbility && !o.produces.is_empty());
        let undo = |prompt: &PromptPayload| options(prompt).into_iter().find(|o| o.kind == ActionKindView::UndoManaAbility);
        let take = |o: &ActionOptionMessage| AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(o));
        let mut taken_back = false;
        for _ in 0..2_000 {
            let (id, seat, prompt) = request(&step);
            if !taken_back
                && matches!(prompt, PromptPayload::PriorityPromptMessage(_))
                && let Some(forest) = mana(&prompt)
            {
                let tapped = session.answer(id, seat, &take(&forest)).ok().unwrap();
                let (id, seat, prompt) = request(&tapped);
                assert!(mana(&prompt).is_none(), "the Forest is tapped: {prompt:?}");
                let green = |step: &Value| step["Views"]["1"]["Players"].as_array().unwrap().iter().map(|p| p["Mana"][4].as_i64().unwrap()).sum::<i64>();
                assert_eq!(green(&tapped), 1, "the Forest's mana is in the pool");
                let back = undo(&prompt).expect("Undo is offered");
                assert_eq!(back.source, forest.source, "on the Forest");

                step = session.answer(id, seat, &take(&back)).ok().unwrap();
                let (_, _, prompt) = request(&step);
                assert_eq!(mana(&prompt).map(|o| o.source), Some(forest.source), "the Forest is untapped");
                assert!(undo(&prompt).is_none(), "nothing more to take back");
                let pools: Vec<&Value> = step["Views"]["1"]["Players"].as_array().unwrap().iter().map(|p| &p["Mana"]).collect();
                assert!(pools.iter().all(|pool| pool.as_array().unwrap().iter().all(|n| n == 0)), "its mana is gone: {pools:?}");
                taken_back = true;
                continue;
            }
            if let PromptPayload::PayCostsPromptMessage(_) = &prompt
                && let Some(forest) = mana(&prompt)
            {
                step = session.answer(id, seat, &take(&forest)).ok().unwrap();
                let (_, _, prompt) = request(&step);
                assert!(undo(&prompt).is_none(), "the spell is cast, and its mana not taken back: {prompt:?}");
                assert!(taken_back);
                return;
            }
            let answer = choose(&prompt, &mut allow_cast);
            step = match session.answer(id, seat, &answer) {
                Ok(step) => step,
                Err(_) => session.answer(id, seat, &prompt.default_answer()).ok().expect("the default answer is taken"),
            };
        }
        panic!("Llanowar Elves was never paid for");
    }

    /// Plays on until seat 1 may activate a mana ability of the permanent named `name` with priority, activates it for
    /// green, and returns that option and the step after it.
    fn activate_mana_of(session: &mut Session, name: &str) -> (ActionOptionMessage, Value) {
        let mut step = session.step();
        let mut allow_cast = true;
        for _ in 0..2_000 {
            let r = &step["Requests"][0];
            assert!(!r.is_null(), "{name}'s mana ability was never offered before the game ended");
            let prompt: PromptPayload = serde_json::from_value(r["Prompt"].clone()).unwrap();
            let (id, seat) = (r["RequestId"].as_u64().unwrap() as u32, r["Seat"].as_i64().unwrap() as i32);
            let named = |h: ViewHandle| step["Views"]["1"]["Objects"].as_array().unwrap().iter().any(|o| o["Handle"] == json!(h) && o["Name"] == name);
            if seat == 1
                && let PromptPayload::PriorityPromptMessage(p) = &prompt
                && let Some(mana) = p.options.iter().find(|o| AutoMana::is(o) && named(o.source))
            {
                let mut answer = TakeActionAnswer::take(mana);
                answer.color = mana.color_choices.contains(&ColorView::Green).then_some(ColorView::Green);
                let after = session.answer(id, seat, &AnswerPayload::TakeActionAnswer(answer)).ok().unwrap();
                return (mana.clone(), after);
            }
            let answer = match &prompt {
                PromptPayload::MayPromptMessage(_) if seat == 1 => AnswerPayload::MayChosenAnswer(MayChosenAnswer { accept: true }),
                _ => choose(&prompt, &mut allow_cast),
            };
            step = match session.answer(id, seat, &answer) {
                Ok(step) => step,
                Err(_) => session.answer(id, seat, &prompt.default_answer()).ok().expect("the default answer is taken"),
            };
        }
        panic!("{name}'s mana ability was never offered");
    }

    struct AutoMana;
    impl AutoMana {
        fn is(o: &ActionOptionMessage) -> bool {
            o.kind == ActionKindView::ActivateAbility && (!o.produces.is_empty() || o.produces_of_any_one_color > 0)
        }
    }

    /// Activates `name`'s mana ability and takes it back: offered, its green gone after, and the permanent on the
    /// battlefield under the handle it had, its ability offered again.
    fn takes_back(top: &[&str], filler: &str, name: &str) {
        let mut session = Session::new(deal(top, filler, 20 - top.len())).ok().unwrap();
        let (used, after) = activate_mana_of(&mut session, name);
        assert_eq!(green_in_pool(&after), 1, "{name}: its mana is in the pool");
        let undo = options_of(&after).into_iter().find(|o| o.kind == ActionKindView::UndoManaAbility).unwrap_or_else(|| panic!("{name}: Undo is offered"));
        assert_eq!(undo.source, used.source, "{name}: on it");

        let r = &after["Requests"][0];
        let (id, seat) = (r["RequestId"].as_u64().unwrap() as u32, r["Seat"].as_i64().unwrap() as i32);
        let undone = session.answer(id, seat, &AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(&undo))).ok().unwrap();
        assert_eq!(green_in_pool(&undone), 0, "{name}: its mana is gone");
        assert_eq!(zone_of(&undone, used.source), json!("Battlefield"), "{name}: back, under the handle it had");
        assert!(options_of(&undone).iter().any(|o| o.source == used.source && AutoMana::is(o)), "{name}: and can be used again");
        assert!(!options_of(&undone).iter().any(|o| o.kind == ActionKindView::UndoManaAbility), "{name}: nothing more to take back");
    }

    #[test]
    fn sacrificed_and_spent_mana_artifacts_are_taken_back() {
        takes_back(&["Lotus Petal"], "Forest", "Lotus Petal");
        takes_back(&["Pentad Prism"], "Forest", "Pentad Prism");
        // Generous Plunderer makes the Treasure in its controller's upkeep.
        takes_back(&["Generous Plunderer"], "Mountain", "Treasure Token");
    }

    /// Chromatic Star is not taken back: going to the graveyard, it draws a card.
    #[test]
    fn chromatic_star_is_not_taken_back() {
        let mut session = Session::new(deal(&["Chromatic Star"], "Forest", 19)).ok().unwrap();
        let (_, mut step) = activate_mana_of(&mut session, "Chromatic Star");
        // Its {1} is paid with a Forest; until then nothing has happened, and the activation may be taken back.
        let mut allow_cast = false;
        loop {
            let r = &step["Requests"][0];
            let prompt: PromptPayload = serde_json::from_value(r["Prompt"].clone()).unwrap();
            if !matches!(prompt, PromptPayload::PayCostsPromptMessage(_)) {
                break;
            }
            let (id, seat) = (r["RequestId"].as_u64().unwrap() as u32, r["Seat"].as_i64().unwrap() as i32);
            step = session.answer(id, seat, &choose(&prompt, &mut allow_cast)).ok().unwrap();
        }
        assert_eq!(green_in_pool(&step), 1, "the Star's mana");
        assert!(!options_of(&step).iter().any(|o| o.kind == ActionKindView::UndoManaAbility), "no Undo");
    }

    fn options_of(step: &Value) -> Vec<ActionOptionMessage> {
        match serde_json::from_value::<PromptPayload>(step["Requests"][0]["Prompt"].clone()).unwrap() {
            PromptPayload::PriorityPromptMessage(p) => p.options,
            PromptPayload::PayCostsPromptMessage(p) => p.options,
            _ => Vec::new(),
        }
    }

    fn zone_of(step: &Value, handle: ViewHandle) -> Value {
        step["Views"]["1"]["Objects"].as_array().unwrap().iter().find(|o| o["Handle"] == json!(handle)).map_or(Value::Null, |o| o["Zone"].clone())
    }

    fn green_in_pool(step: &Value) -> i64 {
        step["Views"]["1"]["Players"].as_array().unwrap().iter().map(|p| p["Mana"][4].as_i64().unwrap()).sum()
    }

    /// Black Lotus is taken back: sacrificed, it is on the battlefield again under the handle it had, its mana gone.
    #[test]
    fn a_sacrificed_black_lotus_is_taken_back() {
        let mut session = Session::new(deal(&["Black Lotus"], "Forest", 19)).ok().unwrap();
        let (lotus, after) = activate_mana_of(&mut session, "Black Lotus");
        assert_eq!(green_in_pool(&after), 3);
        assert_ne!(zone_of(&after, lotus.source), json!("Battlefield"), "sacrificed");
        let undo = options_of(&after).into_iter().find(|o| o.kind == ActionKindView::UndoManaAbility).expect("Undo is offered");

        let r = &after["Requests"][0];
        let (id, seat) = (r["RequestId"].as_u64().unwrap() as u32, r["Seat"].as_i64().unwrap() as i32);
        let undone = session.answer(id, seat, &AnswerPayload::TakeActionAnswer(TakeActionAnswer::take(&undo))).ok().unwrap();
        assert_eq!(green_in_pool(&undone), 0, "its mana is gone");
        assert_eq!(zone_of(&undone, lotus.source), json!("Battlefield"), "back, under the handle it had");
        assert!(options_of(&undone).iter().any(|o| o.source == lotus.source && o.produces_of_any_one_color == 3), "and can be used again");
    }

    /// Lion's Eye Diamond is never taken back: its cost discards the hand, and what was in it has been seen.
    #[test]
    fn lions_eye_diamond_is_not_taken_back() {
        let mut session = Session::new(deal(&["Lion's Eye Diamond"], "Forest", 19)).ok().unwrap();
        let (_, after) = activate_mana_of(&mut session, "Lion's Eye Diamond");
        assert_eq!(green_in_pool(&after), 3);
        assert!(!options_of(&after).iter().any(|o| o.kind == ActionKindView::UndoManaAbility), "no Undo");
    }

    /// A mana ability whose cost is its own {T} names no permanent to tap: `Tap` is for a tap cost paid by a permanent
    /// of the player's choosing, and the client asks for one on the board when it is set - a Forest or a Gleemox
    /// clicked to pay for a spell would ask which permanent to tap, offering only itself.
    #[test]
    fn a_mana_ability_tapping_itself_names_no_permanent_to_tap() {
        let mut session = Session::new(deal(&["Gleemox", "Llanowar Elves"], "Forest", 18)).ok().unwrap();
        let mut step = session.step();
        let mut allow_cast = true;
        let mut checked = 0;
        for _ in 0..2_000 {
            let request = &step["Requests"][0];
            let prompt: PromptPayload = serde_json::from_value(request["Prompt"].clone()).unwrap();
            if let PromptPayload::PayCostsPromptMessage(p) = &prompt {
                for o in p.options.iter().filter(|o| o.kind == ActionKindView::ActivateAbility && !o.produces.is_empty() || o.produces_of_any_one_color > 0) {
                    assert_eq!(o.tap, None, "{}", o.description);
                    checked += 1;
                }
                if checked >= 2 {
                    return;
                }
            }
            let (id, seat) = (request["RequestId"].as_u64().unwrap() as u32, request["Seat"].as_i64().unwrap() as i32);
            let answer = choose(&prompt, &mut allow_cast);
            step = match session.answer(id, seat, &answer) {
                Ok(step) => step,
                Err(_) => session.answer(id, seat, &prompt.default_answer()).ok().expect("the default answer is taken"),
            };
        }
        panic!("no cost was paid with both mana sources on offer");
    }

    #[test]
    fn a_forced_coin_flip_reaches_every_seat() {
        for (face, won) in [(1u8, true), (2, false)] {
            let mut session = Session::new(deal(&["Mana Crypt"], "Plains", 19)).ok().unwrap();
            session.game.state.force_rolls(&[face]);
            let heard = play_until(&mut session, |events| !of_type(events, "CoinFlippedEvent").is_empty());
            for seat in ["1", "2"] {
                let flips = of_type(&heard[seat], "CoinFlippedEvent");
                assert_eq!(flips.len(), 1, "seat {seat}");
                assert_eq!((flips[0]["Player"].as_i64(), flips[0]["Won"].as_bool()), (Some(1), Some(won)), "seat {seat}");
            }
        }
    }

    /// What an ability did is the entries from its `First` up to its resolution, and nothing it did not make: Mana
    /// Crypt's trigger flipped a coin and took life, and the step, the draw and the trigger itself came before it.
    #[test]
    fn a_resolved_ability_owns_only_the_entries_its_effect_made() {
        let mut session = Session::new(deal(&["Mana Crypt"], "Plains", 19)).ok().unwrap();
        session.game.state.force_rolls(&[2]);
        let heard = play_until(&mut session, |events| {
            let flipped = events.iter().position(|e| e["$type"] == "CoinFlippedEvent");
            flipped.is_some_and(|at| events[at..].iter().any(|e| e["$type"] == "AbilityResolvedEvent"))
        });
        for seat in ["1", "2"] {
            let events = &heard[seat];
            let resolved = events.iter().find(|e| e["$type"] == "AbilityResolvedEvent").unwrap();
            let (first, at) = (resolved["First"].as_u64().unwrap(), resolved["Sequence"].as_u64().unwrap());
            let made: Vec<&str> = events
                .iter()
                .filter(|e| (first..at).contains(&e["Sequence"].as_u64().unwrap()))
                .map(|e| e["$type"].as_str().unwrap())
                .collect();
            assert_eq!(made, ["CoinFlippedEvent", "LifeChangedEvent"], "seat {seat}");
        }
    }

    #[test]
    fn a_forced_die_roll_reaches_every_seat() {
        let top = ["Mox Ruby", "Mox Pearl", "Mox Jet", "Sol Ring", "Comet, Stellar Pup"];
        let mut session = Session::new(deal(&top, "Island", 20)).ok().unwrap();
        session.game.state.force_rolls(&[2]);
        let heard = play_until(&mut session, |events| !of_type(events, "DieRolledEvent").is_empty());
        for seat in ["1", "2"] {
            let rolls = of_type(&heard[seat], "DieRolledEvent");
            assert_eq!(rolls.len(), 1, "seat {seat}");
            assert_eq!((rolls[0]["Player"].as_i64(), rolls[0]["Sides"].as_i64(), rolls[0]["Result"].as_i64()), (Some(1), Some(6), Some(2)), "seat {seat}");
        }
    }
}
