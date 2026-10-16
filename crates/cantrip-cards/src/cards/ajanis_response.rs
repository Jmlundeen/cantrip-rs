use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "This spell costs {3} less to cast if it targets a tapped creature" (CR 601.2c, 601.2f): the target as the total cost
/// is determined, once it is announced; generic mana only (CR 118.7a). What happens to the creature afterwards changes
/// nothing paid.
fn less_for_a_tapped_target(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let tapped = targets.iter().filter_map(|t| t.object()).any(|creature| q.obj(creature).tapped);
    if tapped { CostDelta::less(3) } else { CostDelta::NONE }
}

/// "Destroy target creature."
fn destroy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], destroy),
    own_cost_changes: &[less_for_a_tapped_target],
    ..oracle::AJANIS_RESPONSE
};
