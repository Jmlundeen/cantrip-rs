use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target creature or planeswalker with mana value 2 or less", or any creature or planeswalker when the spell was
/// kicked: judged by the kicker count announced before the target is chosen, and again as it resolves (CR 601.2c,
/// 608.2b).
fn creature_or_planeswalker(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_PLANESWALKER.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.obj(asker.source).kicked > 0 || q.mana_value(id) <= 2)
}

/// Destroy target creature or planeswalker with mana value 2 or less. If this spell was kicked, instead destroy
/// target creature or planeswalker.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(creature_or_planeswalker)], effect),
    kicker: kicker(mana("{2}{B}")),
    late_targets: true,
    ..oracle::BLOODCHIEFS_THIRST
};
