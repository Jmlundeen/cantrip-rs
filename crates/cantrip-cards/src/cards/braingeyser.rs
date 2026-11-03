use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// Target player draws X cards (CR 121.1).
fn target_player_draws_x_cards(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    effects::target_player_draws(g, item, 0, u32::from(item.x));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], target_player_draws_x_cards),
    ..oracle::BRAINGEYSER
};
