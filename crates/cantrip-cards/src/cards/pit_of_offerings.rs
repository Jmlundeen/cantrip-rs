use cantrip_api::*;
use cantrip_mechanics::{mana, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Exile up to three target cards from graveyards" (CR 115.1d, 406.3): the cards that are still legal targets (CR 608.2b), from any
/// graveyards, exiled together with this land (CR 607.2a), which is what "the exiled cards" of its last ability means. Cards exiled
/// by an earlier trigger of an earlier Pit stay with that object, and a Pit that left and came back is a new object with none
/// (CR 400.7).
fn exile_up_to_three(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    g.exile_with(&cards, item.object, false);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add(Mana::C), "{T}: Add {C}."),
        mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any of the exiled cards' colors.")
            .colors_from(mana::colors_of_cards_exiled_with_this),
        triggered(THIS_ENTERS, exile_up_to_three, "When this land enters, exile up to three target cards from graveyards.")
            .targets(&[targets::CARD_IN_A_GRAVEYARD.between(0, 3)]),
    ],
    ..oracle::PIT_OF_OFFERINGS
};
