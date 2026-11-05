use cantrip_api::*;
use cantrip_mechanics::{hand::exile_chosen_nonland_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You choose a nonland card from it and exile that card. Put a +1/+1 counter on up to one
/// target creature you control." Each target is judged on its own as this resolves (CR 608.2b): an opponent who is no longer a
/// legal target (gone, or with hexproof now) shows and loses nothing, and the counter still goes on a creature that is still
/// legal, and the other way round.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(opponent) = targets::player(g, item, 0) {
        exile_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    }
    let Some(creature) = targets::object(g, item, 1) else { return Ok(()) };
    g.add_counters(creature, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT, targets::CREATURE_YOU_CONTROL.between(0, 1)], effect),
    ..oracle::AGGRESSIVE_NEGOTIATIONS
};
