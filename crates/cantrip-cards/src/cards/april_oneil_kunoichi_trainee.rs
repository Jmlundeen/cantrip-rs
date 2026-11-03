use cantrip_api::*;
use cantrip_mechanics::{combat, effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::scry_two, "When April O'Neil enters, scry 2.")],
    block_rules: &[combat::power_three_or_greater_cant_block],
    ..oracle::APRIL_ONEIL_KUNOICHI_TRAINEE
};
