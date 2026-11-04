use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{1}{G}"),
            effects::target_creature_gets::<0, 1>,
            "{1}{G}: Target blocking creature gets +0/+1 until end of turn.",
        )
        .targets(&[targets::BLOCKING_CREATURE]),
    ],
    ..oracle::ELVEN_FORTRESS
};
