use cantrip_api::*;
use cantrip_mechanics::replacements::PLUS_ONE_COUNTERS_ON_YOUR_CREATURES;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    replacements: &[PLUS_ONE_COUNTERS_ON_YOUR_CREATURES],
    // "{6}: Put a +1/+1 counter on target legendary creature" (CR 602.2, 115.1c, 122.1a).
    abilities: &[ability(cost("{6}"), effects::plus_one_counter_on_target, "{6}: Put a +1/+1 counter on target legendary creature.")
        .targets(&[targets::LEGENDARY_CREATURE])],
    ..oracle::YOSHIMARU_BELOVED_COMPANION
};
