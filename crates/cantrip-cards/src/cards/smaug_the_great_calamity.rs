use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static SPEW_FLAME: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::damage_first_target::<5>), ..oracle::SPEW_FLAME };

pub static CARD: CardDef = CardDef { adventure: Some(&SPEW_FLAME), ..oracle::SMAUG_THE_GREAT_CALAMITY };

pub static EXTRA: &[&CardDef] = &[&SPEW_FLAME];
