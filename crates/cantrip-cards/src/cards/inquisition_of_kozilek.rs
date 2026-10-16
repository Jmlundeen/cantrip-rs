use cantrip_api::*;
use cantrip_mechanics::{hand::discard_chosen_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// Target player reveals their hand; you choose a nonland card with mana value 3 or less (CR 202.3) from it; they discard it.
/// Asked even when nothing in the hand is eligible, as Duress is: looking at the hand is part of the spell.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.targets[0].player() else { return Ok(()) };
    discard_chosen_from_revealed_hand(g, item, player, Purpose::TakeFromHand, |q, c| !q.chars(c).is(CardType::Land) && q.mana_value(c) <= 3)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::INQUISITION_OF_KOZILEK
};
