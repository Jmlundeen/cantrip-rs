use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "If you control a creature with a counter on it" (CR 603.4): a counter of any kind, on a creature its controller controls as
/// the attack trigger would be put on the stack, and again as it resolves.
fn you_control_a_creature_with_a_counter(q: &dyn Query, asker: &Asker) -> bool {
    q.permanents_of(asker.controller).into_iter().any(|id| q.chars(id).is(CardType::Creature) && q.state().obj(id).has_counters())
}

/// Flying is a core keyword.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::THIS_ATTACKS,
        effects::each_opponent_loses_1_life,
        "Whenever this creature attacks, if you control a creature with a counter on it, each opponent loses 1 life.",
    )
    .condition(you_control_a_creature_with_a_counter)],
    ..oracle::DELTA_BLOODFLIES
};
