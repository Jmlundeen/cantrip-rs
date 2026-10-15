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
        let events: Vec<Event> = self.game.log[from..bound].iter().map(|r| r.event).collect();
        self.published = bound;
        let mut protos = std::mem::take(&mut self.told);
        protos.extend(self.tracker.collect(&self.game, &events));

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
            match prompt(&self.game, &request, &mut self.handles[index]) {
                Ok((payload, may_cancel)) => requests.push(json!({
                    "RequestId": request.id,
                    "Seat": seat(request.player),
                    "Prompt": payload,
                    "MayCancel": may_cancel,
                })),
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
        let answer = translate(payload, &request, &self.handles[request.player.index()], &self.game).map_err(|e| match e {
            Untranslatable::UnknownHandle(d) => Rejection::new("UnknownHandle", d),
            Untranslatable::Illegal(d) => Rejection::new("IllegalAnswer", d),
        })?;

        let before = self.game.clone();
        let outcome = std::panic::catch_unwind(AssertUnwindSafe(|| self.game.answer(request.id, request.player, answer.clone())));
        match outcome {
            Ok(Ok(())) => {}
            Ok(Err(Rejected(why))) => return Err(Rejection::new("IllegalAnswer", why)),
            Err(_) => {
                self.game = before;
                return Err(Rejection::new("IllegalAnswer", "the engine failed on that answer; the game is as it was"));
            }
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
    pub fn concede(&mut self, seat_number: i32) -> Result<Value, Rejection> {
        let p = player_at(seat_number).filter(|p| p.index() < self.names.len()).ok_or_else(|| Rejection::new("NotYourTurnToAnswer", "no such seat"))?;
        if self.game.is_over() {
            return Err(Rejection::new("IllegalAnswer", "the game is over"));
        }
        self.conceded[p.index()] = true;
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
