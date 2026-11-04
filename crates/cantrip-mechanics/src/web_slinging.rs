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

/// The cost of a web-slinging a permanent gives (CR 702.188a, 113.6, 604): `mana` and a tapped creature `player` controls
/// returned to its owner's hand, the same cost a card's own `way` asks.
pub fn granted_cost(mana: ManaCost) -> Cost {
    Cost::mana(mana).and(return_tapped_to_hand(1, creature))
}

/// Web-slinging a permanent gives to the spells in a hand (CR 702.188a, 113.6): "each legendary spell you cast that's one or
/// more colors has web-slinging {G}{W}{U}" (Amazing Spider-Man). `cost` is each card's web-slinging cost, `granted_cost` of
/// its mana, or none for a card that does not have it. Like a card's own it is cast at the spell's own timing, and a spell
/// cast by it was cast using web-slinging (`WayKind::WebSlinging`).
pub const fn granted(cost: GrantedCost) -> GrantedWay {
    GrantedWay { text: "Web-slinging", cost, from: ZoneKind::Hand, kind: WayKind::WebSlinging, ..GrantedWay::CAST }
}

/// "Was cast using web-slinging" (CR 702.188a): the spell, or the permanent it became, was cast through the way a card
/// lists as web-slinging (`WayKind::WebSlinging`, set by `way` and `granted`), the card's own or one a permanent gave it
/// (Amazing Spider-Man). Another way and a permanent that was not cast were not; a copy of a spell cast so copies its
/// alternative cost (CR 707.10) and was too. Spiders-Man, Heroic Horde and Scarlet Spider, Ben Reilly ask it.
pub fn was_cast_using_web_slinging(q: &dyn Query, id: ObjectId) -> bool {
    crate::sneak::cast_through_turn(q, id, WayKind::WebSlinging).is_some()
}
