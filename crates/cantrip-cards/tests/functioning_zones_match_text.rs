//! Where a triggered ability functions (CR 113.6, `TriggeredDef::functions`) is the battlefield unless its trigger condition
//! can't trigger from there (CR 113.6k) - then the shared trigger says so ("when you cast this spell" is
//! `when_this_spell_is_cast`, functioning from the stack) - or its own text puts it in a zone (CR 113.6b, 113.6m), which
//! the card declares (`AbilityDef::functions_in`). Either way it is held here to the words: each triggered ability's zones
//! are what its `text` says, and the zones a card's abilities function in are what its Oracle text's trigger conditions
//! say. A card that forgets the declaration, or a matcher made with `when` for a condition that can't trigger from the
//! battlefield, fails here rather than never triggering.

use cantrip_api::*;

/// The keyword abilities whose trigger is "when you cast this spell" (CR 702.40a storm, 702.56a replicate, 702.60a ripple,
/// 702.85a cascade, 702.153a casualty), whose ability text may be the keyword alone.
const CAST_KEYWORDS: [&str; 5] = ["storm", "replicate", "ripple", "cascade", "casualty"];

/// The zones the words of a triggered ability say it functions in (CR 113.6), for a card named `name`:
///
/// - the stack, for an ability that says "this spell" - only a spell is one (CR 113.6k: "when you cast this spell", and a
///   reflexive "when you do" of an additional cost paid to cast it) - or is a keyword whose trigger is its cast;
/// - a graveyard, for one that triggers on its own card being put into a graveyard other than from the battlefield ("when
///   this card is put into a graveyard from anywhere", "... into your graveyard from your library": CR 113.6k);
/// - the zone it names for its own card, "this card from your graveyard", "if this card is in your graveyard", and the
///   like for a hand and exile (CR 113.6b, 113.6m);
/// - otherwise the battlefield (CR 113.6).
fn said_zones(text: &str, name: &str) -> ZoneKinds {
    let text = without_reminders(&text.to_ascii_lowercase().replace('\u{2014}', "-"));
    let name = name.to_ascii_lowercase();
    let condition = text.split(", ").next().unwrap_or_default();
    let condition = condition.split(" - ").last().unwrap_or_default();
    if condition.contains("cast this spell")
        || (condition == "when you do" && text.contains("this spell"))
        || CAST_KEYWORDS.iter().any(|k| text.starts_with(k))
    {
        return ON_THE_STACK;
    }
    for verb in [" is put into a graveyard", " is put into your graveyard"] {
        if let Some(at) = condition.find(verb) {
            let subject = condition[..at].trim_start_matches("whenever ").trim_start_matches("when ");
            let itself = subject.starts_with("this ") || (!subject.is_empty() && name.starts_with(subject));
            if itself && !condition[at..].contains("from the battlefield") {
                return IN_A_GRAVEYARD;
            }
        }
    }
    let mut zones = ZoneKinds::NONE;
    for (words, zone) in [("your graveyard", ZoneKind::Graveyard), ("your hand", ZoneKind::Hand), ("exile", ZoneKind::Exile)] {
        if text.contains(&format!("this card from {words}")) || text.contains(&format!("this card is in {words}")) {
            zones = zones.with(zone);
        }
    }
    if zones == ZoneKinds::NONE { ON_THE_BATTLEFIELD } else { zones }
}

