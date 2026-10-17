use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+3 until end of turn." A continuous effect from a resolved spell (CR 611.2) that modifies power
/// and toughness, so it applies in layer 7c (CR 613.4c), after every effect that sets them in 7b whatever the timestamps
/// (Turn to Frog's ruling): it ends in the cleanup step (CR 514.2). A target gone or no longer a creature by resolution is
/// illegal, and the spell does nothing (CR 608.2b).
fn grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, 3, 3);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], grow), ..oracle::GIANT_GROWTH };
