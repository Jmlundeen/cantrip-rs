use cantrip_api::*;
use cantrip_mechanics::hand::discard_chosen_nonland_from_revealed_hand;
use cantrip_mechanics::{discard_chosen, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Choose target opponent. If they lost life this turn, they reveal their hand, you choose a nonland card from it, and
/// they discard that card. Otherwise, they discard a card." Lost life is asked of the history as the ability resolves
/// (CR 119.3); the opponent chooses their own discard in the second case (CR 701.9b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = item.targets[0].player() else { return Ok(()) };
    if g.history().life_lost(opponent) > 0 {
        discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    } else {
        discard_chosen(g, opponent, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effect, "When this creature enters, choose target opponent. If they lost life this turn, they reveal their hand, you choose a nonland card from it, and they discard that card. Otherwise, they discard a card.")
        .targets(&[targets::OPPONENT])],
    ..oracle::THOUGHT_STALKER_WARLOCK
};
