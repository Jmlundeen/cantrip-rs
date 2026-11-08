//! Accounting for the generated workspace scenario tests, without a second game run.
//! `CANTRIP_CORPUS_REPORT` requests a JSON artifact; `CANTRIP_CORPUS_RUN_ID` must be a fresh identity supplied by the
//! caller. Consumers require that identity, `complete`, and `completed == expected`. A filtered/interrupted run keeps
//! its initial incomplete artifact. Only the initial and complete snapshots are written, avoiding quadratic I/O.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::Serialize;

#[derive(Clone, Debug)]
pub enum ScenarioOutcome {
    Passed,
    Failed,
    KnownBug,
    Fixed,
    Missing(Vec<String>),
}

#[derive(Debug, Serialize)]
pub struct CorpusReport {
    pub schema: u32,
    pub run_id: String,
    pub expected: usize,
    pub completed: usize,
    pub complete: bool,
    pub passed: usize,
    pub failed: usize,
    pub known_bugs: usize,
    pub fixed: usize,
    pub missing: usize,
    pub missing_cards: BTreeMap<String, usize>,
}

struct State {
    report: CorpusReport,
    seen: BTreeSet<String>,
}

pub struct ReportSession {
    path: PathBuf,
    state: Mutex<State>,
}

impl ReportSession {
    /// Initialize before executing any scenario. Truncating an old artifact cannot turn it into this run's pass.
    pub fn new(path: &Path, run_id: String, expected: usize) -> Result<Self, String> {
        if run_id.is_empty() {
            return Err("CANTRIP_CORPUS_RUN_ID must be nonempty".into());
        }
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        let report = CorpusReport {
            schema: 1,
            run_id,
            expected,
            completed: 0,
            complete: expected == 0,
            passed: 0,
            failed: 0,
            known_bugs: 0,
            fixed: 0,
            missing: 0,
            missing_cards: BTreeMap::new(),
        };
        let session = Self {
            path: path.to_path_buf(),
            state: Mutex::new(State {
                report,
                seen: BTreeSet::new(),
            }),
        };
        session.write(&session.state.lock().unwrap().report)?;
        Ok(session)
    }

    fn write(&self, report: &CorpusReport) -> Result<(), String> {
        let bytes = serde_json::to_vec(report).map_err(|e| e.to_string())?;
        std::fs::write(&self.path, bytes).map_err(|e| format!("{}: {e}", self.path.display()))
    }

    /// Every generated test records once, including failures, panics, missing definitions and fixed bug markers.
    pub fn record(&self, name: &str, outcome: ScenarioOutcome) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "corpus report lock poisoned".to_string())?;
        if state.seen.contains(name) {
            return Err(format!("scenario {name} was reported twice"));
        }
        if state.report.completed == state.report.expected {
            return Err("more scenarios than the generated list".into());
        }
        state.seen.insert(name.into());
        let report = &mut state.report;
        report.completed += 1;
        match outcome {
            ScenarioOutcome::Passed => report.passed += 1,
            ScenarioOutcome::Failed => report.failed += 1,
            ScenarioOutcome::KnownBug => report.known_bugs += 1,
            ScenarioOutcome::Fixed => report.fixed += 1,
            ScenarioOutcome::Missing(cards) => {
                report.missing += 1;
                for card in cards.into_iter().collect::<BTreeSet<_>>() {
                    *report.missing_cards.entry(card).or_default() += 1;
                }
            }
        }
        report.complete = report.completed == report.expected;
        if report.complete {
            self.write(report)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    fn path() -> PathBuf {
        std::env::temp_dir().join(format!(
            "cantrip-corpus-report-{}-{}.json",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ))
    }
    fn json(path: &Path) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    #[test]
    fn concurrent_results_count_every_outcome_and_missing_card_once_per_scenario() {
        let path = path();
        let report = Arc::new(ReportSession::new(&path, "this-run".into(), 6).unwrap());
        let threads: Vec<_> = [
            ScenarioOutcome::Passed,
            ScenarioOutcome::Failed,
            ScenarioOutcome::KnownBug,
            ScenarioOutcome::Fixed,
            ScenarioOutcome::Missing(vec!["A".into(), "A".into(), "B".into()]),
            ScenarioOutcome::Missing(vec!["A".into()]),
        ]
        .into_iter()
        .enumerate()
        .map(|(i, outcome)| {
            let report = Arc::clone(&report);
            std::thread::spawn(move || report.record(&i.to_string(), outcome).unwrap())
        })
        .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let data = json(&path);
        assert_eq!(data["run_id"], "this-run");
        assert_eq!(data["completed"], 6);
        assert_eq!(data["complete"], true);
        for key in ["passed", "failed", "known_bugs", "fixed"] {
            assert_eq!(data[key], 1, "{key}");
        }
        assert_eq!(data["missing"], 2);
        assert_eq!(data["missing_cards"]["A"], 2);
        assert_eq!(data["missing_cards"]["B"], 1);
        assert!(report.record("0", ScenarioOutcome::Passed).is_err());
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn stale_and_incomplete_artifacts_cannot_claim_a_new_complete_run() {
        let path = path();
        let old = ReportSession::new(&path, "old-run".into(), 1).unwrap();
        old.record("old", ScenarioOutcome::Passed).unwrap();
        assert_eq!(json(&path)["complete"], true);
        let current = ReportSession::new(&path, "new-run".into(), 2).unwrap();
        current.record("one", ScenarioOutcome::Passed).unwrap();
        let data = json(&path);
        assert_eq!(data["run_id"], "new-run");
        assert_eq!(data["complete"], false);
        assert_eq!(data["completed"], 0);
        assert!(ReportSession::new(&path, String::new(), 2).is_err());
        std::fs::remove_file(path).unwrap();
    }
}
