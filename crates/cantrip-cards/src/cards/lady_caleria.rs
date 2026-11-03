use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::damage_first_target::<3>, "{T}: Lady Caleria deals 3 damage to target attacking or blocking creature.")
            .targets(&[targets::ATTACKING_OR_BLOCKING_CREATURE]),
    ],
    ..oracle::LADY_CALERIA
};
