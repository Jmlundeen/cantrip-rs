use cantrip_api::*;
use cantrip_oracle as oracle;

/// "You control another creature with power 4 or greater": a creature its controller controls now other than this one, whose
/// power as it is now (CR 208.1, layers applied) is at least 4. A creature with no power does not count.
fn controls_another_big_creature(q: &dyn Query, this: ObjectId) -> bool {
    let me = q.controller(this);
    q.state().battlefield().iter().any(|&id| id != this && q.controller(id) == me && q.chars(id).is(CardType::Creature) && q.chars(id).power.is_some_and(|p| p >= 4))
}

/// "This creature can't attack ... unless you control another creature with power 4 or greater" (CR 508.1c): asked of this
/// creature as attackers are declared; once declared it stays attacking whatever happens to the other creature (CR 506.4).
fn attacks_only_with_another_big_creature(q: &dyn Query, asker: &Asker, attacker: ObjectId, _: Target) -> bool {
    asker.source != attacker || controls_another_big_creature(q, attacker)
}

/// "This creature can't ... block unless you control another creature with power 4 or greater" (CR 509.1b).
fn blocks_only_with_another_big_creature(q: &dyn Query, source: ObjectId, blocker: ObjectId, _: ObjectId) -> bool {
    source != blocker || controls_another_big_creature(q, blocker)
}

pub static CARD: CardDef = CardDef {
    attack_rules: &[attacks_only_with_another_big_creature],
    block_rules: &[blocks_only_with_another_big_creature],
    ..oracle::TIGER_DILLO
};
