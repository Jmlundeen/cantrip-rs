use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Whenever Picard enters or attacks, put two +1/+1 counters on another target creature you control." The counters are
/// put together (CR 122.1a); "another" is the target filter's (CR 109.5) and a target that has left the battlefield is
/// not affected (CR 608.2b).
fn two_counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS_OR_ATTACKS,
        two_counters,
        "Whenever Picard enters or attacks, put two +1/+1 counters on another target creature you control.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::PICARD_STEADFAST_CAPTAIN
};
