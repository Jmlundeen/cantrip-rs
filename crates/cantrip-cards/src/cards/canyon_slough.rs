use cantrip_api::*;
use cantrip_mechanics::{mana, triggers};
use cantrip_oracle as oracle;

/// "This land enters tapped." (CR 614.1d) and "Cycling {2}" (CR 702.29a): the shared tapped entry and cycling trigger.
pub static CARD: CardDef = CardDef {
    abilities: &[triggers::cycling(mana("{2}"), "Cycling {2}")],
    enters: Some(mana::enters_tapped),
    ..oracle::CANYON_SLOUGH
};
