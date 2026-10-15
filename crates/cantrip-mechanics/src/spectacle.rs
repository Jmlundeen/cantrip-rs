//! Spectacle [cost] (CR 702.137a): "You may cast this spell by paying [cost] rather than its mana cost if an opponent
//! lost life this turn." Drill Bit prints it.
//!
//! A card with spectacle lists `way(cost, text)` among its `ways`.

use crate::history::opponents_who_lost_life_this_turn;
use cantrip_api::*;

/// "Lost life", not a lower life total (CR 119.3): asked of the event history.
pub fn opponent_lost_life(q: &dyn Query, asker: &Asker) -> bool {
    opponents_who_lost_life_this_turn(q, asker.controller) > 0
}

/// The alternative cost of spectacle: cast from the hand for `cost` (CR 118.9), if an opponent lost life this turn.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef { text, cost: Cost::mana(cost), from: ZoneKind::Hand, exile_after: false, condition: Some(opponent_lost_life), instant_speed: false }
}
