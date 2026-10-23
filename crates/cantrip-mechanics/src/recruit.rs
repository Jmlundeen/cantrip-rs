//! Recruit (CR 701.70a): "Draw a card, then discard a card. If you discarded a nonland card this way, create a 1/1 white
//! Human Soldier creature token." The Queen of Dale prints it.
//!
//! The player draws (an empty library draws nothing, CR 121.4, and loses at the next state check, CR 704.5b), then
//! discards a card they choose (CR 701.9); with an empty hand there is nothing to discard and so no token. The token is made
//! only for a card that was actually discarded: one its owner can't be made to discard stays (CR 101.2), and a card
//! discarded is judged by what it is, a land card or not.

use crate::{choose_from_hand, tokens};
use cantrip_api::*;

/// `player` recruits, on behalf of `source` (the object whose ability does it).
pub fn recruit(g: &mut dyn Game, player: PlayerId, source: ObjectId) -> Result<(), Suspend> {
    g.draw(player, 1);
    let chosen = choose_from_hand(g, player, source, 1)?;
    let discarded = g.discard_together(&chosen);
    if discarded.iter().any(|&card| !g.chars(card).is(CardType::Land)) {
        tokens::create_by_rule(g, player, &tokens::HUMAN_SOLDIER, 1, false)?;
    }
    Ok(())
}
