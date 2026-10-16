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
