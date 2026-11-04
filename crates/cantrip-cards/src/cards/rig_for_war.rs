use cantrip_api::*;
use cantrip_mechanics::{effects, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets +3/+0 and gains reach and first strike until end of turn" (CR 611.2a, 702.17, 702.7): one instruction, the
/// creature being the spell's first target; nothing if it is no longer a legal target as the spell resolves (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 3, 0);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::Reach);
        effects::gain_until_end_of_turn(g, item.object, creature, Keyword::FirstStrike);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effect), ..oracle::RIG_FOR_WAR };
