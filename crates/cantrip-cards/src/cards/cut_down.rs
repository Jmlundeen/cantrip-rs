use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target creature with total power and toughness 5 or less": the computed values, so a counter or an Equipment
/// moves a creature in and out of range.
fn small_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if {
        let c = q.chars(id);
        c.power.unwrap_or(0) as i32 + c.toughness.unwrap_or(0) as i32 <= 5
    })
}

/// Destroy it. Nothing happens if it is no longer a legal target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: small_creature }], effect }),
    ..oracle::CUT_DOWN
};
