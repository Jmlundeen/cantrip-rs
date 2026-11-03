//! Coven (CR 207.2c, an ability word with no rules meaning of its own): "if you control three or more creatures with
//! different powers". Ambitious Farmhand prints it with the word, Collector's Cage without.

use cantrip_api::*;

/// How many different powers there are among the creatures `player` controls, as the game is now.
pub fn different_powers(q: &dyn Query, player: PlayerId) -> usize {
    let mut powers: Vec<i16> = Vec::new();
    for id in q.permanents_of(player) {
        let c = q.chars(id);
        if c.is(CardType::Creature)
            && let Some(p) = c.power
            && !powers.contains(&p)
        {
            powers.push(p);
        }
    }
    powers.len()
}

/// Whether `player` controls three or more creatures with different powers, as the game is now.
pub fn you_control(q: &dyn Query, player: PlayerId) -> bool {
    different_powers(q, player) >= 3
}

/// `you_control` as a condition of the ability's controller, asked of an `Asker`: an activation restriction (Ambitious
/// Farmhand's "Activate only if you control three or more creatures with different powers").
pub fn you_control_asked(q: &dyn Query, asker: &Asker) -> bool {
    you_control(q, asker.controller)
}
