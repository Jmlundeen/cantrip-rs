use cantrip_api::*;
use cantrip_mechanics::{spectacle, hand};
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target player reveals their hand. You choose a nonland card from it. That player discards that card." Asked
/// whenever the hand is not empty, even when nothing in it is a nonland card: looking at the hand is part of the spell.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    hand::discard_chosen_nonland_from_revealed_hand(g, item, player)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ways: &[spectacle::way(mana("{B}"), "Spectacle {B}")],
    ..oracle::DRILL_BIT
};
