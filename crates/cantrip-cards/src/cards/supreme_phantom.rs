use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_oracle as oracle;

/// "Other Spirits you control get +1/+1." (CR 613.4c, layer 7c): every other permanent its controller controls that is a
/// Spirit as the layers have it so far, the type read through the words of this card's text (CR 612).
pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_of_subtype_you_control::<{ lands::sub(Subtype::Spirit) }>,
        plus_pt::<1, 1>,
        "Other Spirits you control get +1/+1.",
    )],
    ..oracle::SUPREME_PHANTOM
};
