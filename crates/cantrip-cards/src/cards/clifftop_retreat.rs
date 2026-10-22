use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Mountain) }, { sub(Subtype::Plains) }>),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Red, Color::White]), "{T}: Add {R} or {W}.")],
    ..oracle::CLIFFTOP_RETREAT
};
