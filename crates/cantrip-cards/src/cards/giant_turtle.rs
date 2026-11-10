use cantrip_api::*;
use cantrip_mechanics::history;
use cantrip_oracle as oracle;

/// "This creature can't attack if it attacked during your last turn." (CR 604.1).
fn creature_cant_attack_if_attacked_rule(q: &dyn Query, asker: &Asker, attacker: ObjectId, _: Target) -> bool {
    !(attacker == asker.source && history::attacked_during_last_turn_of(q, q.controller(asker.source), asker.source))
}

pub static CARD: CardDef = CardDef {
    attack_rules: &[creature_cant_attack_if_attacked_rule],
    ..oracle::GIANT_TURTLE
};
