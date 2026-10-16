//! Flashback [cost] (CR 702.34a): "You may cast this card from your graveyard by paying [cost] rather than its mana
//! cost. If you do, exile it as it leaves the stack instead of putting it anywhere else." Angelfire Ignition, Echo of
//! Eons, Eviscerator's Insight, Faithless Looting, Firebolt, Lingering Souls, Practiced Offense, Rite of Harmony, Rite of
//! Oblivion and Sevinne's Reclamation print it; Snapcaster Mage gives it with `grant_flashback`.
//!
//! A card with flashback lists `way(cost, text)` among its `ways`.

use cantrip_api::*;

/// The alternative cost of flashback: cast from the graveyard for `cost` (CR 118.9), exiled as it leaves the stack.
pub const fn way(cost: Cost, text: &'static str) -> WayDef {
    cantrip_api::way(cost, ZoneKind::Graveyard, text)
        .exile_after()
}
