use cantrip_api::*;
use cantrip_mechanics::effects::{enters_with_x_counters, put_its_counters_on_target};
use cantrip_mechanics::triggers::THIS_LEAVES;
use cantrip_mechanics::{targets, warp};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_x_counters::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(
        THIS_LEAVES,
        put_its_counters_on_target,
        "When this creature leaves the battlefield, put its counters on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ways: &[warp::way(mana("{X}{G}"), "Warp {X}{G}")],
    end_step_riders: warp::RIDERS,
    delayed: &[warp::EXILE],
    ..oracle::BROODGUARD_ELITE
};
