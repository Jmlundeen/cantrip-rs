use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::ANOTHER_CREATURE_YOU_CONTROL_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        ANOTHER_CREATURE_YOU_CONTROL_ENTERS,
        effects::damage_first_target::<1>,
        "Alliance \u{2014} Whenever another creature you control enters, Raphael deals 1 damage to target opponent.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::RAPHAEL_TOUGH_TURTLE
};
