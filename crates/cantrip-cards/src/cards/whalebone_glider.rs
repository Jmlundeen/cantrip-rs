use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(
            cost("{2}").and(TAP),
            effects::first_target_gains_flying,
            "{2}, {T}: Target creature with power 3 or less gains flying until end of turn.",
        )
        .targets(&[target(targets::creature_with_power_at_most::<3>)]),
    ],
    ..oracle::WHALEBONE_GLIDER
};
