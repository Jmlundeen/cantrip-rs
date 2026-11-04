//! "This ability can't be copied" (CR 101.2, 707.10) is declared by a card, not read from its words:
//! `ActivatedDef::cant_be_copied` (`ability(..).cant_be_copied()`). A flag a card can forget is a rules deviation waiting to
//! happen, so this test holds each activated ability's flag to its own text, reminder text left out: the flag is set exactly
//! where the ability says the words. Gogo, Master of Mimicry says them.

use cantrip_api::*;

const WORDS: &str = "this ability can't be copied";

/// An ability's text, reminder text left out, lowercased, with straight apostrophes.
fn words(text: &str) -> String {
    let mut out = String::new();
    let mut depth = 0;
    for ch in text.chars() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if depth == 0 => out.push(ch),
            _ => {}
        }
    }
    out.to_lowercase().replace('\u{2019}', "'")
}

#[test]
fn cant_be_copied_is_declared_exactly_where_the_ability_says_it() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let mut said = 0;
    for card in table.cards.iter() {
        for ability in card.abilities {
            let AbilityDef::Activated(a) = ability else { continue };
            let says_it = words(a.text).contains(WORDS);
            said += usize::from(says_it);
            if a.cant_be_copied != says_it {
                wrong.push(format!("{}: the ability \"{}\" has cant_be_copied {}, its text {}", card.name, a.text, a.cant_be_copied, if says_it { "says it" } else { "doesn't" }));
            }
        }
    }
    assert!(said > 0, "no activated ability read as saying \"This ability can't be copied\"");
    assert!(wrong.is_empty(), "\"This ability can't be copied\" declared otherwise than the text says:\n{}", wrong.join("\n"));
}
