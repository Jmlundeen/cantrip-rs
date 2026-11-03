use cantrip_api::*;
use cantrip_mechanics::combat;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    block_rules: &[combat::power_three_or_greater_cant_block],
    ..oracle::AMROU_KITHKIN
};
