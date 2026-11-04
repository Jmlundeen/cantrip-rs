use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
                cost("{U}{U}"),
                effects::tap_targets,
                "{U}{U}: Tap target creature without flying.",
            )
            .targets(&[targets::CREATURE_WITHOUT_FLYING]),
    ],
    ..oracle::FLOOD
};
