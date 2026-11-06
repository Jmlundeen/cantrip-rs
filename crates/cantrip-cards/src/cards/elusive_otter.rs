use cantrip_api::*;
use cantrip_mechanics::combat::power_less_than_its_cant_block_it;
use cantrip_mechanics::{effects, targets, triggers::PROWESS};
use cantrip_oracle as oracle;

/// The Adventure (CR 715): Grove's Bounty.
pub static GROVES_BOUNTY: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_CONTROL.between(0, TargetSpec::COUNTED)], effects::distribute_plus_one_counters),
    divide: Some(Divide { total: TargetSpec::COUNTED, reads: None }),
    ..oracle::GROVES_BOUNTY
};

pub static CARD: CardDef = CardDef {
    abilities: &[PROWESS],
    block_rules: &[power_less_than_its_cant_block_it],
    adventure: Some(&GROVES_BOUNTY),
    ..oracle::ELUSIVE_OTTER
};

pub static EXTRA: &[&CardDef] = &[&GROVES_BOUNTY];
