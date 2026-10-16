use cantrip_api::*;
use cantrip_mechanics::{blight, destroy_target, targets};
use cantrip_oracle as oracle;

/// "Target creature with mana value 2 or less" (CR 115.1, 202.3), asked as the spell is cast and again as it resolves
/// (CR 608.2b).
fn creature_mv2(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.mana_value(id) <= 2)
}

/// "Destroy target creature with mana value 2 or less. If this spell's additional cost was paid, you gain 2 life."
/// The life is gained whether or not the creature was destroyed (it may be indestructible); a spell whose only target
/// became illegal does nothing at all (CR 608.2b). A copy of the spell has the cost paid when the original did
/// (CR 707.10).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    if blight::blighted(item) {
        g.gain_life(item.controller, 2);
    }
    Ok(())
}

/// "As an additional cost to cast this spell, you may blight 1" (CR 701.68a, 601.2b): an optional cost announced before
/// the target (CR 601.2b-c), which is chosen after it. (The tactical bot never pays it: a known limitation.) Cinder Strike, Burning Curiosity and Pyrrhic Strike also blight as an optional cost.
pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: creature_mv2 }], effect }),
    late_targets: true,
    optional_cost: Some(blight::optional(1)),
    labels: &["Cast without blighting", "Blight 1: put a -1/-1 counter on a creature you control"],
    ..oracle::REQUITING_HEX
};
