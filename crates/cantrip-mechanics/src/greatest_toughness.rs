//! "The greatest toughness among creatures you control" (CR 208.1): Abzan Monument and Ghalta, the Immovable ask it.

use cantrip_api::*;

/// The greatest toughness among the creatures `player` controls, as the characteristics are now (CR 208.1). A creature has a
/// toughness, so a creature whose toughness is not a number is not one to count; with no creature, or none above 0, the
/// amount is 0, since a negative toughness used as an amount counts as 0 (CR 107.1b).
pub fn among_creatures_you_control(q: &dyn Query, player: PlayerId) -> u32 {
    q.permanents_of(player)
        .into_iter()
        .map(|id| q.chars(id))
        .filter(|c| c.is(CardType::Creature))
        .filter_map(|c| c.toughness)
        .max()
        .unwrap_or(0)
        .max(0) as u32
}
