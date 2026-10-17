//! Outlaws (CR 700.12): "The term outlaw refers to an object that has the Assassin, Mercenary, Pirate, Rogue, and/or Warlock
//! creature type." Cards refer to the term, not to the five types in their text, so a text-changing effect (CR 612) does not
//! reach it. Shoot the Sheriff ("non-outlaw") and Laughing Jasper Flint ("outlaws you control", CR 700.12a) print it.

use cantrip_api::*;

const OUTLAW_TYPES: [Subtype; 5] = [Subtype::Assassin, Subtype::Mercenary, Subtype::Pirate, Subtype::Rogue, Subtype::Warlock];

/// Whether an object with these characteristics is an outlaw.
pub fn is_outlaw(chars: &Chars) -> bool {
    OUTLAW_TYPES.iter().any(|&s| chars.has_subtype(s))
}

/// How many outlaws `player` controls (CR 700.12a): permanents only, as they are now.
pub fn outlaws_controlled_by(q: &dyn Query, player: PlayerId) -> usize {
    q.permanents_of(player).into_iter().filter(|&id| is_outlaw(&q.chars(id))).count()
}
