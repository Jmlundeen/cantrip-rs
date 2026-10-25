//! A graveyard is kept in order (CR 404.2), and the owner of cards put into one together arranges them (CR 404.3). The
//! core asks that only while some object of the game reads a graveyard's order (`CardDef::reads_graveyard_order`), so a
//! card that reads it and does not say so would leave its owner unasked: a rules deviation. This test holds the flag to
//! the card's Oracle text (`CardDef::text` and its faces'), reminder text left out: a sentence about a graveyard that
//! places something in it - "the top creature card of your graveyard", "on top of your graveyard", "with a creature card
//! directly above it" - reads its order. A "top" whose place is a library ("the top card of your library into your
//! graveyard", "from your graveyard on top of your library") does not.
//!
//! The other way round, no card declares the flag without saying the words.

use cantrip_api::*;

/// Each sentence of an Oracle text, reminder text left out, lowercased, with straight apostrophes.
fn sentences(text: &str) -> Vec<String> {
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
    out.to_lowercase().replace('\u{2019}', "'").split(['.', '\n']).map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Whether `sentence` reads the order of a graveyard.
fn reads_order(sentence: &str) -> bool {
    if !sentence.contains("graveyard") {
        return false;
    }
    if ["above it", "below it", "directly above", "directly below"].iter().any(|w| sentence.contains(w)) {
        return true;
    }
    // Each "top" or "bottom": the zone it is of is the first graveyard or library named after it.
    ["top", "bottom"].iter().any(|end| {
        sentence.match_indices(end).any(|(at, _)| {
            let after = &sentence[at..];
            match (after.find("graveyard"), after.find("library")) {
                (Some(g), Some(l)) => g < l,
                (Some(_), None) => true,
                _ => false,
            }
        })
    })
}

fn texts(card: &CardDef) -> Vec<&'static str> {
    let mut all = vec![card.text];
    for face in [card.back, card.adventure, card.omen, card.split.map(|(l, _)| l), card.split.map(|(_, r)| r)].into_iter().flatten() {
        all.push(face.text);
    }
    all
}

#[test]
fn reads_graveyard_order_is_declared_exactly_where_the_oracle_text_says_it() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let mut said = 0;
    for card in table.cards.iter() {
        let says = texts(card).iter().any(|t| sentences(t).iter().any(|s| reads_order(s)));
        said += usize::from(says);
        if card.reads_graveyard_order != says {
            wrong.push(format!(
                "{}: CardDef::reads_graveyard_order is {}, the Oracle text {}",
                card.name,
                card.reads_graveyard_order,
                if says { "reads a graveyard's order" } else { "doesn't" }
            ));
        }
    }
    // Corpse Dance and Shallow Grave say it at least: the reading finds the words at all.
    assert!(said > 0, "no Oracle text read as reading a graveyard's order");
    assert!(wrong.is_empty(), "graveyard order declared otherwise than the Oracle text says:\n{}", wrong.join("\n"));
}

#[test]
fn each_sentence_is_read_for_a_graveyard_order() {
    for (sentence, reads) in [
        ("Return the top creature card of your graveyard to the battlefield.", true),
        ("At the beginning of the end step, if this card is in your graveyard with a creature card directly above it, you may return it to your hand.", true),
        ("Put the top card of your graveyard on the bottom of your library.", true),
        ("Mill two cards. (Put the top two cards of your library into your graveyard.)", false),
        ("Put the top two cards of your library into your graveyard.", false),
        ("When this creature dies, you may exile it. If you do, put another target card from your graveyard on top of your library.", false),
        ("Reveal the top five cards of your library. Put a land card from among them onto the battlefield and the rest into your graveyard.", false),
        ("Return target creature card from your graveyard to the battlefield.", false),
    ] {
        assert_eq!(sentences(sentence).iter().any(|s| reads_order(s)), reads, "{sentence}");
    }
}
