use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Return target nonland permanent to its owner's hand; if you controlled that permanent, draw a card. Control is
/// read as the spell resolves, before the permanent leaves.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let yours = g.controller(permanent) == item.controller;
    g.return_to_hand(&[permanent]);
    if yours {
        g.draw(item.controller, 1);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[cantrip_mechanics::targets::NONLAND_PERMANENT], effect),
    ..oracle::BOOMERANG_BASICS
};
