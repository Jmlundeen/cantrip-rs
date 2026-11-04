use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// Target creature gets -3/-0 until end of turn (CR 611.2a). "Then if that creature's power is 0 or less, destroy it": the
/// power is read after the pump, as it is now (CR 608.2c); nothing happens at all if the target is no longer legal (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, -3, 0);
    if g.chars(creature).power.unwrap_or(0) <= 0 {
        g.destroy(creature);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::DEPRESSURIZE
};