/// The text without its reminder text, which says what a keyword does rather than being an ability (CR 207.2).
fn without_reminders(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0usize;
    for c in text.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

/// Whether an Oracle text has a keyword whose trigger is its cast, or gives one to spells, outside quotes (quoted, it is an
/// ability of what the card makes, an emblem's): "Cascade", "Spells you cast have ripple 4" (Thrumming Stone).
fn cast_keyword(text: &str) -> bool {
    let text = text.to_ascii_lowercase();
    text.split(['"', '\u{201c}', '\u{201d}']).step_by(2).flat_map(|part| part.split('\n')).any(|line| {
        let line = without_reminders(line);
        CAST_KEYWORDS.iter().any(|k| line.starts_with(k) || line.contains(&format!(" have {k}")) || line.contains(&format!(" has {k}")))
    })
}

/// The triggered abilities of an Oracle text: each line, quoted ability and parenthesised reminder cut into sentences, a
/// sentence that starts with "when", "whenever" or "at" (after an ability word's dash) beginning one and the sentences after
/// it in the same part continuing it ("If you do, return this card from your graveyard ..."). A delayed trigger an ability
/// creates ("at the beginning of the next end step") is no ability of its card and functions from no zone (CR 603.7).
fn oracle_abilities(text: &str) -> Vec<String> {
    let text = text.to_ascii_lowercase().replace('\u{2014}', "-");
    let mut abilities = Vec::new();
    for part in text.split(['\n', '(', ')', '"', '\u{201c}', '\u{201d}']) {
        let mut current: Option<String> = None;
        for sentence in part.split(". ") {
            let s = sentence.trim();
            let s = match s.find(" - ") {
                Some(at) if s[..at].split(' ').count() <= 3 => s[at + 3..].trim(),
                _ => s,
            };
            if s.starts_with("when ") || s.starts_with("whenever ") || s.starts_with("at ") {
                abilities.extend(current.take());
                let delayed = s.starts_with("at the beginning of the next") || s.starts_with("at the beginning of your next");
                current = (!delayed).then(|| s.to_string());
            } else if let Some(ability) = current.as_mut() {
                ability.push_str(". ");
                ability.push_str(s);
            }
        }
        abilities.extend(current);
    }
    abilities
}

/// The zones, by name, for a failure's message.
fn names(zones: ZoneKinds) -> Vec<&'static str> {
    [ZoneKind::Battlefield, ZoneKind::Stack, ZoneKind::Hand, ZoneKind::Graveyard, ZoneKind::Exile, ZoneKind::Library, ZoneKind::Command]
        .into_iter()
        .filter(|&z| zones.contains(z))
        .map(ZoneKind::name)
        .collect()
}

fn off_the_battlefield(zones: ZoneKinds) -> ZoneKinds {
    ZoneKinds(zones.0 & !ON_THE_BATTLEFIELD.0)
}

#[test]
fn functioning_zones_are_what_the_words_say() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let (mut stack, mut graveyard) = (0, 0);
    for card in &table.cards {
        let mut declared = ZoneKinds::NONE;
        for a in card.abilities {
            let AbilityDef::Triggered(t) = a else { continue };
            // A state trigger's condition is a state of the permanent (CR 603.8), never a zone's.
            if t.state.is_some() {
                continue;
            }
            let said = said_zones(t.text, card.name);
            if t.functions != said {
                wrong.push(format!("{}: \"{}\" functions in {:?}, its words say {:?}", card.name, t.text, names(t.functions), names(said)));
            }
            declared = declared.union(off_the_battlefield(t.functions));
        }
        // The abilities it gives spells (Thrumming Stone's ripple): one a spell has functions where a spell's does.
        for a in card.delayed {
            if let AbilityDef::Triggered(t) = a
                && t.functions == ON_THE_STACK
            {
                declared = declared.union(ON_THE_STACK);
            }
        }
        if card.text.is_empty() {
            continue;
        }
        let mut oracle = oracle_abilities(card.text).iter().fold(ZoneKinds::NONE, |z, a| z.union(off_the_battlefield(said_zones(a, card.name))));
        if cast_keyword(card.text) {
            oracle = oracle.union(ON_THE_STACK);
        }
        stack += usize::from(oracle.contains(ZoneKind::Stack));
        graveyard += usize::from(oracle.contains(ZoneKind::Graveyard));
        if oracle != declared {
            wrong.push(format!("{}: its Oracle text's triggered abilities function off the battlefield in {:?}, its abilities in {:?}", card.name, names(oracle), names(declared)));
        }
    }
    // The reading finds them at all: Emrakul, the Promised End's cast trigger, Bloodghast's return from the graveyard.
    assert!(stack > 0 && graveyard > 0, "no Oracle text read as functioning on the stack or in a graveyard");
    assert!(wrong.is_empty(), "triggered abilities whose zones differ from their words:\n{}", wrong.join("\n"));
}
