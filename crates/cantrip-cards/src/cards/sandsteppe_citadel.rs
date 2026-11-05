use cantrip_api::*;
use cantrip_mechanics::mana;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Black, Color::Green]), "{T}: Add {W}, {B}, or {G}.")],
    ..oracle::SANDSTEPPE_CITADEL
};
