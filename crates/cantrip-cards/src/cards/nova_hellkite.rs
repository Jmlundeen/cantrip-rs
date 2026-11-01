use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::damage_first_target::<1>, "When this creature enters, it deals 1 damage to target creature an opponent controls.")
        .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ways: &[warp::way(mana("{2}{R}"), "Warp {2}{R}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::NOVA_HELLKITE
};
