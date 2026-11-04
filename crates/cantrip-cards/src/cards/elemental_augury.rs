use cantrip_api::*;
use cantrip_mechanics::{library, targets, top_of_library};
use cantrip_oracle as oracle;

/// Look at and reorder the targeted player's top three cards (CR 608.2b, 608.2c).
fn look_and_reorder(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let cards = top_of_library(g, player, 3);
    g.look_at(item.controller, &cards);
    let order = library::ask_library_order(g, item.controller, item.object, &cards, DestinationKind::LibraryTop)?;
    g.arrange_library(player, &order, &[]);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}"),
        look_and_reorder,
        "{3}: Look at the top three cards of target player's library, then put them back in any order.",
    ).targets(&[targets::PLAYER])],
    ..oracle::ELEMENTAL_AUGURY
};
