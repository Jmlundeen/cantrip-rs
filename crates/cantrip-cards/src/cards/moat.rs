use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures without flying can't attack." (CR 508.1c): a restriction on every creature, whoever controls it, judged as
/// attackers are declared with the creature as the layers have it then. A creature already attacking stays so if it
/// loses flying afterwards (CR 506.4a).
fn only_flyers_attack(q: &dyn Query, _: ObjectId, attacker: ObjectId) -> bool {
    q.chars(attacker).has(Keyword::Flying)
}

pub static CARD: CardDef = CardDef { attack_rules: &[only_flyers_attack], ..oracle::MOAT };
