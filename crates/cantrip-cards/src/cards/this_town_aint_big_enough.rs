use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "This spell costs {3} less to cast if it targets a permanent you control" (CR 601.2c, 601.2f): judged once the targets
/// are announced, as the total cost is determined; generic mana only (CR 118.7a). Any one of the targets being under the
/// caster's control is enough.
fn less_for_a_target_you_control(q: &dyn Query, _: ObjectId, caster: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let yours = targets.iter().filter_map(|t| t.object()).any(|permanent| q.controller(permanent) == caster);
    if yours { CostDelta::less(3) } else { CostDelta::NONE }
}

/// "Return up to two target nonland permanents to their owners' hands." Those still legal go together (CR 608.2b,
/// 400.7); none chosen, nothing happens.
fn return_them(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.return_to_hand(&targets::legal_objects(g, item));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[up_to(2, targets::NONLAND_PERMANENT.filter)], return_them),
    own_cost_changes: &[less_for_a_target_you_control],
    ..oracle::THIS_TOWN_AINT_BIG_ENOUGH
};
