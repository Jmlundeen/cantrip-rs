use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {1} less to cast if it targets a blocking creature" (CR 601.2c, 601.2f): the target as the total cost is
/// determined, once it is announced; generic mana only (CR 118.7a). Only a creature can be blocking (CR 506.4, 509.1).
fn costs_one_less_if_it_targets_a_blocking_creature(q: &dyn Query, _: ObjectId, _: PlayerId, _: Costed, targets: &[Target]) -> CostDelta {
    let blocking = targets.iter().filter_map(|t| t.object()).any(|creature| q.state().blocks.iter().any(|b| b.blocker == creature));
    if blocking { CostDelta::less(1) } else { CostDelta::NONE }
}

/// "Target creature you control gets +2/+0 and gains indestructible until end of turn" (CR 611.2a, 702.12): both come from the
/// spell, to the creature it targeted, if it is still a legal target as this resolves (CR 608.2b).
fn flourish(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    instruction(g, |g| {
        cantrip_mechanics::pump(g, item.object, creature, 2, 0);
        effects::gain_indestructible_until_end_of_turn(g, item.object, creature);
    });
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], flourish),
    own_cost_changes: &[costs_one_less_if_it_targets_a_blocking_creature],
    ..oracle::DARK_ENDURANCE
};
