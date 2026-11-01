//! A card names its delayed abilities by their place in its `delayed` list (`AbilityRef::delayed(card, n)`): the delayed
//! and reflexive triggers it creates, the abilities it grants, a keyword's returning ability (`earthbend::RETURN`). One
//! that names a place its list doesn't have names no ability; in play `CardTable::ability` answers it with one that does
//! nothing rather than crash, so the card would silently do nothing. This test finds it: every `AbilityRef::delayed`
//! with a literal place in a card file is within the longest `delayed` list of the definitions that file registers, and
//! every card that earthbends lists `earthbend::RETURN` first (the delayed ability 0 that `earthbend::effect` creates).
//! A place that is computed - a variable or a loop (Dire Flail, Old-Growth Troll), or a const parameter a card passes to a
//! shared mechanic (`delayed::*::<N>`, `station`, `backup`) - is not read here: `CardTable::ability`'s debug assertion
//! finds it instead, in the checked runs of the scenarios, which run every line of a card's file.

use cantrip_api::*;
use std::fs;
use std::path::Path;

/// `source` with comments blanked, so a reference in a comment is not read as code.
fn without_comments(source: &str) -> String {
    source.lines().map(|l| l.split("//").next().unwrap_or("")).collect::<Vec<_>>().join("\n")
}

/// The literal places of every `AbilityRef::delayed(card, n)` in `code`.
fn literal_places(code: &str) -> Vec<usize> {
    let mut places = Vec::new();
    for (at, _) in code.match_indices("AbilityRef::delayed(") {
        let args = &code[at + "AbilityRef::delayed(".len()..];
        // The second argument: after the first comma at depth 0, up to the closing parenthesis.
        let mut depth = 0i32;
        let mut comma = None;
        for (k, c) in args.char_indices() {
            match c {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' if depth == 0 => break,
                ')' | ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    comma = Some(k);
                    break;
                }
                _ => {}
            }
        }
        let Some(comma) = comma else { continue };
        let second: String = args[comma + 1..].chars().take_while(|&c| c != ')').collect();
        if let Ok(n) = second.trim().parse::<usize>() {
            places.push(n);
        }
    }
    places
}

/// The definitions a card's `CARD` brings: its faces and halves.
fn faces(card: &'static CardDef, into: &mut Vec<&'static CardDef>) {
    into.push(card);
    for face in [card.back, card.adventure, card.omen, card.bestow].into_iter().flatten() {
        faces(face, into);
    }
    if let Some((left, right)) = card.split {
        faces(left, into);
        faces(right, into);
    }
}

#[test]
fn every_delayed_ability_a_card_names_is_listed() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cards");
    let mut stems: Vec<(String, String)> = fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs") && p.file_stem().is_some_and(|s| s != "mod"))
        .map(|p| (p.file_stem().unwrap().to_string_lossy().to_string(), fs::read_to_string(&p).unwrap()))
        .filter(|(_, text)| text.contains("pub static CARD:"))
        .collect();
    stems.sort();
    let files = cantrip_cards::files();
    assert_eq!(stems.len(), files.len(), "the card files and the registry's files disagree");
    let mut wrong = Vec::new();
    let mut checked = 0;
    for ((stem, text), (card, extra)) in stems.iter().zip(files) {
        let mut defs = Vec::new();
        faces(card, &mut defs);
        for &e in extra {
            faces(e, &mut defs);
        }
        let longest = defs.iter().map(|d| d.delayed.len()).max().unwrap_or(0);
        let code = without_comments(text);
        for n in literal_places(&code) {
            checked += 1;
            if n >= longest {
                wrong.push(format!("{stem}: names delayed ability {n}, but lists {longest}"));
            }
        }
        if code.contains("earthbend::effect") || code.contains("earthbend::enters") {
            checked += 1;
            if !code.split("delayed:").skip(1).any(|list| list.trim_start().trim_start_matches("&[").trim_start().starts_with("earthbend::RETURN")) {
                wrong.push(format!("{stem}: earthbends, but doesn't list earthbend::RETURN as its delayed ability 0"));
            }
        }
    }
    assert!(checked > 50, "no delayed ability references were found: the test reads nothing");
    assert!(wrong.is_empty(), "cards that name delayed abilities they don't list:\n{}", wrong.join("\n"));
}

#[test]
fn the_check_reads_literal_places_only() {
    assert_eq!(literal_places("g.reflexive_trigger(a, b, AbilityRef::delayed(this, 1), Target::None);"), vec![1]);
    assert_eq!(literal_places("AbilityRef::delayed(g.card_id(&CARD), 0)"), vec![0]);
    assert!(literal_places("AbilityRef::delayed(back, has as u8)").is_empty());
}
