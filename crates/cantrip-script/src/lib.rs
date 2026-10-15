//! The `.cantrip` scenario runner against the engine (docs/engine-design.md, "Scenarios"): the scenario corpus in
//! `scenarios/` is the engine's acceptance test.

pub mod dump;
pub mod parse;
pub mod run;

pub use parse::GameScript;
pub use run::{ScriptFailure, ScriptResult, run};

/// The marker of a known bug: a scenario that asserts what the rules require and that the engine does not do yet.
pub const KNOWN_BUG: &str = "# Known bug:";

/// What a scenario's `# Known bug:` lines say, joined, or `None` for an ordinary scenario. A known bug is expected to
/// fail; the day it passes, its marker comes off and it is an ordinary scenario (scenarios/README.md).
pub fn known_bug(text: &str) -> Option<String> {
    let lines: Vec<&str> = text
        .lines()
        .filter_map(|l| l.trim_start().strip_prefix(KNOWN_BUG))
        .map(str::trim)
        .collect();
    (!lines.is_empty()).then(|| lines.join(" "))
}
