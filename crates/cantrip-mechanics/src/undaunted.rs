//! Undaunted (CR 702.125a): "This spell costs {1} less to cast for each opponent you have." A static ability that
//! functions while the spell is on the stack. Võx, Child of the Collective grants it to a spell as it is cast.
//!
//! Players who have left the game are not counted (CR 702.125b): `opponents` is the players still in the game. Each
//! instance of undaunted applies (CR 702.125c): a spell with two reduces twice, as two cost changes each ask this once.

use cantrip_api::*;

/// "Undaunted" (CR 702.125a, 702.125b, 601.2f, 800.4a): {1} less to cast for each opponent `player` has still in the game.
/// A cost change; one instance, so each further instance on the same spell is another change using this (CR 702.125c).
pub fn undaunted(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(q.state().opponents(player).count() as u32)
}
