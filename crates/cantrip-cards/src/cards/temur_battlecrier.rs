use cantrip_api::*;
use cantrip_mechanics::ferocious;
use cantrip_oracle as oracle;

/// "During your turn, spells you cast cost {1} less to cast for each creature you control with power 4 or greater."
/// (CR 601.2f, 118.7a): asked as the total cost is determined, for the spell's controller only, on that player's own
/// turn, and counting the creatures they control then (Battlecrier itself included); a generic reduction takes only the
/// generic part of the cost.
fn spells_cost_less(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    if !matches!(costed, Costed::Spell(_)) || player != q.controller(source) || q.state().turn.active != player {
        return CostDelta::NONE;
    }
    CostDelta::less(ferocious::count(q, player).min(u8::MAX as usize) as u8)
}

pub static CARD: CardDef = CardDef { cost_changes: &[spells_cost_less], ..oracle::TEMUR_BATTLECRIER };
