use cantrip_api::*;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        state_triggered(lands::you_control_no_islands, effects::sacrifice_self, "When you control no Islands, sacrifice Skeleton Ship."),
        ability(TAP, effects::minus_one_counters_on_target::<1>, "{T}: Put a -1/-1 counter on target creature.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::SKELETON_SHIP
};
