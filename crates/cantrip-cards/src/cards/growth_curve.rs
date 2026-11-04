use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { spell: spell(&[targets::CREATURE_YOU_CONTROL], effects::plus_one_counter_on_target_then_double), ..oracle::GROWTH_CURVE };
