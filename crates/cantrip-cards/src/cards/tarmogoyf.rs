use cantrip_api::*;
use cantrip_mechanics::goyf;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { statics: &[goyf::SIZE], ..oracle::TARMOGOYF };
