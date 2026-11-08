use cantrip_api::*;
use cantrip_mechanics::{pump, targets};
use cantrip_oracle as oracle;

/// "Target attacking creature gets +3/+3 and gains trample until end of turn." (CR 506.4: only an attacking creature is
/// a legal target; CR 608.2b, 611.2a): the target gets +3/+3 and trample with the spell's duration, all in one event.
fn amok(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        pump(g, item.object, creature, 3, 3);
        cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ATTACKING_CREATURE], amok),
    ..oracle::RUN_AMOK
};
