use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {3} less to cast if it targets a tapped permanent" (CR 601.2c, 601.2f): judged once the target is
/// announced, as the total cost is determined; generic mana only (CR 118.7a). Anything done to the permanent afterwards
/// changes nothing paid.
fn less_for_a_tapped_target(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let tapped = targets.iter().filter_map(|t| t.object()).any(|permanent| q.obj(permanent).tapped);
    if tapped { CostDelta::less(3) } else { CostDelta::NONE }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_VEHICLE], effects::exile_first_target),
    own_cost_changes: &[less_for_a_tapped_target],
    ..oracle::RIDES_END
};
