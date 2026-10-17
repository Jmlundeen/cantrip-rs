use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Each spell costs {3} more to cast except during its controller's turn" (CR 601.2f): the spell's controller is the
/// player casting it (CR 601.2a), so it costs more whenever that player is not the active player.
fn tax(q: &dyn Query, _: ObjectId, player: PlayerId, costed: Costed, _: &[Target]) -> CostDelta {
    match costed {
        Costed::Spell(_) if q.state().turn.active != player => CostDelta::more(3),
        _ => CostDelta::NONE,
    }
}

pub static CARD: CardDef = CardDef { cost_changes: &[tax], ..oracle::DEFENSE_GRID };
