use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {2} more to cast if it targets a Dragon" (CR 601.2c, 601.2f): the target as the total cost is
/// determined, once it is announced. "Dragon" is the word of this spell's own text (CR 612).
fn costs_more_if_it_targets_a_dragon(q: &dyn Query, source: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let dragon = q.words(source).subtype(Subtype::Dragon);
    if targets.iter().filter_map(|t| t.object()).any(|object| q.chars(object).has_subtype(dragon)) {
        CostDelta::more(2)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::destroy_first_target),
    own_cost_changes: &[costs_more_if_it_targets_a_dragon],
    ..oracle::DRAGONS_PREY
};
