//! Cleave [cost] (CR 702.148a): "You may cast this spell by paying [cost] rather than its mana cost. If you do, remove
//! the words in square brackets from this spell's text." Winged Portent prints it.
//!
//! A card with cleave lists `way(cost, text)` as its only alternative cost; its effect asks `cleaved` of the resolving
//! spell to decide whether the bracketed words are there.

use cantrip_api::*;

/// The alternative cost of cleave: cast from the hand for `cost` (CR 118.9). It is the card's first way.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    WayDef { text, cost: Cost::mana(cost), from: ZoneKind::Hand, exile_after: false, condition: None, instant_speed: false }
}

/// Whether the spell was cast for its cleave cost, so that the bracketed words are removed: judged as it resolves.
pub fn cleaved(g: &dyn Game, spell: ObjectId) -> bool {
    g.obj(spell).cast_way == 1
}
