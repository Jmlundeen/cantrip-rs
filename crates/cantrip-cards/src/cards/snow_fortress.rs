use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}"), effects::this_gets::<1, 0>, "{1}: This creature gets +1/+0 until end of turn."),
        ability(cost("{1}"), effects::this_gets::<0, 1>, "{1}: This creature gets +0/+1 until end of turn."),
        ability(cost("{3}"), effects::damage_first_target::<1>, "{3}: This creature deals 1 damage to target creature without flying that's attacking you.")
            .targets(&[targets::CREATURE_WITHOUT_FLYING_ATTACKING_YOU]),
    ],
    ..oracle::SNOW_FORTRESS
};
