//! Beam me up [cost]: "You may cast this card from your graveyard for [cost] if you also return a creature you control to
//! its owner's hand. Then exile this spell." An alternative cost (CR 118.9) paid from the graveyard, the creature
//! returned as the cost is paid (CR 601.2h), and the spell exiled as it leaves the stack. Adversary by Algorithm,
//! Energize, Open Communications, Pure Energy and There's Coffee in that Nebula print it.
//!
//! A card with it lists `way(cost, text)` among its `ways`.

use cantrip_api::*;

use crate::targets::filters::creature;

/// The alternative cost of beam me up: `cost` and a creature you control returned to its owner's hand, cast from the
/// graveyard and exiled as it leaves the stack. `text` is the way's text ("Beam me up {3}{G}"), which a scenario's
/// `Ability:` matches and the client shows.
pub const fn way(cost: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost::mana(cost).and(return_to_hand(1, creature)), ZoneKind::Graveyard, text).exile_after()
}
