use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, effects::first_target_cant_block_this_turn, "When this creature enters, target creature can't block this turn.").targets(&[targets::CREATURE])],
    ..oracle::SUMMIT_INTIMIDATOR
};
