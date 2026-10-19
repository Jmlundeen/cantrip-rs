use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonland permanent. Put a +1/+1 counter on each creature target player controls."
/// Recheck the player’s target legality before putting counters on their current creatures.
fn destroy_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    let Some(player) = targets::player(g, item, 1) else { return Ok(()) };
    for id in g.permanents_of(player) {
        if g.chars(id).is(CardType::Creature) {
            g.add_counters(id, CounterKind::PlusOne, 1);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT, targets::PLAYER], destroy_and_grow),
    ..oracle::TWISTED_FATES
};
