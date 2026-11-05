use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "When Will Riker enters, put a +1/+1 counter on another target creature you control." "Another" is the target
/// filter's (CR 109.5, 115.1): Will Riker himself is never a legal target, and a permanent that has left the
/// battlefield by the time the ability resolves is not a target any more (CR 608.2b).
fn counter_another(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        counter_another,
        "When Will Riker enters, put a +1/+1 counter on another target creature you control.",
    )
    .targets(&[targets::OTHER_CREATURE_YOU_CONTROL])],
    ..oracle::WILL_RIKER_ASSUMING_COMMAND
};
