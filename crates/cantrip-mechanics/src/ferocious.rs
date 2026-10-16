//! Ferocious (CR 207.2c, an ability word with no rules meaning of its own): "if you control a creature with power 4 or
//! greater". Stubborn Denial, Fanatic of Rhonas and Bolt Bend print it, the last without the word.

use cantrip_api::*;

/// Whether `player` controls a creature with power 4 or greater, as the game is now.
pub fn you_control(q: &dyn Query, player: PlayerId) -> bool {
    q.permanents_of(player).into_iter().any(|id| {
        let c = q.chars(id);
        c.is(CardType::Creature) && c.power.is_some_and(|p| p >= 4)
    })
}
