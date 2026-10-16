use cantrip_api::*;
use cantrip_mechanics::{same_name, targets};
use cantrip_oracle as oracle;

/// Return the target and all other cards with the same name as it from your graveyard to your hand, together. The
/// target first, then the graveyard in its order.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(target) = targets::object(g, item, 0) else { return Ok(()) };
    let player = item.controller;
    let mut returning = vec![target];
    for &card in g.state().zone(Zone::graveyard(player)) {
        if card != target && same_name(g, target, card) {
            returning.push(card);
        }
    }
    g.return_to_hand(&returning);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_CARD_IN_YOUR_GRAVEYARD], effect),
    ..oracle::ECHOING_RETURN
};
