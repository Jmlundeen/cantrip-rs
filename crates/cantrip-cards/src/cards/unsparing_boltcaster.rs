use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, history, targets};
use cantrip_oracle as oracle;

/// "Target creature an opponent controls that was dealt damage this turn" (CR 115.1, 120.2): judged as it is chosen and again as
/// the ability resolves (CR 608.2b).
fn creature_an_opponent_controls_that_was_dealt_damage_this_turn(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_AN_OPPONENT_CONTROLS.filter)(q, asker, t) && matches!(t, Target::Object(id) if history::was_dealt_damage_this_turn(q.history(), id))
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::damage_first_target::<5>,
        "When this creature enters, it deals 5 damage to target creature an opponent controls that was dealt damage this turn.",
    )
    .targets(&[target(creature_an_opponent_controls_that_was_dealt_damage_this_turn)])],
    ..oracle::UNSPARING_BOLTCASTER
};
