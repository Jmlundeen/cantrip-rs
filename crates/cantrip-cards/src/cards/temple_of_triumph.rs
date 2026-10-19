use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[lands::SCRY_LAND, mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}.")],
    ..oracle::TEMPLE_OF_TRIUMPH
};
