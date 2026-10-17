//! Regeneration (CR 701.19).

use cantrip_api::*;

/// "Regenerate this creature" (CR 701.19a): a regeneration shield on the ability's source until end of turn, if it is still
/// the permanent it was (CR 400.7). Golgari Grave-Troll prints it, and River Boa and Drudge Skeletons.
pub fn this(g: &mut dyn Game, item: &StackItem) {
    if crate::targets::permanent(g, item.object) {
        g.add_effect(item.object, item.object, EffectKind::RegenerationShield, Duration::EndOfTurn);
    }
}

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
