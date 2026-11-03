use cantrip_api::*;
use cantrip_mechanics::{flashback, targets};
use cantrip_oracle as oracle;

/// "Choose up to three target cards in graveyards. The owners of those cards shuffle them into their libraries. You gain 2 life." (CR 608.2c, 701.24a).
fn shuffle_cards_into_libraries_and_gain_2(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards = targets::legal_objects(g, item);
    g.put_into_library(&cards, LibraryPlace::ShuffledIn);
    g.end_sentence();
    g.gain_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CARD_IN_A_GRAVEYARD.between(0, 3)], shuffle_cards_into_libraries_and_gain_2),
    ways: &[flashback::way(cost("{1}{G}"), "Flashback {1}{G}")],
    ..oracle::TURN_THE_EARTH
};
