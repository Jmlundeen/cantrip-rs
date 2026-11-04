use cantrip_api::*;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef { cost_changes: &[cost_reduction::second_spell_you_cast_each_turn_costs_less::<2>], ..oracle::UTHROS_PSIONICIST };
