use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // The target is mandatory, so the Kavu itself is the target when it is the only creature.
    abilities: &[triggered(THIS_ENTERS, effects::damage_first_target::<4>, "When this creature enters, it deals 4 damage to target creature.")
        .targets(&[targets::CREATURE])],
    ..oracle::FLAMETONGUE_KAVU
};
