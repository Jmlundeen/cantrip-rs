use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creature spells you cast cost {2} less to cast." (CR 604.1).
fn creature_spells_cast_cost_2_rule(
    q: &dyn Query,
    source: ObjectId,
    player: PlayerId,
    costed: Costed,
    _: &[Target],
) -> CostDelta {
    cantrip_mechanics::cost_reduction::spells_you_cast(q, source, player, costed, CardType::Creature, 2)
}

pub static CARD: CardDef = CardDef {
    cost_changes: &[creature_spells_cast_cost_2_rule],
    ..oracle::PLANAR_GATE
};
