use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE], effects::damage_first_target::<5>), ..oracle::SCORCHING_SHOT };
