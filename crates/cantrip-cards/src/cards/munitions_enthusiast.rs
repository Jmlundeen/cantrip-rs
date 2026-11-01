use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_DIES};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_DIES, effects::damage_first_target::<1>, "When this creature dies, it deals 1 damage to any target.")
        .targets(&[targets::ANY])],
    ..oracle::MUNITIONS_ENTHUSIAST
};
