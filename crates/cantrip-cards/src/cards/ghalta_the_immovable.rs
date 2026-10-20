use cantrip_api::*;
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
        .clamp(0, u8::MAX as i16);
    CostDelta::less(greatest as u8)
}

/// "Creatures you control can attack as though they didn't have defender" (CR 702.3b).
fn your_creature(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    q.controller(creature) == asker.controller && q.chars(creature).is(CardType::Creature)
}

/// "Each creature you control with toughness greater than its power assigns combat damage equal to its toughness rather
/// than its power" (CR 510.1a): judged as combat damage is assigned.
fn your_creature_tougher_than_strong(q: &dyn Query, asker: &Asker, creature: ObjectId) -> bool {
    let chars = q.chars(creature);
    your_creature(q, asker, creature) && chars.toughness.unwrap_or(0) > chars.power.unwrap_or(0)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_by_greatest_toughness],
    attacks_despite_defender: &[your_creature],
    damage_by_toughness: &[your_creature_tougher_than_strong],
    ..oracle::GHALTA_THE_IMMOVABLE
};
