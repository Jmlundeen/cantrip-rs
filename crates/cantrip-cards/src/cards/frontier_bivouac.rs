use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue, Color::Red]), "{T}: Add {G}, {U}, or {R}.")],
    ..oracle::FRONTIER_BIVOUAC
};
