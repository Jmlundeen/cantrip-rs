//! Regeneration (CR 701.19).

use cantrip_api::*;

/// "Regenerate target permanent" (CR 701.19a): a regeneration shield on it until end of turn, if it is still a legal
/// target. Welding Jar and Yavimaya Hollow print it.
pub fn target(g: &mut dyn Game, item: &StackItem, slot: usize) {
    if !g.target_legal(item, slot) {
        return;
    }
    if let Target::Object(permanent) = item.targets[slot] {
        g.add_effect(item.object, permanent, EffectKind::RegenerationShield, Duration::EndOfTurn);
    }
}
