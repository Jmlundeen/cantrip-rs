use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{8}").and(TAP),
                effects::damage_first_target::<4>,
                "{8}, {T}: This artifact deals 4 damage to any target.",
            )
            .targets(&[targets::ANY]),
    ],
    ..oracle::ALADDINS_RING
};
