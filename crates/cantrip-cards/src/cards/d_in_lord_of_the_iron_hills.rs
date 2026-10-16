//! Dáin, Lord of the Iron Hills. Vigilance is a core keyword, read from its characteristics.

use cantrip_api::*;
use cantrip_mechanics::{statics, storied};
use cantrip_oracle as oracle;

/// "As long as you have an enduring story, creatures can't attack you unless their controller pays {1} for each of those
/// creatures." (CR 508.1d, 508.1h, 702.195b): only attacks on Dáin's controller cost anything, never one on a planeswalker
/// they control (the rulings), and only while that player has the designation.
fn attack_cost(q: &dyn Query, asker: &Asker, attacker: ObjectId, target: Target) -> Option<ManaCost> {
    if !storied::has_an_enduring_story(q.state(), asker.controller) {
        return None;
    }
    statics::attacking_you_costs::<1>(q, asker, attacker, target)
}

pub static CARD: CardDef = CardDef {
    // "Storied" (CR 702.195a): the rules give its controller an enduring story once they control three or more artifacts,
    // legendaries and/or Sagas.
    storied: true,
    attack_costs: &[attack_cost],
    ..oracle::D_IN_LORD_OF_THE_IRON_HILLS
};
