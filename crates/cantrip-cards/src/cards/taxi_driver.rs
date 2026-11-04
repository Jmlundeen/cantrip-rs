use cantrip_api::*;
use cantrip_mechanics::{haste_until_end_of_turn, targets};
use cantrip_oracle as oracle;

/// "Target creature gains haste until end of turn" (CR 611.2a, 702.10): the ability's first target, nothing if it is no longer a
/// legal target as the ability resolves (CR 608.2b).
fn haste(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    haste_until_end_of_turn(g, item.object, creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{1}, {T}"), haste, "{1}, {T}: Target creature gains haste until end of turn.").targets(&[targets::CREATURE])],
    ..oracle::TAXI_DRIVER
};
