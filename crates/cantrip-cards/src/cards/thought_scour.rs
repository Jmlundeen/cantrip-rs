use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::targets;

/// Target player mills two cards, then you draw a card: in that order, even when the target is you (CR 608.2c).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if g.target_legal(item, 0)
        && let Some(player) = item.target(0).player()
    {
        g.mill(player, 2);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::THOUGHT_SCOUR
};
