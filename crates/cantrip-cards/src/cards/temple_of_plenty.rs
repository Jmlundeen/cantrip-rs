use cantrip_api::*;
use cantrip_mechanics::{lands, mana};
use cantrip_oracle as oracle;

/// "This land enters tapped" and "When this land enters, scry 1" are the shared land paths (mana::enters_tapped for the
/// first, lands::SCRY_LAND for the second, CR 614.1d and 701.22): the card adds only its own {T} ability and its colors.
pub static CARD: CardDef = CardDef {
    enters: Some(mana::enters_tapped),
    abilities: &[lands::SCRY_LAND, mana_ability(TAP, add_one_of(&[Color::Green, Color::White]), "{T}: Add {G} or {W}.")],
    ..oracle::TEMPLE_OF_PLENTY
};
