//! Ferocious (CR 207.2c, an ability word with no rules meaning of its own): "if you control a creature with power 4 or
//! greater". Stubborn Denial, Fanatic of Rhonas and Bolt Bend print it, the last without the word; Temur Battlecrier and
//! Dragonhawk, Fate's Tempest count such creatures.

use cantrip_api::*;

/// How many creatures `player` controls with power 4 or greater, as the game is now.
pub fn count(q: &dyn Query, player: PlayerId) -> usize {
    q.permanents_of(player)
        .into_iter()
        .filter(|&id| {
            let c = q.chars(id);
            c.is(CardType::Creature) && c.power.is_some_and(|p| p >= 4)
        })
        .count()
}

/// Whether `player` controls a creature with power 4 or greater, as the game is now.
pub fn you_control(q: &dyn Query, player: PlayerId) -> bool {
    count(q, player) > 0
}
