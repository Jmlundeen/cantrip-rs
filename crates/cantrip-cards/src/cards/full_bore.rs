use cantrip_api::*;
use cantrip_mechanics::{pump, targets, warp};
use cantrip_oracle as oracle;

/// "Target creature you control gets +3/+2 until end of turn. If that creature was cast for its warp cost, it also gains
/// trample and haste until end of turn." Whether it was is read as the spell resolves, from the creature (CR 702.185a).
fn bore(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    pump(g, item.object, creature, 3, 2);
    // A sentence ends (CR 608.2c): the state triggers judge what it did before the next (`Game::end_sentence`).
    g.end_sentence();
    if warp::was_cast_for_warp(g, creature) {
        instruction(g, |g| {
            cantrip_mechanics::trample_until_end_of_turn(g, item.object, creature);
            cantrip_mechanics::haste_until_end_of_turn(g, item.object, creature);
        });
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL], bore),
    ..oracle::FULL_BORE
};
