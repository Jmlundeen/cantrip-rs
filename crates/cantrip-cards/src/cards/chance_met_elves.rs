use cantrip_api::*;
use cantrip_mechanics::{targets, triggers};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on this creature", if it is still the permanent that triggered.
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if targets::permanent(g, item.object) {
        g.add_counters(item.object, CounterKind::PlusOne, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOU_SCRY, grow, "Whenever you scry, put a +1/+1 counter on this creature. This ability triggers only once each turn.")
        .triggers_each_turn(1)],
    ..oracle::CHANCE_MET_ELVES
};
