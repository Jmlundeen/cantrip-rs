use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &lands::bounce_land::<{ lands::U }, { lands::B }>("{T}: Add {U}{B}."),
    ..oracle::DIMIR_AQUEDUCT
};
