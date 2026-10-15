//! Every scenario of `scenarios/*.cantrip` against the engine, one test each (see build.rs). A scenario whose decks
//! name a card the engine does not have yet is skipped with a note; one that can run must pass - unless it carries a
//! `# Known bug:` marker, in which case it must fail (a panic counts), and passing is the error: the bug is fixed and
//! the marker has to come off.

use std::path::{Path, PathBuf};

use cantrip_script::{GameScript, known_bug, run};

fn scripts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios")
}

fn scenario(name: &str) {
    let path = scripts().join(format!("{name}.cantrip"));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
    let script = GameScript::load(&path).unwrap_or_else(|e| panic!("{name}: {e}"));
    let mut missing: Vec<String> = (0..script.decks.len()).flat_map(|i| script.decklist(i)).filter(|c| cantrip_engine::card(c).is_none()).collect();
    missing.sort();
    missing.dedup();
    if !missing.is_empty() {
        println!("{name}: skipped, the engine has no {}", missing.join(", "));
        return;
    }
    match known_bug(&text) {
        None => {
            if let Err(failure) = run(&script) {
                panic!("\n{}", failure.report);
            }
        }
        Some(bug) => {
            let outcome = std::panic::catch_unwind(|| run(&script).map(|_| ()));
            if let Ok(Ok(())) = outcome {
                panic!("{name} passes: the known bug is fixed ({bug}). Remove its `# Known bug:` line.");
            }
            println!("{name}: known bug, still failing ({bug})");
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/scenarios.rs"));

/// The checker itself: a scenario with one expectation changed must fail.
#[test]
fn a_wrong_expectation_fails() {
    let path = scripts().join("duress-takes-the-named-card.cantrip");
    let text = std::fs::read_to_string(&path).unwrap().replace("Bot2.Graveyard = Counterspell", "Bot2.Graveyard = Lightning Bolt");
    let script = GameScript::parse(&text, "mutated").unwrap();
    let failure = run(&script).err().expect("a wrong expectation must fail");
    assert!(failure.message.contains("Bot2.Graveyard"), "{}", failure.message);
}
