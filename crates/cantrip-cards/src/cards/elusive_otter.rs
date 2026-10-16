use cantrip_api::*;
use cantrip_mechanics::{counters_divided, targets, triggers::PROWESS};
use cantrip_oracle as oracle;

/// "Creatures with power less than this creature's power can't block it" (CR 509.1b): of a possible block with the Otter
/// as the attacker, the blocker's power must be at least the Otter's, both as they are as blockers are declared. A
/// block that has been made stands if powers change afterwards, since this is only asked when blocks are declared.
fn power_less_than_it_cant_block(q: &dyn Query, source: ObjectId, blocker: ObjectId, attacker: ObjectId) -> bool {
    source != attacker || q.chars(blocker).power.unwrap_or(0) >= q.chars(source).power.unwrap_or(0)
}

/// Distribute X +1/+1 counters among any number of target creatures you control (CR 601.2d): each target got its share
/// as the spell was cast, and one that has since become illegal is left out with its share (CR 608.2b).
fn grove_s_bounty(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counters_divided(g, item, CounterKind::PlusOne);
    Ok(())
}

/// The Adventure (CR 715): Grove's Bounty.
pub static GROVES_BOUNTY: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL.between(0, TargetSpec::COUNTED)], grove_s_bounty),
    divide: Some(Divide { total: TargetSpec::COUNTED, of_x: None }),
    ..oracle::GROVES_BOUNTY
};

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    block_rules: &[power_less_than_it_cant_block],
    adventure: Some(&GROVES_BOUNTY),
    ..oracle::ELUSIVE_OTTER
};

pub static EXTRA: &[&CardDef] = &[&GROVES_BOUNTY];
