use cantrip_api::*;
use cantrip_mechanics::{counter_unless_pays, ferocious, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each creature you control with power 4 or greater" (CR 601.2f, 118.7a): counted once, as
/// the total cost is determined, generic mana only, {X} included.
fn less_for_each_big_creature(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(ferocious::count(q, player) as u32)
}

/// "Counter target spell unless its controller pays {X}" (CR 118.12, 107.3): X is the value chosen as this was cast, the reduction
/// above not changing it.
fn counter_unless_x(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_unless_pays(g, item, 0, generic(item.x as u32))?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], counter_unless_x),
    own_cost_changes: &[less_for_each_big_creature],
    ..oracle::SPECTRAL_DENIAL
};
