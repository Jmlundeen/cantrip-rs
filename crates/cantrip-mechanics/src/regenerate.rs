//! Regeneration (CR 701.19).

use cantrip_api::*;

/// Regenerate a permanent (CR 701.19a): create its shield until end of turn if it is still the same permanent.
/// Regeneration, Golgari Grave-Troll, Welding Jar and Yavimaya Hollow use this keyword action.
pub fn permanent(g: &mut dyn Game, source: ObjectId, object: ObjectId) {
    if crate::targets::permanent(g, object) {
        g.add_effect(source, object, EffectKind::RegenerationShield, Duration::EndOfTurn);
    }
}

/// "Regenerate this creature" (CR 701.19a): the ability's source. Golgari Grave-Troll prints it.
pub fn this(g: &mut dyn Game, item: &StackItem) {
    permanent(g, item.object, item.object);
}

/// "Regenerate this creature" as a whole ability effect (CR 701.19a): Scavenging Ghoul and Golgari Grave-Troll.
pub fn this_effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    this(g, item);
    Ok(())
}

/// "Regenerate target permanent" (CR 701.19a): if it is still a legal target. Welding Jar and Yavimaya Hollow print it.
pub fn target(g: &mut dyn Game, item: &StackItem, slot: usize) {
    if !g.target_legal(item, slot) {
        return;
    }
    if let Target::Object(object) = g.state().target(item, slot) {
        permanent(g, item.object, object);
    }
}
