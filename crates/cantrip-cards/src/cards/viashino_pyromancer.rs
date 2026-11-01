use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::damage_first_target::<2>, "When this creature enters, it deals 2 damage to target player or planeswalker.")
        .targets(&[targets::PLAYER_OR_PLANESWALKER])],
    ..oracle::VIASHINO_PYROMANCER
};
