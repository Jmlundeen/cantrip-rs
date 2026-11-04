use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS, warp};
use cantrip_oracle as oracle;

/// "Each of up to two target creatures" (CR 601.2c): one slot, up to two targets; each still legal gets a counter.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::plus_one_counter_on_each_target,
        "When this creature enters, put a +1/+1 counter on each of up to two target creatures.",
    )
    .targets(&[targets::CREATURE.between(0, 2)])],
    ways: &[warp::way(mana("{2}{W}"), "Warp {2}{W}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::WEFTBLADE_ENHANCER
};
