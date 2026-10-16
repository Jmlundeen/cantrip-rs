use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mana_ability(TAP, add_any_color(1).colors(ColorSet(0b11100)), "{T}: Add {B}, {R}, or {G}."),
        triggers::EXALTED,
    ],
    ..oracle::IGNOBLE_HIERARCH
};
