use cantrip_api::*;
use cantrip_mechanics::mana::enters_tapped;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(
            TAP,
            add_one_of(&[Color::White, Color::Blue, Color::Black]),
            "{T}: Add {W}, {U}, or {B}.",
        ),
    ],
    enters: Some(enters_tapped),
    ..oracle::ARCANE_SANCTUM
};
