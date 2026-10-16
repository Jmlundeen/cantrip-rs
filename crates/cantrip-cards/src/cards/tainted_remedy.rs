use cantrip_api::*;
use cantrip_mechanics::replacements::OPPONENTS_LOSE_LIFE_INSTEAD_OF_GAINING;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { replacements: &[OPPONENTS_LOSE_LIFE_INSTEAD_OF_GAINING], ..oracle::TAINTED_REMEDY };
