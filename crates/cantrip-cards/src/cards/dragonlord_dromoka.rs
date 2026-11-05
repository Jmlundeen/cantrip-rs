use cantrip_api::*;
use cantrip_mechanics::statics;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    cast_rules: &[statics::opponents_cant_cast_spells_during_your_turn],
    ..oracle::DRAGONLORD_DROMOKA
};
