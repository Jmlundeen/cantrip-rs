use cantrip_api::*;
use cantrip_mechanics::statics::{grant_delayed_ability, your_artifact_creatures};
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Artifact creatures you control have "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life."" (CR 613.1f).
pub static CARD: CardDef = CardDef {
    statics: &[
        static_ability(
            Layer::Ability,
            your_artifact_creatures,
            grant_delayed_ability,
            "Artifact creatures you control have \"Whenever this creature attacks, each opponent loses 1 life and you gain 1 life.\"",
        ),
    ],
    delayed: &[
        triggered(
            triggers::THIS_ATTACKS,
            effects::each_opponent_loses_1_life_you_gain_1,
            "Whenever this creature attacks, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    ..oracle::BIO_ASSET_ALLOCATOR
};
