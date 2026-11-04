use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[
        mana_ability(TAP, add_one_of(&[Color::Black, Color::Red]), "{T}: Add {B} or {R}."),
        lands::surveil_ability(cost("{4}, {T}"), "{4}, {T}: Surveil 1."),
    ],
    ..oracle::OMINOUS_ASYLUM
};
