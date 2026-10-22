//! Goad (CR 701.15), a keyword action: until the next turn of the controller of the spell or ability that goads it, the
//! creature is goaded - it attacks each combat if able, and attacks a player other than that controller if able (CR 701.15b),
//! which the core enforces as attackers are declared. Goaded is a designation, neither an ability nor a copiable value: a
//! goaded creature that loses its abilities is still goaded. Undercity's Arena goads; so do Disrupt Decorum, Kardur, Doomscourge
//! and the other goad cards when they are built.

use cantrip_api::*;

/// `item`'s controller goads `creature` (CR 701.15a): it is goaded until that player's next turn. A creature that player has
/// already goaded is not goaded again (CR 701.15d: no further requirement); another player's goad is a requirement of its own
/// (CR 701.15c).
pub fn goad(g: &mut dyn Game, item: &StackItem, creature: ObjectId) {
    let goader = item.controller;
    if g.state().effects.iter().any(|e| e.affected == creature && e.kind == EffectKind::Goaded && e.controller == goader) {
        return;
    }
    g.add_effect(item.object, creature, EffectKind::Goaded, Duration::UntilNextTurnOf(goader));
}
