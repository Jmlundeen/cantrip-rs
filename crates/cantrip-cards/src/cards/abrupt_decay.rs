use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target nonland permanent with mana value 3 or less."
fn nonland_permanent_mv3(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::NONLAND_PERMANENT.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 3)
}

/// Destroy target nonland permanent with mana value 3 or less. (It can't be countered: the card's keyword.)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: nonland_permanent_mv3 }], effect }),
    ..oracle::ABRUPT_DECAY
};
