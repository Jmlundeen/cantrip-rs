use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "All creatures able to block enchanted creature do so." (CR 509.1c): a requirement on every creature that the
/// restrictions let block the creature this enchants - one able to block it, which is the core's to judge (CR 509.1b: a
/// tapped creature, one that can't block, or one its evasion keeps out is not able, the rulings). It gives no creature the
/// ability to block it (the ruling). The defending player obeys as many such requirements as any legal declaration could:
/// with menace, two creatures able to block it both must, and one alone need not.
fn able_blockers_block_it(q: &dyn Query, source: ObjectId, _: ObjectId, attacker: ObjectId) -> bool {
    q.attached_to(source) == Some(attacker)
}

pub static CARD: CardDef = CardDef {
    enchant: Some(targets::CREATURE),
    block_requirements: &[able_blockers_block_it],
    ..oracle::LURE
};
