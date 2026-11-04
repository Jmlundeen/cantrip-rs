use cantrip_api::*;
use cantrip_mechanics::{creature_types_among_non_borg, damage_target, targets, triggers};
use cantrip_oracle as oracle;

/// "Target attacking or blocking creature an opponent controls" (CR 508.1, 509.1): a creature that is attacking or
/// blocking now, controlled by a player other than Saavik's controller.
fn attacking_or_blocking_an_opponent_controls(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ATTACKING_OR_BLOCKING_CREATURE.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.controller(id) != asker.controller)
}

/// "She deals X damage to target attacking or blocking creature an opponent controls, where X is the number of creature
/// types among non-Borg creatures you control." X is determined once as the ability resolves (CR 608.2h); a target that
/// is no longer legal takes no damage (CR 608.2b).
fn damage_by_creature_types(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let x = creature_types_among_non_borg(g, item.object, item.controller) as u16;
    damage_target(g, item, 0, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ENTERS,
        damage_by_creature_types,
        "Federation \u{2014} When Saavik enters, she deals X damage to target attacking or blocking creature an opponent controls, where X is the number of creature types among non-Borg creatures you control.",
    )
    .targets(&[target(attacking_or_blocking_an_opponent_controls)])],
    ..oracle::SAAVIK_STOIC_STUDENT
};
