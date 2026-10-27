//! Runs every scenario of `scenarios` whose cards the engine has, and says how each went; the scenarios with cards it
//! lacks are counted, with the cards that would bring in the most of them. A scenario marked `# Known bug:` asserts
//! what the rules require and is expected to fail: it is counted as a known bug, and one that passes is reported as
//! fixed - an error until its marker is removed.
//!
//!   cargo run -p cantrip-script --bin corpus [-- --failing] [-- --missing N] [-- --list]
//!   cargo run -p cantrip-script --bin corpus -- --card "Mind Stone"
//!
//! `--card` runs only the scenarios whose decks hold that card, and says of each what happened: the full report of a
//! failure, the cards still missing for one that cannot run. `--scenario <part of a file name>` runs only the scenarios
//! whose names hold it, reported the same way; `--file <path>` runs one scenario file from anywhere (a draft in a
//! scratchpad, a reviewer's probe).
//!
//! For writing a scenario, with `--card` or `--scenario`:
//!
//!   --dump      prints the state each scenario ended in, pass or fail (in place of a bogus `Dump = ?` expectation)
//!   --diff N    prints the keys that changed between just before line N was answered and the end, as expectation
//!               lines with what each was: which keys the card moved - the values to expect are the rules' to say
//!   --next      once every line has played, stops at the next question that is not priority or a payment, and
//!               reports it with what it offers: what the next line would answer
//!   --brief     a failure's report without its state dump (the nearest keys of a missing one are still named)

use std::collections::BTreeMap;
use std::path::Path;

use cantrip_engine::card;
use cantrip_script::{GameScript, Watch, known_bug, run_watched};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let show_failing = args.iter().any(|a| a == "--failing");
    let list = args.iter().any(|a| a == "--list");
    let card_only: Option<String> = args.iter().position(|a| a == "--card").and_then(|i| args.get(i + 1)).cloned();
    let scenario_only: Option<String> = args.iter().position(|a| a == "--scenario").and_then(|i| args.get(i + 1)).cloned();
    let dump = args.iter().any(|a| a == "--dump");
    let watch = Watch {
        before_line: args.iter().position(|a| a == "--diff").and_then(|i| args.get(i + 1)).and_then(|v| v.parse().ok()),
        next: args.iter().any(|a| a == "--next"),
        brief: args.iter().any(|a| a == "--brief"),
    };
    if args.iter().any(|a| a == "--diff") && watch.before_line.is_none() {
        eprintln!("--diff takes the number of a line of the scenario file, as --diff 31");
        std::process::exit(2);
    }
    let file: Option<String> = args.iter().position(|a| a == "--file").and_then(|i| args.get(i + 1)).cloned();
    let only = card_only.clone().or(scenario_only.clone()).or(file.clone());
    let missing_top: usize = args
        .iter()
        .position(|a| a == "--missing")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(25);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scenarios");

    let mut files: Vec<_> = match &file {
        Some(path) => vec![Path::new(path).to_path_buf()],
        None => std::fs::read_dir(&dir)
            .expect("scripts")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "cantrip"))
            .collect(),
    };
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
        if let Some(only) = &card_only
            && !script.cards().into_iter().any(|c| c.eq_ignore_ascii_case(only))
        {
            continue;
        }
        if let Some(part) = &scenario_only
            && !name.to_ascii_lowercase().contains(&part.to_ascii_lowercase())
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
            run_watched(&script, if detailed { watch } else { Watch::default() })
                .map(|result| {
                    if detailed && watch.next {
                        println!("  {name}: --next: after its last line the script ends with nothing asked but priority");
                    }
                    if detailed && dump {
                        println!("  state {name} ended in:\n{}", result.dump.render());
                    }
                    if let Some(line) = watch.before_line.filter(|_| detailed) {
                        match &result.before {
                            Some(before) => {
                                println!("  {name}: what changed from just before line {line} to the end:");
                                for change in result.dump.diff(before) {
                                    println!("    {change}");
                                }
                            }
                            None => println!("  {name}: line {line} was never answered (a line of the action list, by its number in the file)"),
                        }
                    }
                })
                .map_err(|f| if detailed { f.report } else { f.message })
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
