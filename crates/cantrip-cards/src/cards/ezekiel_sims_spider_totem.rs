use cantrip_api::*;
use cantrip_mechanics::triggers::BEGINNING_OF_YOUR_COMBAT;
use cantrip_mechanics::{effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        BEGINNING_OF_YOUR_COMBAT,
        effects::target_creature_gets::<2, 2>,
        "At the beginning of combat on your turn, target Spider you control gets +2/+2 until end of turn.",
    )
    .targets(&[target(targets::permanent_of_subtype_you_control::<{ lands::sub(Subtype::Spider) }>)])],
    ..oracle::EZEKIEL_SIMS_SPIDER_TOTEM
};
