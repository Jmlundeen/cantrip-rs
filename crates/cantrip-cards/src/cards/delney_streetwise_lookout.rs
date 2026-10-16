use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures you control with power 2 or less can't be blocked by creatures with power 3 or greater." (CR 509.1b): of a
/// possible block, both creatures as they are as blockers are declared; a block that has been made stands if powers
/// change afterwards (the rulings).
fn small_ones_evade_big_blockers(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    !(q.controller(attacker) == q.controller(source)
        && q.chars(attacker).power.is_some_and(|p| p <= 2)
        && q.chars(blocker).power.is_some_and(|p| p >= 3))
}

/// "If a triggered ability of a creature you control with power 2 or less triggers, that ability triggers an additional
/// time." (CR 603.2d): the creature as it is when the ability triggers - as it last existed on the battlefield for one that
/// looks back - so a later change of power neither adds nor removes a triggering (the rulings).
fn small_creatures_trigger_again(q: &dyn Query, asker: &Asker, _: &Event, triggering: &Triggering) -> bool {
    let c = q.chars(triggering.source);
    triggering.of_a_permanent_controlled_by(asker.controller) && c.is(CardType::Creature) && c.power.is_some_and(|p| p <= 2)
}

pub static CARD: CardDef = CardDef {
    block_rules: &[small_ones_evade_big_blockers],
    additional_triggers: &[small_creatures_trigger_again],
    ..oracle::DELNEY_STREETWISE_LOOKOUT
};
