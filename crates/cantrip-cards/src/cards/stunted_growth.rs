use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target player chooses three cards from their hand and puts them on top of their library in any order." (CR 608.2c, 401.4).
fn put_three_on_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    effects::put_cards_from_hand_on_top(g, player, item.object, 3)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], put_three_on_top),
    ..oracle::STUNTED_GROWTH
};
