use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Look at target player's hand; draw a card. The look changes no game state, so it is not modelled; the draw happens
/// even if the player is no longer a legal target to look at.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::PLAYER], effect }),
    ..oracle::GITAXIAN_PROBE
};
