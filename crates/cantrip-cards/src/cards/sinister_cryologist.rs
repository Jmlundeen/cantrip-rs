use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::target_creature_gets::<-3, 0>,
        "When this creature enters, target creature an opponent controls gets -3/-0 until end of turn.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ways: &[warp::way(mana("{U}"), "Warp {U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::SINISTER_CRYOLOGIST
};
