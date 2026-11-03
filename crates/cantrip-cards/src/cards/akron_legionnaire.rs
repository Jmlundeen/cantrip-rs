use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Except for creatures named Akron Legionnaire and artifact creatures, creatures you control can't attack." (CR 604.1).
fn except_creatures_named_akron_legionnaire_rule(
    q: &dyn Query,
    asker: &Asker,
    attacker: ObjectId,
    _: Target,
) -> bool {
    let asker = &Asker::new(asker.source, q.controller(asker.source));
    !((!((q.chars(attacker).is(CardType::Creature)
        && q.chars(attacker).name == "Akron Legionnaire"))
        && !((q.chars(attacker).is(CardType::Artifact)
            && q.chars(attacker).is(CardType::Creature)))
        && (q.chars(attacker).is(CardType::Creature)
            && asker.controller == q.controller(attacker))))
}

pub static CARD: CardDef = CardDef {
    attack_rules: &[except_creatures_named_akron_legionnaire_rule],
    ..oracle::AKRON_LEGIONNAIRE
};
