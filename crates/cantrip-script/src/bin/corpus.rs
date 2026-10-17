//! Runs every scenario of `scenarios` whose cards the engine has, and says how each went; the scenarios with cards it
//! lacks are counted, with the cards that would bring in the most of them. A scenario marked `# Known bug:` asserts
//! what the rules require and is expected to fail: it is counted as a known bug, and one that passes is reported as
//! fixed - an error until its marker is removed.
//!
//!   cargo run -p cantrip-script --bin corpus [-- --failing] [-- --missing N] [-- --list]
//!   cargo run -p cantrip-script --bin corpus -- --card "Mind Stone"
//!
//! `--card` runs only the scenarios whose decks hold that card, and says of each what happened: the full report of a
//! failure, the cards still missing for one that cannot run.

use std::collections::BTreeMap;
use std::path::Path;

use cantrip_engine::card;
use cantrip_script::{GameScript, known_bug, run};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let show_failing = args.iter().any(|a| a == "--failing");
    let list = args.iter().any(|a| a == "--list");
    let only: Option<String> = args.iter().position(|a| a == "--card").and_then(|i| args.get(i + 1)).cloned();
    let missing_top: usize = args
        .iter()
        .position(|a| a == "--missing")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(25);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");

    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .expect("scripts")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "cantrip"))
        .collect();
    files.sort();

    let mut passed = Vec::new();
    let mut failed = Vec::new();
    let mut known = Vec::new();
    let mut fixed = Vec::new();
    let mut lacking: BTreeMap<String, usize> = BTreeMap::new();
    let mut blocked = 0;
    for path in &files {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let bug = std::fs::read_to_string(path).ok().and_then(|t| known_bug(&t));
        let script = match GameScript::load(path) {
            Ok(s) => s,
            Err(e) => {
                failed.push((name, format!("does not parse: {e}")));
                continue;
            }
        };
        if let Some(only) = &only
            && !script.cards().into_iter().any(|c| c.eq_ignore_ascii_case(only))
        {
            continue;
        }
        let mut missing: Vec<String> = script.cards().into_iter().filter(|c| card(c).is_none()).collect();
        missing.sort();
        missing.dedup();
        if !missing.is_empty() {
            if only.is_some() {
                println!("  needs {}: {name}", missing.join(", "));
            }
            blocked += 1;
            for m in missing {
                *lacking.entry(m).or_default() += 1;
            }
            continue;
        }
        let detailed = only.is_some();
        let outcome = std::panic::catch_unwind(|| {
            run(&script).map(|_| ()).map_err(|f| if detailed { f.report } else { f.message })
        });
        if let Some(bug) = bug {
            match outcome {
                Ok(Ok(())) => {
                    if detailed {
                        println!("  FIXED {name}: the known bug no longer fails ({bug}); remove its `# Known bug:` line");
                    }
                    fixed.push(name);
                }
                failing => {
                    if detailed {
                        // The report, so it can be checked that the scenario fails for the reason its marker gives.
                        let why = match failing {
                            Ok(Err(report)) => report,
                            _ => "panicked".to_string(),
                        };
                        println!("  known bug {name}: {bug}\n{why}");
                    }
                    known.push(name);
                }
            }
            continue;
        }
        if detailed {
            match &outcome {
                Ok(Ok(())) => println!("  pass {name}"),
                Ok(Err(report)) => println!("  FAIL {name}\n{report}"),
                Err(_) => println!("  PANIC {name}"),
            }
        }
        match outcome {
            Ok(Ok(())) => passed.push(name),
            Ok(Err(message)) => failed.push((name, message.lines().next().unwrap_or("").to_string())),
            Err(panic) => {
                let text = panic
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| panic.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_default();
                failed.push((name, format!("panicked: {text}")));
            }
        }
    }

    println!(
        "{} scenarios: {} pass, {} fail, {} known bugs, {} need cards the engine does not have.",
        files.len(),
        passed.len(),
        failed.len() + fixed.len(),
        known.len(),
        blocked
    );
    for name in &fixed {
        println!("  FIXED {name}: a known bug that now passes; remove its `# Known bug:` line");
    }
    if show_failing || failed.len() <= 40 {
        for (name, why) in &failed {
            println!("  FAIL {name}: {why}");
        }
    }
    if list {
        for name in &passed {
            println!("  pass {name}");
        }
    }
    if only.is_some() {
        if !failed.is_empty() || !fixed.is_empty() {
            std::process::exit(1);
        }
        return;
    }
    let mut ranked: Vec<(&String, &usize)> = lacking.iter().collect();
    ranked.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    println!("Cards most often missing (scenarios each appears in):");
    for (card, n) in ranked.into_iter().take(missing_top) {
        println!("  {n:3}  {card}");
    }
}
