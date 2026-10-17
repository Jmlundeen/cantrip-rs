use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Target creature or planeswalker you don't control" (CR 109.5, 115.1): a creature or a planeswalker under any other
/// player's control.
fn creature_or_planeswalker_you_dont_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_PLANESWALKER.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "Target creature you control deals damage equal to its power to target creature or planeswalker you don't control":
/// one-sided, and the creature is the source of the damage (CR 120.3, 609.7), so its abilities (lifelink, deathtouch) and
/// color count. Power is read as the spell resolves (CR 608.2c). If either target is illegal then, no damage is dealt
/// (CR 608.2b): a creature that is not a legal target cannot deal it, and there is nothing to deal it to. A creature with
/// no power deals none (CR 120.8).
fn bite(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let (Some(creature), Some(victim)) = (targets::object(g, item, 0), targets::object(g, item, 1)) {
        let power = g.chars(creature).power.unwrap_or(0);
        if power > 0 {
            g.damage(creature, Target::Object(victim), power as u16);
        }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL, target(creature_or_planeswalker_you_dont_control)], bite),
    ..oracle::HARD_HITTING_QUESTION
};
