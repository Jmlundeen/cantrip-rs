//! The sidecar `Cantrip.Server` runs: any number of games, one JSON object per line on stdin, one reply per request on
//! stdout, in order; stderr is the log. The protocol is written up in docs/host-protocol.md, every payload
//! `protocol.rs`'s JSON, and `NativeRulesHost` (C#) drives this binary.

use std::collections::HashMap;
use std::io::{self, BufRead, Write};

use cantrip_host::protocol::{AnswerPayload, AttackMessage, BlockMessage, ViewHandle};
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
        /// The format's minimum deck size, which the server knows from the table's format (`Setup::minimum_deck_size`, CR
        /// 100.2a-b): always said, never inferred. A scenario's is its own.
        #[serde(rename = "MinimumDeckSize", default)]
        minimum_deck_size: Option<u32>,
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
    /// What the rule of an outstanding choice of cards says of a partial selection (docs/host-protocol.md, "Checking a
    /// selection"): nothing advances.
    Check {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "RequestId")]
        request_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
        #[serde(rename = "Selected", default)]
        selected: Vec<ViewHandle>,
    },
    /// What the rules make of a declaration of blockers for an outstanding declaration (docs/host-protocol.md, "Checking a
    /// declaration of blockers"): nothing advances.
    CheckBlocks {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "RequestId")]
        request_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
        #[serde(rename = "Blocks", default)]
        blocks: Vec<BlockMessage>,
    },
    /// What the rules make of a declaration of attackers for an outstanding declaration (docs/host-protocol.md, "Checking a
    /// declaration of attackers"): nothing advances.
    CheckAttacks {
        #[serde(rename = "GameId")]
        game_id: u32,
        #[serde(rename = "RequestId")]
        request_id: u32,
        #[serde(rename = "Seat")]
        seat: i32,
        #[serde(rename = "Attacks", default)]
        attacks: Vec<AttackMessage>,
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
    /// Which cards with companion in a sideboard the deck fulfils the condition of (CR 702.139a), for the deck editor.
    Companions {
        #[serde(rename = "Deck")]
        deck: Vec<String>,
        #[serde(rename = "Sideboard", default)]
        sideboard: Vec<String>,
        #[serde(rename = "MinimumDeckSize")]
        minimum_deck_size: u32,
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
    /// The companion chosen as the deck was built (CR 702.139a): dealt into the sideboard beside it, and revealed before the
    /// game if the deck fulfils its condition (CR 103.2b). The server has refused a deck whose companion it doesn't.
    #[serde(rename = "Companion", default)]
    companion: Option<String>,
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
            Request::NewGame { seats, seed, shuffle, first_player, chooser, basic_lands_outside, minimum_deck_size, scenario } => {
                let made = match scenario {
                    Some(spec) => Session::scenario(&spec.text, &spec.name, seats.into_iter().map(|s| s.name).collect(), &spec.driven),
                    None if minimum_deck_size.is_none() => {
                        Err(Rejection::new("InvalidSetup", "a dealt game says its format's minimum deck size (MinimumDeckSize)"))
                    }
                    None => {
                        let companions = seats.iter().map(|s| s.companion.clone()).collect();
                        let deal = Deal {
                            seats: seats.into_iter().map(|s| (s.name, s.deck, s.sideboard)).collect(),
                            basic_lands_outside,
                            companions,
                            minimum_deck_size: minimum_deck_size.unwrap_or_default(),
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
            Request::Check { game_id, request_id, seat, selected } => self.with(game_id, |g| g.check(request_id, seat, &selected)),
            Request::CheckBlocks { game_id, request_id, seat, blocks } => self.with(game_id, |g| g.check_blocks(request_id, seat, &blocks)),
            Request::CheckAttacks { game_id, request_id, seat, attacks } => self.with(game_id, |g| g.check_attacks(request_id, seat, &attacks)),
            Request::Concede { game_id, seat } => self.with(game_id, |g| g.concede(seat)),
            Request::View { game_id, seat } => self.with(game_id, |g| g.view(seat).map(|v| json!({ "View": v }))),
            Request::Oracle { name } => {
                let card = cantrip_engine_card(&name);
                ok(json!({ "Card": card }))
            }
            Request::Companions { deck, sideboard, minimum_deck_size } => match cantrip_host::session::companions(&deck, &sideboard, minimum_deck_size) {
                Ok(checks) => ok(json!({ "Companions": checks })),
                Err(r) => rejected(r),
            },
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
