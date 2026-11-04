use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This creature can't attack alone" (CR 508.1c): a restriction on the whole declaration, not on one attack - it attacks
/// as one of two or more attackers, and no other creature's attack makes any difference. Asked of this creature only.
fn cant_attack_alone(_: &dyn Query, asker: &Asker, creature: ObjectId, _: Target) -> bool {
    creature != asker.source
}

pub static CARD: CardDef = CardDef {
    attack_alone_rules: &[cant_attack_alone],
    ..oracle::PET_SEHLAT
};
