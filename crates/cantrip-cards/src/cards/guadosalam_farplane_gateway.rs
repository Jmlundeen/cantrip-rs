use cantrip_api::*;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue]), "{T}: Add {G} or {U}.")],
    ..oracle::GUADOSALAM_FARPLANE_GATEWAY
};
