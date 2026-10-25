//! Goad (CR 701.15), a keyword action: until the next turn of the controller of the spell or ability that goads it, the
//! creature is goaded - it attacks each combat if able, and attacks a player other than that controller if able (CR 701.15b),
//! which the core enforces as attackers are declared. Goaded is a designation, neither an ability nor a copiable value: a
//! goaded creature that loses its abilities is still goaded. Undercity's Arena goads; so do Disrupt Decorum, Kardur, Doomscourge
//! and the other goad cards when they are built. A card that prints its own duration ("goaded for the rest of the game":
//! Dack Fayden, Helping Hand, Jon Irenicus, Life of the Party) goads for that long instead (`goad_for`).

use cantrip_api::*;

/// `item`'s controller goads `creature` (CR 701.15a): it is goaded until that player's next turn. A creature that player has
/// already goaded is not goaded again (CR 701.15d: no further requirement); another player's goad is a requirement of its own
/// (CR 701.15c).
pub fn goad(g: &mut dyn Game, item: &StackItem, creature: ObjectId) {
    goad_for(g, item, creature, Duration::UntilNextTurnOf(item.controller));
}

/// `item`'s controller goads `creature` for the `duration` the card prints in place of "until your next turn" ("for the rest
/// of the game" is `Duration::Indefinite`, CR 611.2a). The goader is the controller of the spell or ability (CR 701.15b),
/// and stays that player whoever later controls its source or the creature. Goading a creature that player has already
/// goaded has no effect (CR 701.15d), whatever either duration.
pub fn goad_for(g: &mut dyn Game, item: &StackItem, creature: ObjectId, duration: Duration) {
    let goader = item.controller;
    if g.state().effects.iter().any(|e| e.affected == creature && e.kind == EffectKind::Goaded && e.controller == goader) {
        return;
    }
    g.add_effect_by(goader, item.object, creature, EffectKind::Goaded, duration);
}
