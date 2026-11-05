use cantrip_api::*;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}.")],
    ..oracle::RABANASTRE_ROYAL_CITY
};
