use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_oracle as oracle;

/// Draw three cards, then put two cards from your hand on top of your library in any order: the cards are chosen
/// together and put back one at a time in the order chosen, so the last chosen ends on top.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 3);
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let cards = choose::cards(g, player, item.object, Purpose::PutOnTop, hand, 2, false)?;
    // The last card chosen ends up on top.
    let order: Vec<ObjectId> = cards.iter().rev().copied().collect();
    g.put_into_library(&order, LibraryPlace::Top);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: Some(SpellDef { targets: &[], effect }), ..oracle::BRAINSTORM };
