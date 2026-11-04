use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::return_first_target_to_hand,
        "When Spider-Byte enters, return up to one target nonland permanent to its owner's hand.",
    )
    .targets(&[targets::NONLAND_PERMANENT.between(0, 1)])],
    ..oracle::SPIDER_BYTE_WEB_WARDEN
};
