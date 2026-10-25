use cantrip_api::*;
use cantrip_mechanics::replacements::PLUS_ONE_COUNTERS_ON_YOUR_CREATURES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    replacements: &[PLUS_ONE_COUNTERS_ON_YOUR_CREATURES],
    ..oracle::HARDENED_SCALES
};
