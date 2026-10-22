use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Island) }, { sub(Subtype::Mountain) }>),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Blue, Color::Red]), "{T}: Add {U} or {R}.")],
    ..oracle::SULFUR_FALLS
};
