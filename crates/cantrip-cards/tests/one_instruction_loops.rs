//! A loop that changes the game with no event for each object - "each creature you control gets +1/+1", "those creatures
//! gain trample" - is one instruction of the card's text (CR 608.2c, 611.2c): the state triggers judge it once it is done,
//! never with some objects changed and others not (CR 603.8). The engine can't tell such a loop from a run of separate
//! instructions, so the card says it (`cantrip_api::instruction`, `Game::one_instruction`), or says that each pass ends one
//! (`Game::end_sentence`). Under `CANTRIP_VERIFY` the engine refuses a resolution that changes the game with no event twice
//! and says neither, but a scenario that runs such a loop over one object never makes the second change; this test finds
//! the loop itself, in every card file and every shared mechanic.

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..")
}

/// The calls that change the game with no event: a continuous effect beginning (`add_effect`, `add_player_effect` and the
/// shared mechanics that make one), a copy effect, a change of control.
fn is_change(name: &str) -> bool {
    matches!(
        name,
        "add_effect" | "add_player_effect" | "become_copy" | "gain_control" | "set_copy" | "pump" | "haste_until_end_of_turn" | "trample_until_end_of_turn"
    ) || (name.starts_with("gain_") && name.ends_with("_until_end_of_turn"))
}

/// `source` with comments and the insides of string and character literals blanked, so braces and words in them are not
/// read as code. Offsets are kept.
fn code_only(source: &str) -> Vec<u8> {
    let s = source.as_bytes();
    let mut out = s.to_vec();
    let mut i = 0;
    while i < s.len() {
        if s[i..].starts_with(b"//") {
            while i < s.len() && s[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if s[i..].starts_with(b"/*") {
            while i < s.len() && !s[i..].starts_with(b"*/") {
                if s[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
            for k in i..(i + 2).min(s.len()) {
                out[k] = b' ';
            }
            i += 2;
        } else if s[i] == b'"' {
            out[i] = b' ';
            i += 1;
            while i < s.len() && s[i] != b'"' {
                let step = if s[i] == b'\\' { 2 } else { 1 };
                for k in i..(i + step).min(s.len()) {
                    if s[k] != b'\n' {
                        out[k] = b' ';
                    }
                }
                i += step;
            }
            if i < s.len() {
                out[i] = b' ';
            }
            i += 1;
        } else if s[i] == b'\'' {
            // A character literal ('x', '\n', '{'); a lifetime ('a) has no closing quote two or three bytes on.
            let end = if s.get(i + 1) == Some(&b'\\') { s[i + 2..].iter().position(|&c| c == b'\'').map(|p| i + 2 + p) } else if s.get(i + 2) == Some(&b'\'') { Some(i + 2) } else { None };
            match end {
                Some(end) => {
                    for k in i..=end {
                        out[k] = b' ';
                    }
                    i = end + 1;
                }
                None => i += 1,
            }
        } else {
            i += 1;
        }
    }
    out
}

/// The offset of the `}` that closes the `{` at `open`.
fn block_end(code: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    for (k, &c) in code.iter().enumerate().skip(open) {
        if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                return k;
            }
        }
    }
    code.len()
}

fn is_ident(c: u8) -> bool {
    c.is_ascii_alphanumeric() || c == b'_'
}

/// Every word of `code` in `[from, to)` with where it starts.
fn words(code: &[u8], from: usize, to: usize) -> Vec<(usize, &str)> {
    let mut found = Vec::new();
    let mut i = from;
    while i < to {
        if is_ident(code[i]) && (i == 0 || !is_ident(code[i - 1])) {
            let start = i;
            while i < to && is_ident(code[i]) {
                i += 1;
            }
            found.push((start, std::str::from_utf8(&code[start..i]).unwrap_or("")));
        } else {
            i += 1;
        }
    }
    found
}

/// The word at `at` is called: an open parenthesis follows it.
fn called(code: &[u8], after: usize) -> bool {
    code[after..].iter().find(|c| !c.is_ascii_whitespace()) == Some(&b'(')
}

/// The `for` loops of `source` that make a change with no event and are neither inside an instruction the code says is one
/// nor end one each pass: their line numbers.
fn unsaid_loops(source: &str) -> Vec<usize> {
    let code = code_only(source);
    let all = words(&code, 0, code.len());
    // The spans said to be one instruction: the closure passed to `instruction`, `one_instruction` or `settle_once`.
    let grouped: Vec<(usize, usize)> = all
        .iter()
        .filter(|(at, w)| matches!(*w, "instruction" | "one_instruction" | "settle_once") && called(&code, at + w.len()))
        .filter_map(|(at, _)| code[*at..].iter().position(|&c| c == b'{').map(|p| (*at, block_end(&code, at + p))))
        .collect();
    let mut lines = Vec::new();
    for &(at, w) in &all {
        if w != "for" {
            continue;
        }
        // `for x in xs {`: the header runs to the first `{`, with no `;` in it (a `for<'a>` bound has none of this).
        let Some(open) = code[at..].iter().position(|&c| c == b'{' || c == b';').map(|p| at + p) else { continue };
        if code[open] != b'{' || !words(&code, at, open).iter().any(|(_, w)| *w == "in") {
            continue;
        }
        let end = block_end(&code, open);
        let body = words(&code, open, end);
        let changes = body.iter().any(|(b, w)| is_change(w) && called(&code, b + w.len()));
        let ends_each = body.iter().any(|(_, w)| *w == "end_sentence");
        let inside = grouped.iter().any(|&(from, to)| from <= at && end <= to);
        if changes && !ends_each && !inside {
            lines.push(source[..at].matches('\n').count() + 1);
        }
    }
    lines
}

fn rust_files(dir: &Path, into: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())).flatten() {
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, into);
        } else if path.extension().is_some_and(|e| e == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn every_loop_of_changes_with_no_event_says_whether_it_is_one_instruction() {
    let mut files = Vec::new();
    rust_files(&root().join("cantrip-cards/src/cards"), &mut files);
    rust_files(&root().join("cantrip-mechanics/src"), &mut files);
    assert!(files.len() > 100, "the card files were not found");
    let mut unsaid = Vec::new();
    for file in &files {
        let source = fs::read_to_string(file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        for line in unsaid_loops(&source) {
            unsaid.push(format!("{}:{line}", file.display()));
        }
    }
    assert!(
        unsaid.is_empty(),
        "these loops change the game with no event for each object, and say neither that they are one instruction \
         (`cantrip_api::instruction`) nor that each pass ends one (`Game::end_sentence`; CR 603.8, 608.2c, 611.2c):\n{}",
        unsaid.join("\n")
    );
}

#[test]
fn the_check_finds_a_loop_and_passes_one_said_to_be_one_instruction() {
    let unsaid = "fn f(g: &mut dyn Game) {\n    for id in ids {\n        g.add_effect(a, id, k, d);\n    }\n}\n";
    assert_eq!(unsaid_loops(unsaid), vec![2]);
    let said = "fn f(g: &mut dyn Game) {\n    instruction(g, |g| {\n        for id in ids {\n            pump(g, a, id, 1, 1);\n        }\n    });\n}\n";
    assert!(unsaid_loops(said).is_empty());
    let each = "fn f(g: &mut dyn Game) {\n    for id in ids {\n        pump(g, a, id, 1, 1);\n        g.end_sentence();\n    }\n}\n";
    assert!(unsaid_loops(each).is_empty());
    let quiet = "fn f(g: &mut dyn Game) {\n    for id in ids {\n        g.draw(id, 1); // add_effect(\n    }\n}\n";
    assert!(unsaid_loops(quiet).is_empty());
}
