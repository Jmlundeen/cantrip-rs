use cantrip_api::*;
use cantrip_mechanics::statics::NONBASIC_LANDS_ARE_MOUNTAINS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { statics: &[NONBASIC_LANDS_ARE_MOUNTAINS], ..oracle::BLOOD_MOON };
