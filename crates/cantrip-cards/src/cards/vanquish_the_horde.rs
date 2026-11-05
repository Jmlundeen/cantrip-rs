use cantrip_api::*;
use cantrip_mechanics::effects::destroy_all_creatures_no_regen;
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast for each creature on the battlefield." (CR 601.2f, 118.7a): counted as the cost
/// is determined, every creature every player controls.
fn less_per_creature_on_the_battlefield(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    let creatures = q.state().seats().flat_map(|p| q.permanents_of(p)).filter(|&id| q.chars(id).is(CardType::Creature)).count();
    CostDelta::less(creatures as u32)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[less_per_creature_on_the_battlefield],
    spell: spell(&[], destroy_all_creatures_no_regen),
    ..oracle::VANQUISH_THE_HORDE
};
