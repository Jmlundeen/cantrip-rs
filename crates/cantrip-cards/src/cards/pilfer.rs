use cantrip_api::*;
use cantrip_mechanics::{hand::discard_chosen_nonland_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// "Target opponent reveals their hand. You choose a nonland card from it. That player discards that card." (CR 701.9a): any
/// nonland card, a creature too. The hand is revealed; when no nonland card is in it there is no choice to make, so nothing is asked (CR 608.2d), and
/// nothing is revealed or asked of an empty hand. With its target illegal the spell does not resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[targets::OPPONENT], effect), ..oracle::PILFER };
