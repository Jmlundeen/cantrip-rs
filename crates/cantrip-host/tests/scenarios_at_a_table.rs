//! Every scenario of `scenarios/*.cantrip` played at a table (docs/scenarios-in-the-browser.md): Bot1's seat driven, its
//! every answer the one the host's step says the script means, sent back over the wire as a client would send it. A
//! scenario that passes alone must pass so: what the host says to answer is what the script means, in the seat's own
//! handles.

use std::path::{Path, PathBuf};

use cantrip_host::protocol::{ActionKindView, AnswerPayload, TakeActionAnswer, ViewHandle};
use cantrip_host::session::Session;
use cantrip_script::{GameScript, known_bug, run};
use serde_json::Value;

fn scripts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios")
}

/// Plays one scenario at a table with Bot1 driven. Ok, or the verdict's report.
fn at_a_table(name: &str, text: &str) -> Result<(), String> {
    let mut session = Session::scenario(text, name, Vec::new(), &[1]).map_err(|r| r.detail)?;
    let mut step = session.step();
    for _ in 0..20_000 {
        let scenario = &step["Scenario"];
        if let Some(verdict) = scenario["Verdict"].as_object() {
            return if verdict["Passed"] == Value::Bool(true) { Ok(()) } else { Err(verdict["Report"].as_str().unwrap_or("").to_string()) };
        }
        let expected = &scenario["Expected"];
        if expected.is_null() {
            return Err(format!("the step says nothing to answer and gives no verdict: {}", step["Requests"]));
        }
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).map_err(|e| format!("the expected answer does not read back: {e}"))?
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| format!("{}: {} ({:?}, line {})", r.code, r.detail, answer, expected["Line"]))?;
    }
    Err("the scenario went on too long at the table".into())
}

