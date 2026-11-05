use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+0 until end of turn. Draw a card." (CR 611.2a, 121.1): the pump, then in a second sentence
/// (CR 608.2c) the draw. If the creature is no longer a legal target the spell does not resolve (CR 608.2b), so nothing, the draw
/// included, happens.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 0);
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::REBELLIOUS_STRIKE };
