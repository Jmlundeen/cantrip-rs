use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        // "Tap target creature an opponent controls" (CR 701.26a): nothing is tapped if the target is no longer legal as the
        // ability resolves (CR 608.2b).
        effects::tap_targets,
        "When this creature enters, tap target creature an opponent controls.",
    )
    .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS])],
    ..oracle::VEXING_DRONE
};
