use cantrip_api::*;
use cantrip_mechanics::effects::target_player_loses_1_life_you_gain_1;
use cantrip_mechanics::{targets, triggers, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::this_or_another_creature_or_artifact_you_control_dies),
        target_player_loses_1_life_you_gain_1,
        "Whenever this creature or another creature or artifact you control dies, target opponent loses 1 life and you gain 1 life.",
    )
    .targets(&[targets::OPPONENT])],
    ways: &[warp::way(mana("{B}"), "Warp {B}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::SUSURIAN_VOIDBORN
};
