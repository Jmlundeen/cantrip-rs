use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Non-Eye creatures you control can't attack. This creature can't be blocked except by Walls." (CR 604.1).
fn non_eye_creatures_control_cant_rule(
    q: &dyn Query,
    asker: &Asker,
    attacker: ObjectId,
    _: Target,
) -> bool {
    let asker = &Asker::new(asker.source, q.controller(asker.source));
    !((!(q.chars(attacker).has_subtype(q.words(asker.source).subtype(Subtype::Eye)))
        && q.chars(attacker).is(CardType::Creature)
        && asker.controller == q.controller(attacker)))
}

/// "Non-Eye creatures you control can't attack. This creature can't be blocked except by Walls." (CR 604.1).
fn non_eye_creatures_control_cant_rule_2(
    q: &dyn Query,
    source: ObjectId,
    blocker: ObjectId,
    attacker: ObjectId,
) -> bool {
    let asker = &Asker::new(source, q.controller(source));
    !(attacker == asker.source
        && !(q.chars(blocker).has_subtype(q.words(asker.source).subtype(Subtype::Wall))))
}

pub static CARD: CardDef = CardDef {
    block_rules: &[non_eye_creatures_control_cant_rule_2],
    attack_rules: &[non_eye_creatures_control_cant_rule],
    ..oracle::EVIL_EYE_OF_ORMS_BY_GORE
};
