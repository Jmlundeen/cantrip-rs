use cantrip_api::*;
use cantrip_mechanics::attach::attached_object;
use cantrip_mechanics::statics::plus_pt;
use cantrip_mechanics::targets::{self, has_flying_creature};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast if it targets a creature with flying" (CR 601.2c, 601.2f, 702.9): the enchant target as the total cost is
/// determined, once it is announced, read from the layers; generic mana only (CR 118.7a). Whether it flies afterwards changes nothing paid, and
/// the mana value stays {2}{B} whatever was paid (the ruling).
fn costs_1_less_if_it_targets_a_creature_with_flying(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let flying = targets.iter().filter_map(|t| t.object()).any(|creature| has_flying_creature(q, creature));
    if flying { CostDelta::less(1) } else { CostDelta::NONE }
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    own_cost_changes: &[costs_1_less_if_it_targets_a_creature_with_flying],
    statics: &[static_ability(Layer::PtModify, attached_object, plus_pt::<-5, -3>, "Enchanted creature gets -5/-3.")],
    ..oracle::SWAMPSNARE_TRAP
};
