use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonland permanent. Put a +1/+1 counter on each creature target player controls."
/// Recheck the player’s target legality before putting counters on their current creatures.
fn destroy_and_grow(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    let Some(player) = targets::player(g, item, 1) else { return Ok(()) };
    cantrip_mechanics::effects::counters_on_creatures_you_control(g, player, CounterKind::PlusOne, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONLAND_PERMANENT, targets::PLAYER], destroy_and_grow),
    ..oracle::TWISTED_FATES
};
