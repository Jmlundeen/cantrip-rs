use cantrip_api::*;
use cantrip_mechanics::triggers::YOUR_UPKEEP;
use cantrip_mechanics::{discard_chosen, mayhem, tokens};
use cantrip_oracle as oracle;

/// "Discard a card, then create a Treasure token" (CR 701.9, 111.10a): the discard is not optional, and nothing is asked of an
/// empty hand; the Treasure comes whether or not a card was discarded.
fn discard_then_treasure(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    discard_chosen(g, item.controller, item.object, 1)?;
    tokens::create_treasure(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(YOUR_UPKEEP, discard_then_treasure, "At the beginning of your upkeep, discard a card, then create a Treasure token.")],
    ways: &[mayhem::way(mana("{2}{B/R}"), "Mayhem {2}{B/R}")],
    ..oracle::ULTIMATE_GREEN_GOBLIN
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
