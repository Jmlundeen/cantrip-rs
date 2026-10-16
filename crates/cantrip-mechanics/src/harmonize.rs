//! Harmonize [cost] (CR 702.180a): "You may cast this card from your graveyard by paying [cost] and tapping up to one
//! untapped creature you control rather than paying this spell's mana cost", "If you cast this spell using its harmonize
//! ability, its total cost is reduced by an amount of generic mana equal to the tapped creature's power", and "If the harmonize
//! cost was paid, exile this card instead of putting it anywhere else any time it would leave the stack". Wild Ride, Glacial
//! Dragonhunt, Channeled Dragonfire and Winternight Stories print it.
//!
//! A card with harmonize lists `way(cost, text)` among its `ways`. The creature is part of the way's cost, `Cost::tap_to_reduce`,
//! which the core pays by the rules for costs (CR 702.180b, 601.2b, 601.2f): the creature is chosen as the harmonize cost is
//! chosen, before any target; the total cost comes down by its power in generic mana only, read as the total cost is
//! determined; it is tapped as the cost is paid, summoning sick or not (it is not a {T} ability's cost, CR 107.5).

use cantrip_api::*;

/// The alternative cost of harmonize: cast from the graveyard (CR 118.9) for `mana`, tapping up to one untapped creature to
/// reduce it, exiled as it leaves the stack. `text` is the way's full text ("Harmonize {4}{R}").
pub const fn way(mana: ManaCost, text: &'static str) -> WayDef {
    cantrip_api::way(Cost { mana, tap_to_reduce: true, ..Cost::NONE }, ZoneKind::Graveyard, text).exile_after()
}
