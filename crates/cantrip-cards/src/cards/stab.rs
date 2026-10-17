use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets -2/-2 until end of turn." (CR 611.2a): a continuous effect from the layer 7c, on the creature if
/// it is still a legal target (CR 608.2b), the same creature (CR 400.7). A creature left with toughness 0 or less dies as a
/// state-based action (CR 704.5f).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        pump(g, item.object, creature, -2, -2);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::STAB };
