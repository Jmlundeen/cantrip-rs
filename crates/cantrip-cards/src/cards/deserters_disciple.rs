use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(TAP, effects::target_creature_cant_be_blocked_this_turn, "{T}: Another target creature you control with power 2 or less can't be blocked this turn.")
        .targets(&[targets::OTHER_CREATURE_YOU_CONTROL_WITH_POWER_2_OR_LESS])],
    ..oracle::DESERTERS_DISCIPLE
};
