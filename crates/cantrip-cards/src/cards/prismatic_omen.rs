use cantrip_api::*;
use cantrip_mechanics::statics::{every_basic_land_type, your_lands};
use cantrip_oracle as oracle;

/// "Lands you control are every basic land type in addition to their other types." (CR 305.7, 613.1d, layer 4)
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::Type,
        your_lands,
        every_basic_land_type,
        "Lands you control are every basic land type in addition to their other types.",
    )],
    ..oracle::PRISMATIC_OMEN
};
