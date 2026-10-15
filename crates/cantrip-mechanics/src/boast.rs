//! Boast (CR 702.142a): "[Cost]: [Effect]. Activate only if this creature attacked this turn and only once each turn."
//! Broadside Bombardiers prints it.
//!
//! A card with boast lists `ability(text, cost, targets, effect)` among its `abilities`.

use cantrip_api::*;

fn attacked_this_turn(q: &dyn Query, asker: &Asker) -> bool {
    q.attacked_this_turn(asker.source)
}

/// The boast ability: activated only if its source attacked this turn, and, by `once_each_turn`, once each turn.
pub const fn ability(
    text: &'static str,
    cost: Cost,
    targets: &'static [TargetSpec],
    effect: fn(&mut dyn Game, &StackItem) -> Result<(), Suspend>,
) -> AbilityDef {
    AbilityDef::Activated(ActivatedDef {
        text,
        cost,
        targets,
        effect,
        condition: Some(attacked_this_turn),
        once_each_turn: true,
        ..ActivatedDef::BLANK
    })
}
