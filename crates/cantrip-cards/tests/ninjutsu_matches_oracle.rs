//! Ninjutsu (CR 702.49a) and commander ninjutsu, a variant of it (CR 702.49d), are the class "ninjutsu abilities" a card can
//! name (Silver-Fur Master). Which abilities those are is `ActivatedDef::ninjutsu`, set by the keyword's shared mechanics
//! (`ninjutsu::ninjutsu`); a card that wrote one by hand would forget it, and what names the class would miss it. So the flag
//! is held here to the Oracle text, both ways: each line that is the keyword is an ability with the flag, and each ability
//! with the flag is such a line. An ability a card lists only to give (`CardDef::delayed`: "each creature card in your hand has
//! ninjutsu {2}{U}{B}", Satoru Umezawa) is no line of its own; its own text says what it is, and the flag is held to that.

use cantrip_api::*;

/// Whether `line` is the keyword itself, the words before its cost: not a line that only speaks of it.
fn is_ninjutsu_line(line: &str) -> bool {
    let words = line.split(['{', '\u{2014}', '(']).next().unwrap_or_default().trim().to_lowercase();
    words == "ninjutsu" || words == "commander ninjutsu"
}

#[test]
fn ninjutsu_abilities_are_declared_exactly_where_the_oracle_text_says_them() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let mut said = 0;
    for card in table.cards.iter() {
        let lines = card.text.lines().filter(|l| is_ninjutsu_line(l)).count();
        said += lines;
        let mut flagged = 0;
        for ability in card.abilities {
            let AbilityDef::Activated(a) = ability else { continue };
            let says = is_ninjutsu_line(a.text);
            flagged += usize::from(a.ninjutsu);
            if a.ninjutsu != says {
                wrong.push(format!("{}: the ability \"{}\" has ninjutsu {}, its text {}", card.name, a.text, a.ninjutsu, if says { "says it" } else { "doesn't" }));
            }
        }
        for ability in card.delayed {
            let AbilityDef::Activated(a) = ability else { continue };
            if a.ninjutsu != is_ninjutsu_line(a.text) {
                wrong.push(format!("{}: the ability \"{}\" it gives has ninjutsu {}", card.name, a.text, a.ninjutsu));
            }
        }
        if flagged != lines {
            wrong.push(format!("{}: {lines} ninjutsu lines in its Oracle text, {flagged} ninjutsu abilities in its file", card.name));
        }
    }
    assert!(said > 0, "no Oracle text read as a ninjutsu keyword");
    assert!(wrong.is_empty(), "ninjutsu declared otherwise than the Oracle text says:\n{}", wrong.join("\n"));
}

#[test]
fn each_ninjutsu_line_is_read_as_one() {
    for (line, ninjutsu) in [
        ("Ninjutsu {U} ({U}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand tapped and attacking.)", true),
        ("Ninjutsu {2}{U}{U}", true),
        ("Commander ninjutsu {3}{U}{B} ({3}{U}{B}, Return an unblocked attacker you control to hand: Put this card onto the battlefield from your hand or the command zone tapped and attacking.)", true),
        ("Ninjutsu abilities you activate cost {1} less to activate.", false),
        ("Each creature card in your hand has ninjutsu {2}{U}{B}.", false),
    ] {
        assert_eq!(is_ninjutsu_line(line), ninjutsu, "{line}");
    }
}
