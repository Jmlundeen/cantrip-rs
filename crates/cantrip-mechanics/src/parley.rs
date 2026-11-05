//! Parley (CR 701.20 reveal, 121.2c draw): "each player reveals the top card of their library, then draws a card." The
//! reveal comes first and is seen by every player; the draws follow as one instruction, the active player's first and
//! then each other player's in turn order (CR 121.2c), every player still in the game and none who has left (CR 800.4a).
//! A player whose library is empty attempts the draw and loses the game the next time state-based actions are checked
//! (CR 121.4, 704.5b). Ambassador K'Ehleyr prints it.
//!
//! A card with parley calls `parley` and does whatever its text says with the revealed cards.

use crate::top_of_library;
use cantrip_api::*;

/// Parley: every player in the game reveals the top card of their library, then each draws a card, in APNAP order.
/// Returns the cards revealed, in that order, so the card can read them (K'Ehleyr counts the nonland cards among them;
/// each is still the same object once drawn, CR 400.7).
pub fn parley(g: &mut dyn Game) -> Vec<ObjectId> {
    let players: Vec<PlayerId> = g.state().apnap().collect();
    let mut revealed = Vec::new();
    instruction(g, |g| {
        for &p in &players {
            if let Some(&card) = top_of_library(g, p, 1).first() {
                g.reveal(p, &[card]);
                revealed.push(card);
            }
        }
        for &p in &players {
            g.draw(p, 1);
        }
    });
    revealed
}
