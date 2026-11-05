use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if a creature is attacking you" (CR 601.2f): judged once, as the total cost
/// is determined, for the player casting it. A creature attacking a planeswalker or a battle of theirs is not attacking them
/// (CR 508.1b), and an attacker of their own attacking someone else is not either; generic mana only (CR 118.7a).
fn less_if_a_creature_attacks_you(q: &dyn Query, _: ObjectId, caster: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.state().attackers.iter().any(|a| a.target == Target::Player(caster)) { CostDelta::less(2) } else { CostDelta::NONE }
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_if_a_creature_attacks_you],
    spell: spell(&[], effects::creatures_you_control_get::<2, 0>),
    ..oracle::RED_ALERT
};
