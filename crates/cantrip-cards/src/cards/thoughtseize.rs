use cantrip_api::*;
use cantrip_mechanics::{hand::discard_chosen_nonland_from_revealed_hand, targets};
use cantrip_oracle as oracle;

/// Target player reveals their hand; you choose a nonland card from it; they discard it; you lose 2 life. Asked even
/// when nothing in the hand is eligible, as Duress is; the life is lost whatever the hand held.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = item.targets[0].player() {
        discard_chosen_nonland_from_revealed_hand(g, item, player)?;
    }
    g.lose_life(item.controller, 2);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::THOUGHTSEIZE
};
