//! One test per scenario in `scenarios/*.cantrip`, so each passes or fails on its own and `cargo test <name>` runs one.
//! Adding a scenario is adding its file: nothing is listed anywhere.

use std::collections::HashSet;
use std::fmt::Write;
use std::path::Path;

fn main() {
    let dir = Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../../scenarios");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .expect("scenarios folder")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "cantrip"))
        .inspect(|p| println!("cargo:rerun-if-changed={}", p.display()))
        .map(|p| p.file_stem().unwrap().to_string_lossy().into_owned())
        .collect();
    names.sort();
    let mut out = String::new();
    let mut seen = HashSet::new();
    for name in names {
        let mut ident: String = name.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        if ident.starts_with(|c: char| c.is_ascii_digit()) {
            ident.insert(0, '_');
        }
        assert!(seen.insert(ident.clone()), "two scenarios make the test name {ident}");
        writeln!(out, "#[test]\nfn {ident}() {{ crate::scenario(\"{name}\"); }}\n").unwrap();
    }
    let dest = Path::new(&std::env::var("OUT_DIR").unwrap()).join("scenarios.rs");
    std::fs::write(dest, out).unwrap();
}
