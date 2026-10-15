use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &lands::bounce_land::<{ lands::B }, { lands::R }>("{T}: Add {B}{R}."),
    ..oracle::RAKDOS_CARNARIUM
};
