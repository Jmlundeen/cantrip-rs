//! Cards brought into the game from outside it (CR 400.11b): the cards a player owns outside the game, which in a match are
//! their sideboard (CR 100.4, 400.11a, 108.3b), kept by the core (`Query::outside_the_game`, `Game::bring_into_hand`).

use cantrip_api::*;

use crate::choose;

/// "You may put a card you own from outside the game into your hand" (CR 400.11b): `player` looks at the cards they own
/// outside the game - which only they may look at - and puts up to one of them into their hand, unrevealed. The card that
/// comes in, if one does. North Wind Avatar and Death Wish print it.
pub fn may_put_card_into_hand(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<Option<ObjectId>, Suspend> {
    let outside = g.outside_the_game(player);
    let chosen = choose::cards(g, player, source, Purpose::PutIntoHand, outside, 1, true)?;
    Ok(g.bring_into_hand(&chosen).first().copied())
}
