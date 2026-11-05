use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(BEGINNING_OF_YOUR_COMBAT, effects::exile_first_target, "At the beginning of combat on your turn, exile up to one target card from a graveyard.")
        .targets(&[targets::CARD_IN_A_GRAVEYARD.optional()])],
    ..oracle::STARTLED_RELIC_SLOTH
};
