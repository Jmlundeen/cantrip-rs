use cantrip_api::*;
use cantrip_mechanics::{combat, statics};
use cantrip_oracle as oracle;

/// "This spell costs {X} less to cast, where X is the greatest toughness among creatures you control" (CR 601.2f): the
/// creatures its caster controls as the total cost is determined, a toughness below zero counting as nothing, and only the
/// generic part of the cost reduced (CR 118.7a).
fn less_by_greatest_toughness(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let greatest = q
        .state()
        .battlefield()
        .iter()
        .filter(|&&id| q.controller(id) == player)
        .map(|&id| q.chars(id))
        .filter(|c| c.is(CardType::Creature))
        .map(|c| c.toughness.unwrap_or(0))
        .max()
        .unwrap_or(0)
        .max(0);
    CostDelta::less(greatest as u32)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_by_greatest_toughness],
    attacks_despite_defender: &[statics::creature_you_control],
    damage_by_toughness: &[combat::your_creature_with_toughness_greater_than_power],
    ..oracle::GHALTA_THE_IMMOVABLE
};
