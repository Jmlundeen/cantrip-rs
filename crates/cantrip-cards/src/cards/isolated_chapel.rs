use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Plains) }, { sub(Subtype::Swamp) }>),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::White, Color::Black]), "{T}: Add {W} or {B}.")],
    ..oracle::ISOLATED_CHAPEL
};
