use cantrip_api::*;
use cantrip_mechanics::{effects, ferocious, targets};
use cantrip_oracle as oracle;

/// "This spell costs {3} less to cast if you control a creature with power 4 or greater" (CR 601.2f): judged as the total
/// cost is determined, once, and generic mana only (CR 118.7a); losing the creature afterwards changes nothing paid.
fn less_with_a_big_creature(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if matches!(costed, Costed::Spell(_)) && ferocious::you_control(q, player) { CostDelta::less(3) } else { CostDelta::NONE }
}

/// "Change the target of target spell or ability with a single target" (CR 115.7a). The new target is chosen as this
/// resolves, by Bolt Bend's controller, and must differ from the old one when another legal one exists (rulings 2024-11-08).
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL_OR_ABILITY_WITH_A_SINGLE_TARGET], effects::change_first_target),
    own_cost_changes: &[less_with_a_big_creature],
    ..oracle::BOLT_BEND
};
