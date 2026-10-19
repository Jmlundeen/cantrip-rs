//! Fight (CR 701.14), the keyword action, for the spells and abilities that name two targets.

use cantrip_api::*;

use crate::targets;

/// "Target creature fights target creature" (CR 701.14a): the creatures `item` targeted in `first` and `second` each deal
/// damage equal to their power to the other, at once (not combat damage, CR 701.14d). If either target is an illegal
/// target as the spell or ability resolves, neither fights or deals damage (CR 701.14b); the verb itself checks that both
/// are still creatures on the battlefield. Bushwhack prints it; Mawloc and Wolverine, Fierce Fighter, whose one target is
/// fought by the source itself, call `Game::fight` directly.
pub fn targets_fight(g: &mut dyn Game, item: &StackItem, first: usize, second: usize) {
    if let (Some(a), Some(b)) = (targets::object(g, item, first), targets::object(g, item, second)) {
        g.fight(a, b);
    }
}

/// "[Creature] deals damage equal to its power to [target]" (CR 120.3, 609.7, 120.8): one-sided, the creature being the
/// source of the damage, so its abilities (lifelink, deathtouch) and colors count. Its power is read now, as the spell or
/// ability resolves (CR 608.2c); a creature with no power deals none. Hard-Hitting Question and Archdruid's Charm print it.
pub fn bite(g: &mut dyn Game, creature: ObjectId, victim: ObjectId) {
    let power = g.chars(creature).power.unwrap_or(0);
    if power > 0 {
        g.damage(creature, Target::Object(victim), power as u16);
    }
}
