use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

/// Sunpetal Grove: "This land enters tapped unless you control a Forest or a Plains" (CR 614.1d); "{T}: Add {G} or
/// {W}."
pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Forest) }, { sub(Subtype::Plains) }>),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Green, Color::White]), "{T}: Add {G} or {W}.")],
    ..oracle::SUNPETAL_GROVE
};
