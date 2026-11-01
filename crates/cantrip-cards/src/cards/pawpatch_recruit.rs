use cantrip_api::*;
use cantrip_mechanics::{offspring, targets, triggers};
use cantrip_oracle as oracle;

/// "Target creature you control other than that creature" (CR 115.1): any creature its controller controls but the one that
/// was targeted, the trigger's subject. With no other creature, the ability has no target and is removed (CR 603.3d).
fn creature_you_control_other_than_that_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    t != q.event_subject() && (targets::CREATURE_YOU_CONTROL.filter)(q, asker, t)
}

/// "Put a +1/+1 counter on target creature you control other than that creature." A target no longer legal means the
/// ability does nothing (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    cantrip_mechanics::effects::plus_one_counter_on_target(g, item)
}

pub static CARD: CardDef = CardDef {
    kicker: Some(offspring::kicker(mana("{2}"))),
    abilities: &[
        offspring::trigger("Offspring {2} (You may pay an additional {2} as you cast this spell. If you do, when this creature enters, create a 1/1 token copy of it.)"),
        triggered(
            triggers::YOUR_CREATURE_TARGETED_BY_AN_OPPONENT,
            counter,
            "Whenever a creature you control becomes the target of a spell or ability an opponent controls, put a +1/+1 counter on target creature you control other than that creature.",
        )
        .targets(&[target(creature_you_control_other_than_that_creature)]),
    ],
    ..oracle::PAWPATCH_RECRUIT
};
