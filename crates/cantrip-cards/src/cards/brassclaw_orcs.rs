use cantrip_api::*;
use cantrip_mechanics::combat;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    block_rules: &[combat::this_cant_block_power_two_or_greater],
    ..oracle::BRASSCLAW_ORCS
};
