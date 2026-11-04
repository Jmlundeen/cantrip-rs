use cantrip_api::*;
use cantrip_mechanics::{effects, statics, targets, triggers, warp};
use cantrip_oracle as oracle;

/// "When this creature enters, put a +1/+1 counter on target creature" and "Each creature you control with a +1/+1 counter on it
/// has trample" (CR 613.1f, 702.19), itself included once it has one.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        effects::plus_one_counter_on_target,
        "When this creature enters, put a +1/+1 counter on target creature.",
    )
    .targets(&[targets::CREATURE])],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_creatures_with_plus_one_counters,
        statics::have_trample,
        "Each creature you control with a +1/+1 counter on it has trample.",
    )],
    ways: &[warp::way(mana("{1}{G}"), "Warp {1}{G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::DRIX_FATEMAKER
};
