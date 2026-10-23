use cantrip_api::*;
use cantrip_mechanics::statics::{have_flying, itself_with_threshold, plus_pt};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(cantrip_mechanics::prepare::enters_prepared),
    prepare: Some(&super::paradox_shaper::OMIT_VARIABLES),
    statics: &[
        static_ability(
            Layer::PtModify,
            itself_with_threshold,
            plus_pt::<1, 0>,
            "Threshold — This creature gets +1/+0 as long as there are seven or more cards in your graveyard.",
        ),
        static_ability(
            Layer::Ability,
            itself_with_threshold,
            have_flying,
            "Threshold — This creature has flying as long as there are seven or more cards in your graveyard.",
        ),
    ],
    ..oracle::THEORIX_METAMAGE
};
