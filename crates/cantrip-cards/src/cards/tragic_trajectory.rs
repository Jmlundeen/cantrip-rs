use cantrip_api::*;
use cantrip_mechanics::{history, pump, targets};
use cantrip_oracle as oracle;

/// "Target creature gets -2/-2 until end of turn. Void - That creature gets -10/-10 until end of turn instead if a nonland
/// permanent left the battlefield this turn or a spell was warped this turn." The void condition is asked as the spell
/// resolves, so a permanent that left (or a spell warped) after it was cast, in response, counts.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let n = if history::void(g) { 10 } else { 2 };
    pump(g, item.object, creature, -n, -n);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effect),
    ..oracle::TRAGIC_TRAJECTORY
};
