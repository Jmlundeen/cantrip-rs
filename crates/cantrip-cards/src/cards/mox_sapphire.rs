use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::mana;

pub static CARD: CardDef = CardDef {
    abilities: &[mana::ISLAND],
    ..oracle::MOX_SAPPHIRE
};
