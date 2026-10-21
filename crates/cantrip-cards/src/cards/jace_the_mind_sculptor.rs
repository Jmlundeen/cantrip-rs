use cantrip_api::*;
use cantrip_mechanics::choose;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "+2: Look at the top card of target player's library. You may put that card on the bottom of that player's library."
/// The one looking is Jace's controller, asked whether to; an empty library shows nothing and asks nothing.
fn look_at_top(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.target(0).player() else { return Ok(()) };
    let Some(&top) = g.state().zone(Zone::library(player)).first() else { return Ok(()) };
    if cantrip_mechanics::effects::ask_may(g, item.controller, top, Purpose::PutOnBottom)? {
        g.arrange_library(player, &[], &[top]);
    }
    Ok(())
}

/// "0: Draw three cards, then put two cards from your hand on top of your library in any order."
fn brainstorm(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 3);
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    let cards = choose::cards(g, player, item.object, Purpose::PutOnTop, hand, 2, false)?;
    // The last card chosen ends up on top.
    let order: Vec<ObjectId> = cards.iter().rev().copied().collect();
    g.put_into_library(&order, LibraryPlace::Top);
    Ok(())
}

/// "\u{2212}1: Return target creature to its owner's hand."
fn bounce(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0)
        && g.is_on_battlefield(creature)
    {
        g.return_to_hand(&[creature]);
    }
    Ok(())
}

/// "\u{2212}12: Exile all cards from target player's library, then that player shuffles their hand into their library."
fn exile_library(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = item.target(0).player() else { return Ok(()) };
    let library = g.state().zone(Zone::library(player)).to_vec();
    g.exile_all(&library);
    let hand = g.state().zone(Zone::hand(player)).to_vec();
    g.put_into_library(&hand, LibraryPlace::Top);
    g.shuffle(player);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(loyalty(2), look_at_top, "+2: Look at the top card of target player's library. You may put that card on the bottom of that player's library.")
            .targets(&[targets::PLAYER]),
        ability(loyalty(0), brainstorm, "0: Draw three cards, then put two cards from your hand on top of your library in any order."),
        ability(loyalty(-1), bounce, "\u{2212}1: Return target creature to its owner's hand.")
            .targets(&[targets::CREATURE]),
        ability(loyalty(-12), exile_library, "\u{2212}12: Exile all cards from target player's library, then that player shuffles their hand into their library.")
            .targets(&[targets::PLAYER]),
    ],
    ..oracle::JACE_THE_MIND_SCULPTOR
};
