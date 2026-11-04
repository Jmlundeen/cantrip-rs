use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::target_connives, "When this creature enters, target creature you control connives.")
        .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::MOB_LOOKOUT
};
