use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Put a +1/+1 counter on each creature you control": those its controller controls as the ability resolves.
fn counters(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::counters_on_creatures_you_control(g, item.controller, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(triggers::YOU_SCRY_OR_SURVEIL, counters, "Whenever you scry or surveil, put a +1/+1 counter on each creature you control.")],
    ..oracle::DENZILORE_FATEHOLD
};
