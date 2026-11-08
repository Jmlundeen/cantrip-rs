use cantrip_api::*;
use cantrip_mechanics::lands::{enters_tapped_unless_you_control_either, sub};
use cantrip_oracle as oracle;

/// "This land enters tapped unless you control a Forest or an Island." (CR 614.1, 305.1): the condition is judged on the
/// permanents of the land's own controller as it enters, so the other player's Forest or Island does not count.
///
/// "{T}: Add {G} or {U}." (CR 605.1a, 106.5): a mana ability whose only cost is the tap, adding one of the two colours.
pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_unless_you_control_either::<{ sub(Subtype::Forest) }, { sub(Subtype::Island) }>),
    abilities: &[mana_ability(TAP, add_one_of(&[Color::Green, Color::Blue]), "{T}: Add {G} or {U}.")],
    ..oracle::HINTERLAND_HARBOR
};
