use cantrip_api::*;
use cantrip_mechanics::effects::target_player_loses_1_life_you_gain_1;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::this_or_another_creature_or_artifact_you_control_dies),
        target_player_loses_1_life_you_gain_1,
        "Whenever this creature or another creature or artifact you control dies, target opponent loses 1 life and you gain 1 life.",
    )
    .targets(&[targets::OPPONENT])],
    ..oracle::AL_BHED_SALVAGERS
};
