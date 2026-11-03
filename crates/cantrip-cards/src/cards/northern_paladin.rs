use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{W}{W}").and(TAP),
                effects::destroy_first_target,
                "{W}{W}, {T}: Destroy target black permanent.",
            )
            .targets(&[targets::permanent_of_color::<{ Color::Black as u8 }>()]),
    ],
    ..oracle::NORTHERN_PALADIN
};
