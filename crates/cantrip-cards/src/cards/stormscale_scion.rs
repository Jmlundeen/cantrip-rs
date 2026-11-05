use cantrip_api::*;
use cantrip_mechanics::statics::{other_of_subtype_you_control, plus_pt};
use cantrip_mechanics::{lands, storm};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    statics: &[static_ability(
        Layer::PtModify,
        other_of_subtype_you_control::<{ lands::sub(Subtype::Dragon) }>,
        plus_pt::<1, 1>,
        "Other Dragons you control get +1/+1.",
    )],
    abilities: &[storm::trigger("Storm (When you cast this spell, copy it for each spell cast before it this turn. Copies become tokens.)")],
    ..oracle::STORMSCALE_SCION
};
