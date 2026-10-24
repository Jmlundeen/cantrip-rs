use cantrip_api::*;
use cantrip_mechanics::effects::{scry_two, this_gains_indestructible_then_tap};
use cantrip_mechanics::enlist::{ENLIST, THIS_ENLISTS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ENLIST,
        triggered(THIS_ENLISTS, scry_two, "Whenever this creature enlists a creature, scry 2."),
        ability(discard_cards(1), this_gains_indestructible_then_tap, "Discard a card: This creature gains indestructible until end of turn. Tap it."),
    ],
    ..oracle::GUARDIAN_OF_NEW_BENALIA
};
