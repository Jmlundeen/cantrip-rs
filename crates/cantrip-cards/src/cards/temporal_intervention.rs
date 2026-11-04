use cantrip_api::*;
use cantrip_mechanics::{hand::discard_chosen_nonland_from_revealed_hand, history, targets};
use cantrip_oracle as oracle;

/// "Void — This spell costs {2} less to cast if a nonland permanent left the battlefield this turn or a spell was warped this
/// turn" (CR 601.2f): judged as the total cost is determined, once, and generic mana only (CR 118.7a).
fn less_with_void(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, _: &[Target]) -> CostDelta {
    if history::void(q) { CostDelta::less(2) } else { CostDelta::NONE }
}

/// "Target opponent reveals their hand. You choose a nonland card from it. That player discards that card." (CR 701.9a)
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(opponent) = targets::player(g, item, 0) else { return Ok(()) };
    discard_chosen_nonland_from_revealed_hand(g, item, opponent)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::OPPONENT], effect),
    own_cost_changes: &[less_with_void],
    ..oracle::TEMPORAL_INTERVENTION
};
