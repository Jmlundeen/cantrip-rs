use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Each of up to two target creatures" (CR 601.2c): one slot, up to two targets; each still legal gets a counter.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    for creature in targets::legal_objects(g, item) {
        g.add_counters(creature, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::BEGINNING_OF_YOUR_COMBAT, counters, "At the beginning of combat on your turn, put a +1/+1 counter on each of up to two target creatures.")
        .targets(&[TargetSpec { min: 0, max: 2, ..targets::CREATURE }])],
    ..oracle::AGENT_BISHOP_MAN_IN_BLACK
};
