use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { abilities: &[mana::TAP_FOR_ANY_COLOR], ..oracle::SYLVAN_CARYATID };
