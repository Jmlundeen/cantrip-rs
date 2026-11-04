use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast if it targets a Spider" (CR 601.2c, 601.2f, 118.7a): the target as the total cost is
/// determined, once it is announced. "Spider" is the word of this spell's own text (CR 612).
fn costs_less_if_it_targets_a_spider(q: &dyn Query, source: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let spider = q.words(source).subtype(Subtype::Spider);
    if targets.iter().filter_map(|t| t.object()).any(|object| q.chars(object).has_subtype(spider)) {
        CostDelta::less(1)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::target_creature_gets::<4, 4>),
    own_cost_changes: &[costs_less_if_it_targets_a_spider],
    ..oracle::GROW_EXTRA_ARMS
};
