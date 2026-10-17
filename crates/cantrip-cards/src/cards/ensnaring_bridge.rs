use cantrip_api::*;
use cantrip_oracle as oracle;

/// "Creatures with power greater than the number of cards in your hand can't attack." (CR 508.1c): a restriction on every
/// creature, whoever controls it, "your hand" its controller's. The ruling: it is judged only as attackers are declared,
/// so a creature that has attacked stays attacking whatever its power or the hand become (CR 506.4a).
fn power_at_most_hand_size(q: &dyn Query, source: ObjectId, attacker: ObjectId) -> bool {
    let hand = q.state().zone(Zone::hand(q.obj(source).controller)).len() as i16;
    q.chars(attacker).power.unwrap_or(0) <= hand
}

pub static CARD: CardDef = CardDef { attack_rules: &[power_at_most_hand_size], ..oracle::ENSNARING_BRIDGE };
