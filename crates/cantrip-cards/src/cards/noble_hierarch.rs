use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1).colors(ColorSet(0b10011)), "{T}: Add {G}, {W}, or {U}."),
        triggers::EXALTED,
    ],
    ..oracle::NOBLE_HIERARCH
};
