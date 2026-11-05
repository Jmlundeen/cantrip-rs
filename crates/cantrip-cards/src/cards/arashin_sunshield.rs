use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Exile up to two target cards from a single graveyard" (CR 115.1, 608.2b): the cards still in that graveyard are exiled
/// together; a target that became illegal is left alone.
fn exile_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.exile_all(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, exile_cards, "When this creature enters, exile up to two target cards from a single graveyard.").targets(&[
            targets::CARD_IN_A_GRAVEYARD.optional().labelled("First card to exile (up to two, from a single graveyard)"),
            targets::ANOTHER_CARD_IN_THE_SAME_GRAVEYARD.optional().labelled("Second card to exile, from the same graveyard"),
        ]),
        ability(cost("{W}").and(TAP), effects::tap_targets, "{W}, {T}: Tap target creature.").targets(&[targets::CREATURE]),
    ],
    ..oracle::ARASHIN_SUNSHIELD
};
