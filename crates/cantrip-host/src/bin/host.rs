//! The sidecar `Cantrip.Server` runs: any number of games, one JSON object per line on stdin, one reply per request on
//! stdout, in order; stderr is the log. The protocol is written up in docs/host-protocol.md, every payload
//! `protocol.rs`'s JSON, and `NativeRulesHost` (C#) drives this binary.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use cantrip_host::protocol::AnswerPayload;
use cantrip_host::session::{Deal, Rejection, Session};
use cantrip_host::view::printed;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Deserialize)]
#[serde(tag = "Cmd")]
enum Request {
    NewGame {
        #[serde(rename = "Seats")]
        seats: Vec<SeatSpec>,
        #[serde(rename = "Seed")]
        seed: u64,
        #[serde(rename = "Shuffle", default = "yes")]
        shuffle: bool,
        #[serde(rename = "FirstPlayer", default)]
        first_player: Option<i32>,
        #[serde(rename = "Chooser", default)]
        chooser: Option<i32>,
        /// A Limited game's basic lands, which each sideboard holds any number of (`Setup::basic_lands_outside`).
        #[serde(rename = "BasicLandsOutside", default)]
        basic_lands_outside: Vec<String>,
        /// A `.cantrip` scenario to play instead of a dealt game: its decks, order and dice are the scenario's, and the
        /// seats named in `Driven` are answered by the table, every other by the script (docs/scenarios-in-the-browser.md).
        #[serde(rename = "Scenario", default)]
        scenario: Option<ScenarioSpec>,
    },
    Start {
        #[serde(rename = "GameId")]
        game_id: u32,
    },
    Answer {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "RequestId")]
        request_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
        #[serde(rename = "Answer")]
        answer: AnswerPayload,
    },
    Concede {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
    },
    View {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
    },
    Oracle {
        #[serde(rename = "Name")]
        name: String,
    },
    Close {
        #[serde(rename = "GameId")]
        game_id: u32,
    },
}

fn yes() -> bool {
    true
}

#[derive(Deserialize)]
struct ScenarioSpec {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Text")]
    text: String,
    #[serde(rename = "Driven", default)]
    driven: Vec<i32>,
}

#[derive(Deserialize)]
struct SeatSpec {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Deck")]
    deck: Vec<String>,
    /// The cards the seat owns outside the game (CR 100.4, 400.11a).
    #[serde(rename = "Sideboard", default)]
    sideboard: Vec<String>,
}

struct Host {
    games: HashMap<u32, Session>,
    next_game: u32,
}

fn rejected(r: Rejection) -> Value {
    json!({ "Rejected": { "Code": r.code, "Detail": r.detail } })
}

fn ok(value: Value) -> Value {
    json!({ "Ok": value })
}

impl Host {
    fn handle(&mut self, request: Request) -> Value {
        match request {
            Request::NewGame { seats, seed, shuffle, first_player, chooser, basic_lands_outside, scenario } => {
                let made = match scenario {
                    Some(spec) => Session::scenario(&spec.text, &spec.name, seats.into_iter().map(|s| s.name).collect(), &spec.driven),
                    None => {
                        let deal = Deal {
                            seats: seats.into_iter().map(|s| (s.name, s.deck, s.sideboard)).collect(),
                            basic_lands_outside,
                            seed,
                            shuffle,
                            first_player,
                            chooser,
                        };
                        Session::new(deal)
                    }
                };
                match made {
                    Ok(session) => {
                        let id = self.next_game;
                        self.next_game += 1;
                        let decklists = session.decklists();
                        self.games.insert(id, session);
                        ok(json!({ "GameId": id, "Decklists": decklists }))
                    }
                    Err(r) => rejected(r),
                }
            }
            Request::Start { game_id } => self.with(game_id, |g| Ok(g.step())),
            Request::Answer { game_id, request_id, seat, answer } => self.with(game_id, |g| g.answer(request_id, seat, &answer)),
            Request::Concede { game_id, seat } => self.with(game_id, |g| g.concede(seat)),
            Request::View { game_id, seat } => self.with(game_id, |g| g.view(seat).map(|v| json!({ "View": v }))),
            Request::Oracle { name } => {
                let card = cantrip_engine_card(&name);
                ok(json!({ "Card": card }))
            }
            Request::Close { game_id } => {
                self.games.remove(&game_id);
                ok(json!({}))
            }
        }
    }

    fn with(&mut self, game_id: u32, f: impl FnOnce(&mut Session) -> Result<Value, Rejection>) -> Value {
        match self.games.get_mut(&game_id) {
            Some(session) => match f(session) {
                Ok(v) => ok(v),
                Err(r) => rejected(r),
            },
            None => rejected(Rejection::new("TableClosed", format!("no game {game_id}"))),
        }
    }
}

fn cantrip_engine_card(name: &str) -> Option<cantrip_host::protocol::ObjectView> {
    let id = cantrip_engine::card(name)?;
    Some(printed(cantrip_engine::cards().get(id)))
}

/// Card code recurses (resolution inside a settle inside a priority), so the host loop runs on a thread with room.
const STACK_SIZE: usize = 256 << 20;

fn main() {
    let worker = std::thread::Builder::new().name("cantrip-native-host".into()).stack_size(STACK_SIZE).spawn(serve).expect("the host thread starts");
    if worker.join().is_err() {
        std::process::exit(1);
    }
}

fn serve() {
    eprintln!("cantrip-native-host: {} cards loaded", cantrip_engine::cards().cards.len());
    let mut host = Host { games: HashMap::new(), next_game: 1 };
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Request>(&line) {
            Ok(request) => {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| host.handle(request)));
                result.unwrap_or_else(|_| rejected(Rejection::new("IllegalAnswer", "the engine failed on that request")))
            }
            Err(e) => rejected(Rejection::new("InvalidSetup", format!("unreadable request: {e}"))),
        };
        let text = serde_json::to_string(&reply).expect("a reply serialises");
        if writeln!(out, "{text}").and_then(|_| out.flush()).is_err() {
            break;
        }
    }
}
