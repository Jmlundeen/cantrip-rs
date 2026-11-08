//! Source-validated corpus membership. The build script and runner use this module and the same `GameScript` parser.
//! A digest covers the bytes, rather than a timestamp: an edit with its timestamp restored is still an edit.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::parse::{GameScript, ParseError};

const PARSER: &[u8] = include_bytes!("parse.rs");
const INDEX_VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScenarioIndex {
    #[serde(default)]
    pub version: u32,
    pub parser_digest: String,
    pub entries: BTreeMap<String, IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub digest: String,
    pub cards: Vec<String>,
    pub parse_error: Option<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SelectionStats {
    pub sources_read: usize,
    pub scripts_parsed: usize,
    pub index_hits: usize,
}

pub struct ScenarioSource {
    pub path: PathBuf,
    text: Option<Result<String, String>>,
    parsed: Option<Result<GameScript, ParseError>>,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn filename(path: &Path) -> String {
    path.file_name().unwrap().to_string_lossy().into_owned()
}

fn read(path: &Path, stats: &mut SelectionStats) -> Result<String, String> {
    stats.sources_read += 1;
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn parse(text: &str, path: &Path, stats: &mut SelectionStats) -> Result<GameScript, ParseError> {
    stats.scripts_parsed += 1;
    GameScript::parse(text, &filename(path))
}

impl ScenarioIndex {
    /// Refresh the build artifact from current source bytes. Only changed/new sources are parsed; deleted sources
    /// disappear. A parser source change invalidates every entry, including a change to `GameScript::cards()`.
    pub fn refresh(&self, paths: &[PathBuf]) -> (Self, SelectionStats) {
        let parser_digest = digest(PARSER);
        let valid_parser = self.version == INDEX_VERSION && self.parser_digest == parser_digest;
        let mut index = Self {
            version: INDEX_VERSION,
            parser_digest,
            entries: BTreeMap::new(),
        };
        let mut stats = SelectionStats::default();
        for path in paths {
            let name = filename(path);
            let text = read(path, &mut stats);
            let source_digest = text
                .as_ref()
                .map(|t| digest(t.as_bytes()))
                .unwrap_or_default();
            if valid_parser
                && let Some(entry) = self.entries.get(&name)
                && !source_digest.is_empty()
                && entry.digest == source_digest
            {
                stats.index_hits += 1;
                index.entries.insert(name, entry.clone());
                continue;
            }
            let script = text
                .map_err(ParseError)
                .and_then(|t| parse(&t, path, &mut stats));
            let (mut cards, parse_error) = match script {
                Ok(script) => (script.cards(), None),
                Err(error) => (Vec::new(), Some(error.to_string())),
            };
            cards.sort();
            cards.dedup();
            index.entries.insert(
                name,
                IndexEntry {
                    digest: source_digest,
                    cards,
                    parse_error,
                },
            );
        }
        (index, stats)
    }
}

/// Filename filtering happens before any source is read. Card filtering validates source bytes against the generated
/// index and reparses only changed/new sources; selected unchanged sources are parsed by `load` before execution.
/// A malformed file cannot establish card membership, so `--card` keeps it as a failure, as the old runner did.
pub fn select_sources(
    paths: &[PathBuf],
    card: Option<&str>,
    scenario: Option<&str>,
    index: &ScenarioIndex,
) -> (Vec<ScenarioSource>, SelectionStats) {
    let mut selected = Vec::new();
    let mut stats = SelectionStats::default();
    let scenario = scenario.map(str::to_ascii_lowercase);
    let valid_parser = index.version == INDEX_VERSION && index.parser_digest == digest(PARSER);
    for path in paths {
        let name = path.file_stem().unwrap().to_string_lossy();
        if scenario
            .as_ref()
            .is_some_and(|part| !name.to_ascii_lowercase().contains(part))
        {
            continue;
        }
        let mut source = ScenarioSource {
            path: path.clone(),
            text: None,
            parsed: None,
        };
        if let Some(card) = card {
            let text = read(path, &mut stats);
            let entry = valid_parser
                .then(|| index.entries.get(&filename(path)))
                .flatten()
                .filter(|entry| {
                    text.as_ref()
                        .is_ok_and(|t| entry.digest == digest(t.as_bytes()))
                });
            let matches = if let Some(entry) = entry {
                stats.index_hits += 1;
                entry.parse_error.is_some()
                    || entry.cards.iter().any(|c| c.eq_ignore_ascii_case(card))
            } else {
                let script = text
                    .as_ref()
                    .map_err(|e| ParseError(e.clone()))
                    .and_then(|t| parse(t, path, &mut stats));
                let matches = script.as_ref().map_or(true, |s| {
                    s.cards().iter().any(|c| c.eq_ignore_ascii_case(card))
                });
                source.parsed = Some(script);
                matches
            };
            if !matches {
                continue;
            }
            source.text = Some(text);
        }
        selected.push(source);
    }
    (selected, stats)
}

impl ScenarioSource {
    /// Load once: focused card selection already read the bytes, and a changed source was already parsed.
    pub fn load(self, stats: &mut SelectionStats) -> Result<(String, GameScript), ParseError> {
        let text = self
            .text
            .unwrap_or_else(|| read(&self.path, stats))
            .map_err(ParseError)?;
        let script = self
            .parsed
            .unwrap_or_else(|| parse(&text, &self.path, stats))?;
        Ok((text, script))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "cantrip-selection-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, text: &str) -> PathBuf {
            let path = self.0.join(format!("{name}.cantrip"));
            std::fs::write(&path, text).unwrap();
            path
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).unwrap();
        }
    }

    fn selected(paths: &[PathBuf], card: &str, index: &ScenarioIndex) -> Vec<String> {
        select_sources(paths, Some(card), None, index)
            .0
            .into_iter()
            .map(|s| filename(&s.path))
            .collect()
    }

    #[test]
    fn membership_matches_parser_for_all_player_card_sources_and_literal_faces_and_tokens() {
        let f = Fixture::new();
        let a = f.write("all-sources", "[Bot1 Deck]\n4 Front // Back\n[Bot2 Deck]\nTreasure\n[Bot1 Sideboard]\nBack Face\n[Basic Lands Outside]\nWastes\n[Bot1 Companion]\nCompanion\n[Bot1 Commander]\nCommander\n[Actions]\nBot1.Hand.Action Only.Cast\n");
        let b = f.write("other", "[Bot1 Deck]\nIsland\n");
        let paths = vec![a, b];
        let (index, _) = ScenarioIndex::default().refresh(&paths);
        for card in [
            "front // back",
            "Treasure",
            "BACK FACE",
            "Wastes",
            "Companion",
            "Commander",
            "Island",
            "Action Only",
            "Front",
        ] {
            let expected: Vec<_> = paths
                .iter()
                .filter(|p| {
                    GameScript::load(p)
                        .unwrap()
                        .cards()
                        .iter()
                        .any(|c| c.eq_ignore_ascii_case(card))
                })
                .map(|p| filename(p))
                .collect();
            assert_eq!(selected(&paths, card, &index), expected, "{card}");
        }
        let (sources, mut stats) = select_sources(&paths, Some("Treasure"), None, &index);
        assert_eq!(stats.scripts_parsed, 0);
        for source in sources {
            source.load(&mut stats).unwrap();
        }
        assert_eq!(stats.sources_read, 2);
        assert_eq!(stats.scripts_parsed, 1);
    }

    #[test]
    fn additions_edits_deletions_and_parser_changes_invalidate_membership() {
        let f = Fixture::new();
        let a = f.write("a", "[Bot1 Deck]\nIsland\n");
        let b = f.write("b", "[Bot1 Deck]\nForest\n");
        let paths = vec![a.clone(), b.clone()];
        let (mut index, _) = ScenarioIndex::default().refresh(&paths);
        assert_eq!(index.refresh(&paths).1.scripts_parsed, 0);
        // Same byte count and restored modification time defeat mtime/size caches, but not source validation.
        let modified = std::fs::metadata(&a).unwrap().modified().unwrap();
        std::fs::write(&a, "[Bot1 Deck]\nForest\n").unwrap();
        std::fs::File::options()
            .write(true)
            .open(&a)
            .unwrap()
            .set_times(std::fs::FileTimes::new().set_modified(modified))
            .unwrap();
        assert!(selected(&paths, "Island", &index).is_empty());
        let c = f.write("c", "[Bot1 Deck]\nForest\n");
        std::fs::remove_file(b).unwrap();
        let current = vec![a, c];
        let (fresh, stats) = index.refresh(&current);
        assert_eq!(stats.scripts_parsed, 2);
        assert!(!fresh.entries.contains_key("b.cantrip"));
        assert_eq!(selected(&current, "Forest", &index).len(), 2);
        index = fresh;
        index.parser_digest = "old-parser".into();
        // Even correct source digests cannot validate old parser membership.
        for entry in index.entries.values_mut() {
            entry.cards = vec!["Invented".into()];
        }
        assert!(selected(&current, "Invented", &index).is_empty());
        assert_eq!(index.refresh(&current).1.scripts_parsed, 2);
        index.parser_digest = digest(PARSER);
        index.version = 0;
        assert!(selected(&current, "Invented", &index).is_empty());
        assert_eq!(index.refresh(&current).1.scripts_parsed, 2);
    }

    #[test]
    fn scenario_names_filter_before_read_and_card_focus_preserves_parse_failures() {
        let f = Fixture::new();
        let wanted = f.write("Mixed-Case-Wanted", "[Bot1 Deck]\nIsland\n");
        let broken = f.write("broken", "no section\n");
        let paths = vec![wanted, broken];
        let (index, _) = ScenarioIndex::default().refresh(&paths);
        let (sources, mut stats) = select_sources(&paths, None, Some("CASE-wanted"), &index);
        assert_eq!(stats.sources_read, 0);
        assert_eq!(sources.len(), 1);
        sources
            .into_iter()
            .next()
            .unwrap()
            .load(&mut stats)
            .unwrap();
        assert_eq!(stats.sources_read, 1);
        assert_eq!(
            selected(&paths, "Unrelated", &index),
            vec!["broken.cantrip"]
        );
        let (sources, _) = select_sources(&paths, Some("Unrelated"), Some("wanted"), &index);
        assert!(sources.is_empty());
    }
}
