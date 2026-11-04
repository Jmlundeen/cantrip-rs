use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This spell costs {3} less to cast if you've cast another spell this turn" (CR 601.2f). Asked of the event history, so a
/// countered spell or one still on the stack counts and the reduction applies once however many were cast; this spell is not yet
/// cast while its cost is worked out (CR 601.2i), so it is never among them.
fn less_after_another_spell(q: &dyn Query, _: ObjectId, player: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if q.history().casts_by(player).next().is_some() {
        CostDelta::less(3)
    } else {
        CostDelta::NONE
    }
}

pub static CARD: CardDef = CardDef { own_cost_changes: &[less_after_another_spell], ..oracle::GIGASTORM_TITAN };
