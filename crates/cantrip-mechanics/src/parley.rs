//! Parley (CR 701.20 reveal, 121.1 draw): "each player reveals the top card of their library, then draws a card." The
//! reveal comes first and is seen by every player; the draws follow as one instruction, in seat order, and a player whose
//! library is empty draws from it as usual (CR 121.3). Ambassador K'Ehleyr prints it.
//!
//! A card with parley calls `parley` and does whatever its text says with the revealed cards.

use crate::top_of_library;
use cantrip_api::*;

/// Parley for the controller of the spell or ability: every player reveals the top card of their library, then each draws
/// a card. Returns the cards revealed, in seat order starting with the controller, so the card can read them (K'Ehleyr
/// counts the nonland cards among them; each is still the same object once drawn, CR 400.7).
pub fn parley(g: &mut dyn Game, controller: PlayerId) -> Vec<ObjectId> {
    let mut players: Vec<PlayerId> = vec![controller];
    players.extend(g.state().seats().filter(|&p| p != controller));
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
