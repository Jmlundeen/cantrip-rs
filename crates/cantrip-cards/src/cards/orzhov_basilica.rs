use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &lands::bounce_land(Mana::W, Mana::B, "{T}: Add {W}{B}."),
    ..oracle::ORZHOV_BASILICA
};
