use cantrip_api::*;
use cantrip_mechanics::statics::{have_haste, your_creatures};
use cantrip_oracle as oracle;

/// "Creatures you control have haste." (CR 702.10, a static ability granting a keyword in layer 6.)
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(Layer::Ability, your_creatures, have_haste, "Creatures you control have haste.")],
    ..oracle::SAMUT_HAZORETS_CHAMPION
};
