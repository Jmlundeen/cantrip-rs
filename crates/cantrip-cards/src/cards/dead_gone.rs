use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static DEAD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::damage_first_target::<2>), ..oracle::DEAD };

/// Gone: "Return target creature you don't control to its owner's hand."
pub static GONE: CardDef = CardDef {
    spell: spell(&[targets::CREATURE_YOU_DONT_CONTROL], effects::return_first_target_to_hand),
    ..oracle::GONE
};

pub static CARD: CardDef = CardDef { split: Some((&DEAD, &GONE)), ..oracle::DEAD_GONE };

pub static EXTRA: &[&CardDef] = &[&DEAD, &GONE];
