use cantrip_api::*;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

// Flying is the core's, read from the card's JSON.
pub static CARD: CardDef = CardDef { cost_changes: &[cost_reduction::second_spell_you_cast_each_turn_costs_less::<1>], ..oracle::HIGHSPIRE_BELL_RINGER };
