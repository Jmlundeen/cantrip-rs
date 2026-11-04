use cantrip_api::*;
use cantrip_mechanics::effects::{enters_with_a_counter, put_its_counters_on_target};
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::THIS_DIES;
use cantrip_oracle as oracle;

/// Dockworker Drone: "This creature enters with a +1/+1 counter on it" (CR 122.1, 614.1c); "When this creature dies, put its
/// counters on target creature you control" puts the same number of each kind it had as it died, not moved (CR 603.10a). With no
/// other creature of yours it has no target and is removed from the stack (CR 603.3d).
pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_a_counter::<{ CounterKind::PlusOne as usize }>),
    abilities: &[triggered(
        THIS_DIES,
        put_its_counters_on_target,
        "When this creature dies, put its counters on target creature you control.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL])],
    ..oracle::DOCKWORKER_DRONE
};
