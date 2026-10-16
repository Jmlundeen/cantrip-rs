use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, effects::ask_may, sacrifice_chosen, triggers::THIS_ATTACKS_OR_BLOCKS};
use cantrip_oracle as oracle;

const ASK: Question = Question::new(
    "DiscardOrSacrifice",
    "discard a card?",
    "If you don't, sacrifice a permanent",
    "Discard a card",
    "Sacrifice a permanent",
);

/// "Sacrifice a permanent unless you discard a card" (CR 608.2d): the controller chooses to discard a card of their hand,
/// which they can only do with a card in it, or else sacrifices a permanent of their choice (any permanent they control,
/// this creature included).
fn sacrifice_unless_discard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let you = item.controller;
    let has_a_card = !g.state().zone(Zone::hand(you)).is_empty();
    if has_a_card && ask_may(g, you, item.object, Purpose::Ask(&ASK))? {
        discard_chosen(g, you, item.object, 1)?;
    } else {
        sacrifice_chosen(g, you, item.object, 1, |_| true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ATTACKS_OR_BLOCKS,
        sacrifice_unless_discard,
        "Whenever Bebop & Rocksteady attack or block, sacrifice a permanent unless you discard a card.",
    )],
    ..oracle::BEBOP_ROCKSTEADY
};
