use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, your_creatures};
use cantrip_oracle as oracle;

/// "Creatures you control have haste." (CR 702.10, a static ability granting a keyword in layer 6.)
pub static CARD: CardDef = CardDef {
    statics: &[StaticDef {
        text: "Creatures you control have haste.",
        layer: Layer::Ability,
        affects: your_creatures,
        apply: have_haste,
        ..StaticDef::BLANK
    }],
    ..oracle::SAMUT_HAZORETS_CHAMPION
};
