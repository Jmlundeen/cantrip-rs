use cantrip_api::*;
use cantrip_mechanics::{library, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast if it targets an attacking creature" (CR 601.2c, 601.2f): the condition is checked
/// with the announced target as the total cost is determined; a Spacecraft that isn't a creature cannot be attacking.
fn less_for_an_attacking_creature(
    q: &dyn Query,
    _: ObjectId,
    _: PlayerId,
    _: Costed,
    targets: &[Target],
) -> CostDelta {
    let attacking_creature = targets
        .iter()
        .filter_map(|target| target.object())
        .any(|id| q.is_attacking(id) && q.chars(id).is(CardType::Creature));
    if attacking_creature {
        CostDelta::less(1)
    } else {
        CostDelta::NONE
    }
}

/// "Target creature or Spacecraft's owner puts it on their choice of the top or bottom of their library." The owner (CR
/// 108.3), not controller, chooses. Spacecraft need not currently be creatures (CR 205.3g).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(object) = targets::object(g, item, 0) else {
        return Ok(());
    };
    library::owner_puts_on_top_or_bottom(g, item.object, object)
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_OR_SPACECRAFT], effect),
    own_cost_changes: &[less_for_an_attacking_creature],
    ..oracle::GUIDANCE_FAILURE
};