/// Hidden information (CR 406.3, 708.2): Headliner Scarlett exiles Bot2's top card, Firdoch Core, face down. What Bot1's
/// view says of that card - at the table, in the views as they are sent - carries nothing of what it is: no name, and not
/// that it is every creature type (Changeling), which would say it.
#[test]
fn a_face_down_changeling_in_exile_tells_the_opponent_nothing_of_what_it_is() {
    let name = "headliner-scarlett-face-down-firdoch-core-shows-an-opponent-no-types";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let mut seen = 0;
    for _ in 0..20_000 {
        let view = session.view(1).map_err(|r| r.detail).unwrap();
        for object in view.objects.iter().filter(|o| o.face_down) {
            seen += 1;
            assert!(!object.every_creature_type, "a face-down card says it is every creature type: {:?}", object.handle);
            assert!(object.type_line_subtypes.is_empty() && object.subtypes.is_empty(), "a face-down card names types");
            assert!(object.granted_abilities.is_empty() && object.lost_abilities.is_empty() && object.name.is_empty());
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(seen > 0, "the scenario never showed Bot1 a face-down card");
}

/// A restriction on casting is shown to the player it binds (CR 101.2): while Avatar's Wrath bars Bot2 from casting outside
/// its hand, Bot2's view says so beside its life total and Bot1's says it of Bot2, and once Bot1's next turn has begun
/// neither says it any more.
#[test]
fn a_bar_on_casting_outside_the_hand_is_shown_to_both_players_while_it_lasts() {
    let name = "avatars-wrath-bars-its-opponent-casting-from-exile-and-graveyard-until-its-casters-next-turn";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after) = (0, false);
    for _ in 0..20_000 {
        let own = session.view(2).map_err(|r| r.detail).unwrap();
        let other = session.view(1).map_err(|r| r.detail).unwrap();
        let facts = |view: &cantrip_host::protocol::PlayerView| view.players[1].pending_effects.clone();
        let (own_facts, other_facts) = (facts(&own), facts(&other));
        let bot1 = own.players[0].name.clone();
        let barred = own_facts.contains(&format!("You can't cast spells from anywhere other than your hand until {bot1}'s next turn."));
        assert_eq!(
            barred,
            other_facts.contains(&"They can't cast spells from anywhere other than their hand until your next turn.".to_string()),
            "the two players are told different things: {own_facts:?} / {other_facts:?}"
        );
        if barred {
            shown += 1;
            assert!(!gone_after, "the restriction came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "Bot2 was never told it can't cast outside its hand");
    assert!(gone_after, "the restriction was still shown after Bot1's next turn began");
}

/// "Players can't gain life" is shown to both players while it applies (CR 119.7): while Bot1's Sunspine Lynx is on the
/// battlefield, each player's view says it of them ("You can't gain life.") and of the other ("They can't gain life."), and once
/// Swords to Plowshares has exiled the Lynx neither says it any more.
#[test]
fn players_cant_gain_life_is_shown_to_both_players_while_the_static_ability_applies() {
    let name = "sunspine-lynx-stops-life-gain-only-while-it-is-on-the-battlefield";
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after) = (0, false);
    for _ in 0..20_000 {
        let views = [session.view(1).map_err(|r| r.detail).unwrap(), session.view(2).map_err(|r| r.detail).unwrap()];
        let says = |viewer: usize, about: usize, text: &str| views[viewer].players[about].pending_effects.iter().any(|f| f == text);
        let applies = says(0, 0, "You can't gain life.");
        for (viewer, about) in [(0, 0), (0, 1), (1, 0), (1, 1)] {
            let text = if viewer == about { "You can't gain life." } else { "They can't gain life." };
            assert_eq!(says(viewer, about, text), applies, "seat {} is told something else of seat {}", viewer + 1, about + 1);
        }
        if applies {
            shown += 1;
            assert!(!gone_after, "the restriction came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "no player was ever told they can't gain life");
    assert!(gone_after, "the restriction was still shown after the Lynx left");
}

/// Plays the scenario `name` at a table and judges what each seat is told, beside Bot1's life total, of a delayed triggered
/// ability that waits for its event (CR 603.7b) whose words contain `words`: nothing before it is set up, then Bot1's own view
/// names its source and gives the words, Bot2's says it is Bot1's ("They have ..."), and once it has lapsed neither does.
fn judge_waiting_delayed_ability(name: &str, words: &str, source: &str) {
    let text = std::fs::read_to_string(scripts().join(format!("{name}.cantrip"))).unwrap();
    let mut session = Session::scenario(&text, name, Vec::new(), &[1]).map_err(|r| r.detail).unwrap();
    let mut step = session.step();
    let (mut shown, mut gone_after, mut first) = (0, false, true);
    for _ in 0..20_000 {
        let mut facts = |seat: i32| session.view(seat).map_err(|r| r.detail).unwrap().players[0].pending_effects.clone();
        let (own, other) = (facts(1), facts(2));
        let own_line = own.iter().find(|f| f.contains(words));
        let other_line = other.iter().find(|f| f.contains(words));
        assert_eq!(own_line.is_some(), other_line.is_some(), "the two seats are told different things: {own:?} / {other:?}");
        if first {
            assert!(own_line.is_none(), "the ability was shown before anything set it up: {own:?}");
            first = false;
        }
        if let (Some(mine), Some(theirs)) = (own_line, other_line) {
            assert!(mine.starts_with(&format!("{source}: ")), "Bot1 is not told where it came from: {mine}");
            assert!(theirs.starts_with(&format!("They have {source}'s delayed ability: ")), "Bot2 is told something else: {theirs}");
            shown += 1;
            assert!(!gone_after, "the waiting ability came back");
        } else if shown > 0 {
            gone_after = true;
        }
        let scenario = &step["Scenario"];
        if scenario["Verdict"].is_object() {
            assert_eq!(scenario["Verdict"]["Passed"], Value::Bool(true));
            break;
        }
        let expected = &scenario["Expected"];
        let answer: AnswerPayload = if expected["Undo"] == Value::Bool(true) {
            AnswerPayload::TakeActionAnswer(TakeActionAnswer::new(ActionKindView::UndoManaAbility, ViewHandle::NONE))
        } else {
            serde_json::from_value(expected["Answer"].clone()).unwrap()
        };
        let id = expected["RequestId"].as_u64().unwrap() as u32;
        let seat = expected["Seat"].as_i64().unwrap() as i32;
        step = session.answer(id, seat, &answer).map_err(|r| r.detail).unwrap();
    }
    assert!(shown > 0, "the waiting ability was never shown");
    assert!(gone_after, "the waiting ability was still shown after it lapsed");
}

/// "Whenever you attack this turn" (Dalkovan Encampment's activated ability) is shown from the moment it is set up until the
/// turn ends.
#[test]
fn a_this_turn_delayed_ability_is_shown_until_the_turn_ends() {
    judge_waiting_delayed_ability(
        "dalkovan-encampment-shows-the-waiting-attack-ability-and-the-warriors-in-the-browser",
        "Whenever you attack this turn",
        "Dalkovan Encampment",
    );
}

/// An "until your next turn" delayed ability (Tamiyo, Seasoned Scholar's +2) is shown until that turn begins.
#[test]
fn an_until_your_next_turn_delayed_ability_is_shown_until_that_turn_begins() {
    judge_waiting_delayed_ability(
        "tamiyo-seasoned-scholar-plus-two-weakens-creatures-attacking-her-too",
        "Whenever a creature attacks you or a planeswalker you control",
        "Tamiyo, Seasoned Scholar",
    );
}

#[test]
fn every_scenario_passes_at_a_table_with_bot1_driven() {
    let mut entries: Vec<PathBuf> =
        std::fs::read_dir(scripts()).unwrap().map(|e| e.unwrap().path()).filter(|p| p.extension().is_some_and(|e| e == "cantrip")).collect();
    entries.sort();
    let mut failed = Vec::new();
    let mut played = 0;
    for path in entries {
        let text = std::fs::read_to_string(&path).unwrap();
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let Ok(script) = GameScript::parse(&text, &name) else { continue };
        if known_bug(&text).is_some() || (0..script.decks.len()).flat_map(|i| script.decklist(i)).any(|c| cantrip_engine::card(&c).is_none()) {
            continue;
        }
        if run(&script).is_err() {
            continue;
        }
        played += 1;
        if let Err(report) = at_a_table(&name, &text) {
            failed.push(format!("{name}: {}", report.lines().take(3).collect::<Vec<_>>().join(" | ")));
        }
    }
    assert!(failed.is_empty(), "{} of {played} scenarios fail at a table:\n{}", failed.len(), failed.join("\n"));
}
