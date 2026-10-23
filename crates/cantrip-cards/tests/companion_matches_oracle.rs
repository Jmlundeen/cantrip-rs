//! Companion (CR 702.139a) is a condition a card declares (`CardDef::companion`), not something read from its words. A card
//! that forgot it could never be revealed as a companion (CR 103.2b), so this test holds the declaration to the card's
//! Oracle text: a card has a condition exactly where a line of its text begins "Companion -".

use cantrip_api::*;

#[test]
fn companion_is_declared_exactly_where_the_oracle_text_says_it() {
    let table = CardTable::new(cantrip_cards::all());
    let mut wrong = Vec::new();
    let mut said = 0;
    for card in table.cards.iter() {
        let says = card.text.lines().any(|line| line.trim_start().starts_with("Companion \u{2014}") || line.trim_start().starts_with("Companion -"));
        said += usize::from(says);
        if says != card.companion.is_some() {
            wrong.push(if says {
                format!("{} says \"Companion\" but declares no condition (`companion`)", card.name)
            } else {
                format!("{} declares a companion condition its text does not have", card.name)
            });
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
    assert!(said > 0, "no card with companion was found: the test reads nothing");
}
