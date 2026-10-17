//! Spectacle [cost] (CR 702.137a): "You may cast this spell by paying [cost] rather than its mana cost if an opponent
//! lost life this turn." Drill Bit prints it.
//!
//! A card with spectacle lists `way(cost, text)` among its `ways`.

use cantrip_api::*;

/// "Lost life", not a lower life total (CR 119.3): asked of the event history.
pub fn opponent_lost_life(q: &dyn Query, asker: &Asker) -> bool {
    crate::history::an_opponent_lost_life(q, asker)
}

/// The alternative cost of spectacle: cast from the hand for `cost` (CR 118.9), if an opponent lost life this turn.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost::mana(cost), ZoneKind::Hand, text)
        .condition(opponent_lost_life)
}
