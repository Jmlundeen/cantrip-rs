use cantrip_api::*;
use cantrip_mechanics::{protection, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    statics: &[protection::enchanted_from_color_not_removing_this_aura::<{ Color::White as u8 }>(
        "Enchanted creature has protection from white. This effect doesn't remove this Aura.",
    )],
    ..oracle::WHITE_WARD
};
