use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Counter target spell; it goes on top of its owner's library instead of into the graveyard.
fn lapse(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    g.counter_to(spell, CounterTo::LibraryTop);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::SPELL], effect: lapse }),
    ..oracle::MEMORY_LAPSE
};
