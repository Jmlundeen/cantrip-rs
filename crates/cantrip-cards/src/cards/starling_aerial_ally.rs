use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

// Flying is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::first_target_gains_flying, "When Starling enters, another target creature you control gains flying until end of turn.")
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::STARLING_AERIAL_ALLY
};
