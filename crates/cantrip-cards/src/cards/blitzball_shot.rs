use cantrip_api::*;
use cantrip_mechanics::{pump, targets, trample_until_end_of_turn};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+3 and gains trample until end of turn." Both changes are one instruction (CR 608.2c).
fn pump_and_trample(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 3, 3);
        trample_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], pump_and_trample), ..oracle::BLITZBALL_SHOT };
