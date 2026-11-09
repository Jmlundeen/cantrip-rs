use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    targeting_rules: &[targets::CANT_BE_TARGET_OF_AURA_SPELLS],
    ..oracle::BARTEL_RUNEAXE
};
