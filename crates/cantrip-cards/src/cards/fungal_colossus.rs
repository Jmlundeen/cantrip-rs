use cantrip_api::*;
use cantrip_mechanics::lands;
use cantrip_oracle as oracle;

/// "This spell costs {X} less to cast, where X is the number of differently named lands you control." (CR 601.2f,
/// 118.7a): a generic reduction of the spell's casting cost, with the land count read as the total cost is determined.
fn cost_reduction(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    CostDelta::less(lands::differently_named_lands(q, player) as u32)
}

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cost_reduction],
    ..oracle::FUNGAL_COLOSSUS
};
