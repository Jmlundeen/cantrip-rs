//! "X can't be 0" (CR 107.3a) is declared by a card, not read from its words: `CardDef::x_cant_be_zero` for the X of a
//! spell's mana cost or of its kicker, `ActivatedDef::x_cant_be_zero` (`ability(..).x_cant_be_zero()`) for the X of an
//! activation cost. A flag a card can forget is a rules deviation waiting to happen, so this test holds the flags to the
//! card's Oracle text (`CardDef::text`, generated from its JSON), line by line, reminder text left out:
//!
//! - a line that is the sentence alone ("X can't be 0.") governs the X of the spell's mana cost: the card's own flag, and
//!   its mana cost has an {X};
//! - "Kicker {X}. X can't be 0." governs the kicker's X: the card's own flag, and its kicker has an {X};
//! - an activated ability's line ("{X}{G}: ... X can't be 0.") governs that ability's X: the ability whose text says it
//!   has the flag, and its activation cost has an {X} in mana;
//! - any other X the sentence governs has no flag yet, and a card that prints one fails here until one is built for it:
//!   a mode's X ("• Put X +1/+1 counters ... X can't be 0.", Marath), the X of one way of casting it only ("If you cast
//!   this spell this way, X can't be 0.", Light Up the Night), a suspend cost's X ("Suspend X-{X}{W}{W}. X can't be 0."),
//!   an activation cost's X that is not mana ("Sacrifice X artifacts: ... X can't be 0.", Ruthless Technomancer).
//!
//! The other way round, no card and no ability declares the flag without saying the words.

use cantrip_api::*;

/// What X a line saying "X can't be 0" governs.
#[derive(Debug, PartialEq)]
enum Governs {
    /// The X of the spell's mana cost.
    ManaCost,
    /// The X of its kicker.
    Kicker,
    /// The X of an activated ability's mana cost.
    Ability,
    /// An X no flag covers yet.
    Uncovered,
}

const WORDS: &str = "x can't be 0";

/// Each line of an Oracle text, reminder text left out, lowercased, with straight apostrophes.
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| {
            let mut out = String::new();
            let mut depth = 0;
            for ch in line.chars() {
                match ch {
                    '(' => depth += 1,
                    ')' => depth -= 1,
                    _ if depth == 0 => out.push(ch),
                    _ => {}
                }
            }
            out.trim().to_lowercase().replace('\u{2019}', "'").replace('\u{2014}', "-")
        })
        .collect()
}

fn governs(line: &str) -> Governs {
    if line == "x can't be 0." {
        Governs::ManaCost
    } else if line == "kicker {x}. x can't be 0." {
        Governs::Kicker
    } else if line.starts_with('\u{2022}') || line.starts_with("suspend") || line.contains("this way") {
        Governs::Uncovered
    } else if line.find(": ").is_some_and(|colon| line[..colon].contains("{x}") && !line[..colon].contains('"')) {
        Governs::Ability
    } else {
        Governs::Uncovered
    }
}

#[test]
fn x_cant_be_zero_is_declared_exactly_where_the_oracle_text_says_it() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let mut said = 0;
    for card in table.cards.iter() {
        let says: Vec<(String, Governs)> = lines(card.text).into_iter().filter(|l| l.contains(WORDS)).map(|l| {
            let g = governs(&l);
            (l, g)
        }).collect();
        said += says.len();
        for (line, g) in &says {
            if *g == Governs::Uncovered {
                wrong.push(format!("{}: \"{line}\" governs an X no flag covers yet: build one for it", card.name));
            }
        }
        let spell = says.iter().any(|(_, g)| *g == Governs::ManaCost);
        let kicker = says.iter().any(|(_, g)| *g == Governs::Kicker);
        if card.x_cant_be_zero != (spell || kicker) {
            wrong.push(format!("{}: CardDef::x_cant_be_zero is {}, the Oracle text {}", card.name, card.x_cant_be_zero, if spell || kicker { "says it" } else { "doesn't" }));
        }
        if spell && card.cost.x == 0 {
            wrong.push(format!("{}: \"X can't be 0\" with no {{X}} in its mana cost", card.name));
        }
        if kicker && !card.kicker.is_some_and(|k| k.cost.x > 0) {
            wrong.push(format!("{}: \"Kicker {{X}}. X can't be 0.\" with no kicker {{X}}", card.name));
        }
        let lines_for_abilities = says.iter().filter(|(_, g)| *g == Governs::Ability).count();
        let mut abilities_saying = 0;
        for ability in card.abilities {
            let AbilityDef::Activated(a) = ability else { continue };
            let text = lines(a.text).join(" ");
            let says_it = text.contains(WORDS);
            abilities_saying += usize::from(says_it);
            if a.x_cant_be_zero != says_it {
                wrong.push(format!("{}: the ability \"{}\" has x_cant_be_zero {}, its text {}", card.name, a.text, a.x_cant_be_zero, if says_it { "says it" } else { "doesn't" }));
            }
            if a.x_cant_be_zero && a.cost.mana.x == 0 {
                wrong.push(format!("{}: the ability \"{}\" can't have X 0 with no {{X}} in its cost", card.name, a.text));
            }
        }
        if abilities_saying != lines_for_abilities {
            wrong.push(format!("{}: {lines_for_abilities} activated abilities say \"X can't be 0\" in its Oracle text, {abilities_saying} in its file", card.name));
        }
    }
    // Thieving Skydiver says it at least: the reading finds the words at all.
    assert!(said > 0, "no Oracle text read as saying \"X can't be 0\"");
    assert!(wrong.is_empty(), "\"X can't be 0\" declared otherwise than the Oracle text says:\n{}", wrong.join("\n"));
}

#[test]
fn each_x_cant_be_zero_line_is_read_as_the_x_it_governs() {
    for (line, governs_x) in [
        ("X can't be 0.", Governs::ManaCost),
        ("Kicker {X}. X can't be 0. (You may pay an additional {X} as you cast this spell.)", Governs::Kicker),
        ("{X}{G}: Until end of turn, this land becomes an X/X green Hydra creature. It's still a land. X can't be 0.", Governs::Ability),
        ("{X}, {T}: Target opponent mills a card, then repeats this process. X can't be 0.", Governs::Ability),
        ("\u{2022} Put X +1/+1 counters on target creature. X can't be 0.", Governs::Uncovered),
        ("Suspend X\u{2014}{X}{W}{W}. X can't be 0.", Governs::Uncovered),
        ("Flashback\u{2014}{3}{R}, Remove X loyalty counters from among planeswalkers you control. If you cast this spell this way, X can't be 0.", Governs::Uncovered),
        ("{2}{B}, Sacrifice X artifacts: Return target creature card with power X or less from your graveyard to the battlefield. X can't be 0.", Governs::Uncovered),
    ] {
        assert_eq!(governs(&lines(line)[0]), governs_x, "{line}");
    }
}
