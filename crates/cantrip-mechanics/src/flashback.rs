//! Flashback [cost] (CR 702.34a): "You may cast this card from your graveyard by paying [cost] rather than its mana
//! cost. If you do, exile it as it leaves the stack instead of putting it anywhere else." Angelfire Ignition, Echo of
//! Eons, Eviscerator's Insight, Faithless Looting, Firebolt, Lingering Souls, Practiced Offense, Rite of Harmony, Rite of
//! Oblivion and Sevinne's Reclamation print it; Snapcaster Mage gives it with `grant_flashback`.
//!
//! A card with flashback lists `way(cost, text)` among its `ways`, marked `WayKind::Flashback`.

use cantrip_api::*;

/// The alternative cost of flashback: cast from the graveyard for `cost` (CR 118.9), exiled as it leaves the stack.
pub const fn way(cost: Cost, text: &'static str) -> WayDef {
    cantrip_api::way(cost, ZoneKind::Graveyard, text)
        .exile_after()
        .kind(WayKind::Flashback)
}

/// "A card with flashback" (CR 702.34a): the card has the keyword itself, read from what it is now, so a card exiled face down,
/// which has no characteristics (CR 406.3a), has none. Flashback another effect gives (Snapcaster Mage, Lier, Disciple of the
/// Drowned) is given only to a card in a graveyard and ends as the card leaves it (CR 400.7); the engine keeps it as a
/// permission to cast from there, not among the card's ways, so this reads only the card's own. Sorceress's Schemes, Runic
/// Repetition and Rootcoil Creeper ask it of an exiled card.
pub fn has_flashback(q: &dyn Query, card: ObjectId) -> bool {
    q.card(q.chars(card).card).ways.iter().any(|way| way.kind == WayKind::Flashback)
}
