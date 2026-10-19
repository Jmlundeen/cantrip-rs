use cantrip_api::*;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if you've cast a noncreature spell this turn" Check earlier casts this
/// turn, including countered spells.
fn less_after_a_noncreature_spell(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.history().casts_by(player).any(|cast| !cast.is(CardType::Creature)) {
        CostDelta::less(2)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggers::PROWESS],
    own_cost_changes: &[less_after_a_noncreature_spell],
    ..oracle::TRAXOS_ACADEMY_GUARDIAN
};
