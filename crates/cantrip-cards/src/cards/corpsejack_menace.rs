use cantrip_api::*;
use cantrip_mechanics::replacements::DOUBLE_PLUS_ONE_COUNTERS_ON_YOUR_CREATURES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { replacements: &[DOUBLE_PLUS_ONE_COUNTERS_ON_YOUR_CREATURES], ..oracle::CORPSEJACK_MENACE };
