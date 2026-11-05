use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}, {T}"), library::put_first_target_on_the_bottom, "{2}, {T}: Put target card from a graveyard on the bottom of its owner's library.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD])],
    ..oracle::JADE_CAST_SENTINEL
};
