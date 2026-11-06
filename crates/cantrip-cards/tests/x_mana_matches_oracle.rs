//! "Spend only black mana on X" (CR 107.3, 601.2h) is declared by a card, not read from its words: `CardDef::x_mana` for the
//! X of a spell's mana cost or of its kicker (Drain Life, Emblazoned Golem), `ActivatedDef::x_mana` (`ability(..).x_mana(..)`)
//! for the X of an activation cost (Crypt Rats, Crimson Hellkite, Atalya, Samite Master, whose modes each say it). A
//! restriction a card can forget is a rules deviation waiting to happen, so this test holds the declarations to the card's
//! Oracle text (`CardDef::text`), reminder text left out: an activated ability whose text says it has it, a card whose
//! other lines say it has it, and the restriction is the one the words name. The other way round, no card and no ability
//! declares one without saying the words.

use cantrip_api::*;

const WORDS: &str = "mana on x";

/// A text, reminder text left out, lowercased.
fn plain(text: &str) -> String {
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
    out.to_lowercase()
}

/// The restriction a text that says the words names, or `None` for words this test doesn't know.
fn named(text: &str) -> Option<XMana> {
    if text.contains("spend only colored mana on x. no more than one mana of each color may be spent this way") {
        return Some(XMana::COLORED_ONE_EACH);
    }
    let types = [("white", Mana::W), ("blue", Mana::U), ("black", Mana::B), ("red", Mana::R), ("green", Mana::G), ("colorless", Mana::C)];
    for (a, ma) in types {
        for (b, mb) in types {
            // Soul Burn's "Spend only black and/or red mana on X."
            if text.contains(&format!("spend only {a} and/or {b} mana on x")) {
                return Some(XMana::either(ma, mb));
            }
        }
        if text.contains(&format!("spend only {a} mana on x")) {
            return Some(XMana::only(ma));
        }
    }
    None
}

#[test]
fn x_mana_is_declared_exactly_where_the_oracle_text_says_it() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    for card in table.cards.iter() {
        let mut ability_lines = 0;
        for ability in card.abilities {
            let AbilityDef::Activated(a) = ability else { continue };
            let text = plain(a.text);
            let says = text.contains(WORDS);
            ability_lines += text.lines().filter(|l| l.contains(WORDS)).count();
            let expected = if says { named(&text) } else { Some(XMana::ANY) };
            match expected {
                None => wrong.push(format!("{}: the ability \"{}\" restricts X in words this test doesn't know", card.name, a.text)),
                Some(x) if x != a.x_mana => wrong.push(format!("{}: the ability \"{}\" has x_mana {:?}, its text says {x:?}", card.name, a.text, a.x_mana)),
                Some(_) => {}
            }
            if says && a.cost.mana.x == 0 {
                wrong.push(format!("{}: the ability \"{}\" restricts an X its cost has no {{X}} for", card.name, a.text));
            }
        }
        let text = plain(card.text);
        let card_lines: Vec<&str> = text.lines().filter(|l| l.contains(WORDS)).collect();
        // The lines an activated ability's text has are its own; the rest are the spell's.
        let spell_says = card_lines.len() > ability_lines;
        let expected = if spell_says { named(&text) } else { Some(XMana::ANY) };
        match expected {
            None => wrong.push(format!("{}: its text restricts X in words this test doesn't know", card.name)),
            Some(x) if x != card.x_mana => wrong.push(format!("{}: CardDef::x_mana is {:?}, its Oracle text says {x:?}", card.name, card.x_mana)),
            Some(_) => {}
        }
        if spell_says && card.cost.x == 0 && !card.kicker.is_some_and(|k| k.cost.x > 0) {
            wrong.push(format!("{}: restricts an X its mana cost and kicker have no {{X}} for", card.name));
        }
    }
    assert!(wrong.is_empty(), "\"Spend only ... mana on X\" declared otherwise than the Oracle text says:\n{}", wrong.join("\n"));
}

#[test]
fn each_restriction_is_read_as_the_mana_it_names() {
    assert_eq!(named(&plain("Spend only black mana on X.")), Some(XMana::only(Mana::B)));
    assert_eq!(named(&plain("{X}, {T}: This creature deals X damage to target creature. Spend only red mana on X.")), Some(XMana::only(Mana::R)));
    assert_eq!(named(&plain("Spend only black and/or red mana on X.")), Some(XMana::either(Mana::B, Mana::R)));
    assert_eq!(
        named(&plain("Spend only colored mana on X. No more than one mana of each color may be spent this way.")),
        Some(XMana::COLORED_ONE_EACH)
    );
    assert_eq!(named(&plain("Spend only mana produced by creatures on X.")), None);
}
