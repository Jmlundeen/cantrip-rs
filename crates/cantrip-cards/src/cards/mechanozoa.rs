use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS, warp};
use cantrip_oracle as oracle;

/// "Tap target artifact or creature an opponent controls and put a stun counter on it" (CR 701.26a, 122.1d) is the shared tap-and-stun
/// step; Warp {2}{U} is the shared mechanic (CR 702.185).
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::tap_and_stun::<1>,
        "When this creature enters, tap target artifact or creature an opponent controls and put a stun counter on it.",
    )
    .targets(&[targets::ARTIFACT_OR_CREATURE_AN_OPPONENT_CONTROLS])],
    ways: &[warp::way(mana("{2}{U}"), "Warp {2}{U}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::MECHANOZOA
};
