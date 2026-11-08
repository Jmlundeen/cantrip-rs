//! One test per scenario in `scenarios/*.cantrip`, so each passes or fails on its own and `cargo test <name>` runs one.
//! Adding a scenario is adding its file: nothing is listed anywhere.

use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;

#[allow(dead_code)]
#[path = "src/parse.rs"]
mod parse;
#[allow(dead_code)]
#[path = "src/selection.rs"]
mod selection;

fn main() {
    let dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../scenarios");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/parse.rs");
    println!("cargo:rerun-if-changed=src/selection.rs");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut paths: Vec<_> = std::fs::read_dir(&dir)
        .expect("scenarios folder")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "cantrip"))
        .inspect(|p| println!("cargo:rerun-if-changed={}", p.display()))
        .collect();
    paths.sort();
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).to_path_buf();
    let index_path = dest.join("scenario-index.json");
    let prior = std::fs::read(&index_path)
        .ok()
        .and_then(|b| serde_json::from_slice::<selection::ScenarioIndex>(&b).ok())
        .unwrap_or_default();
    let (index, _) = prior.refresh(&paths);
    std::fs::write(&index_path, serde_json::to_vec(&index).unwrap()).unwrap();
    let names: Vec<_> = paths
        .iter()
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    let mut out = String::new();
    writeln!(out, "const SCENARIO_COUNT: usize = {};", names.len()).unwrap();
    let mut seen = HashSet::new();
    for name in names {
        let mut ident: String = name
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect();
        if ident.starts_with(|c: char| c.is_ascii_digit()) {
            ident.insert(0, '_');
        }
        assert!(
            seen.insert(ident.clone()),
            "two scenarios make the test name {ident}"
        );
        writeln!(
            out,
            "#[test]\nfn {ident}() {{ crate::scenario(\"{name}\"); }}\n"
        )
        .unwrap();
    }
    std::fs::write(dest.join("scenarios.rs"), out).unwrap();
}
