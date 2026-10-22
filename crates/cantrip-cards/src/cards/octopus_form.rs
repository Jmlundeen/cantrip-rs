use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target creature you control gets +1/+1 and gains hexproof until end of turn. Untap it." The target is checked as
/// the spell resolves (CR 608.2b); with it gone the spell does nothing.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 1, 1);
        cantrip_mechanics::effects::gain_hexproof_until_end_of_turn(g, item.object, creature);
    });
    g.end_sentence();
    g.untap(creature);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], effect),
    ..oracle::OCTOPUS_FORM
};
