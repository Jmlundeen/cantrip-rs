use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP.and(SACRIFICE_THIS), effects::damage_first_target::<1>, "{T}, Sacrifice this creature: It deals 1 damage to any target.").targets(&[targets::ANY])],
    ..oracle::FANATICAL_FIREBRAND
};
