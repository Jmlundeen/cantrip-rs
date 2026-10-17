use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[lands::SCRY_LAND, mana_ability(TAP, add_one_of(&[Color::Blue, Color::Black]), "{T}: Add {U} or {B}.")],
    ..oracle::TEMPLE_OF_DECEIT
};
