//! Start your engines! (CR 702.179): "If you have no speed, it starts at 1. It increases once on each of your turns when
//! an opponent loses life. Max speed is 4." The keyword is a card's `start_your_engines: true`, which the rules read to
//! give its controller a speed of 1 (`Player::speed`, CR 704.5aa); the speed's own triggered ability is the rules'
//! (CR 702.179d). What is here is a player's speed as effects read it. Perilous Snare prints the keyword.

use cantrip_api::*;

/// The highest speed (CR 702.179e).
pub use cantrip_api::intrinsic::MAX_SPEED;

/// `player`'s speed as an effect reads it: 0 for a player who has none (CR 702.179f).
pub fn speed(s: &GameState, player: PlayerId) -> u8 {
    s.player(player).speed.unwrap_or(0)
}

/// Whether `player` has max speed (CR 702.179e): their speed is 4.
pub fn has_max_speed(s: &GameState, player: PlayerId) -> bool {
    speed(s, player) == MAX_SPEED
}
