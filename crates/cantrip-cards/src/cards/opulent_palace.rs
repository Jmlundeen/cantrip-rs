use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Black, Color::Green, Color::Blue]), "{T}: Add {B}, {G}, or {U}.")],
    ..oracle::OPULENT_PALACE
};
