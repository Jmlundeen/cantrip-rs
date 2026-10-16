//! Web-slinging [cost] (CR 702.188a): "You may cast this spell by paying [cost] and returning a tapped creature you
//! control to its owner's hand rather than paying its mana cost." Spider-Sense, Spider-Man, Web-Slinger, Amazing
//! Spider-Girl, Spider-Man, Brooklyn Visionary and Spiders-Man, Heroic Horde print it.
//!
//! A card with it lists `way(cost, text)` among its `ways`: an alternative cost (CR 118.9) used from the hand, at the
//! spell's own timing, following CR 601.2b and 601.2f-h. The creature is chosen as the cost is paid, once the mana is
//! (`PermanentCost::tapped`): it must be tapped then, not before casting began, so one tapped for a mana ability while
//! paying the web-slinging cost may be the one returned (the 2025-09-19 ruling).

use cantrip_api::*;

use crate::targets::filters::creature;

/// The alternative cost of web-slinging: `cost` and a tapped creature you control returned to its owner's hand. `text`
/// is the way's text ("Web-slinging {U}"), which a scenario's `Ability:` matches and the client shows.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost::mana(cost).and(return_tapped_to_hand(1, creature)), ZoneKind::Hand, text).kind(WayKind::WebSlinging)
}

/// "Was cast using web-slinging" (CR 702.188a): the spell, or the permanent it became, was cast through the way a card
/// lists as web-slinging (`WayKind::WebSlinging`, set by `way`). Another way, a copy and a permanent that was not cast
/// were not. Spiders-Man, Heroic Horde and Scarlet Spider, Ben Reilly ask it.
pub fn was_cast_using_web_slinging(q: &dyn Query, id: ObjectId) -> bool {
    let object = q.state().obj(id);
    object.cast_way > 0 && q.card(object.card).ways.get(object.cast_way as usize - 1).is_some_and(|w| w.kind == WayKind::WebSlinging)
}
