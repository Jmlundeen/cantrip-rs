use cantrip_api::*;
use cantrip_mechanics::cost_reduction;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    cost_changes: &[cost_reduction::spells_you_cast_cost_1_less],
    ..oracle::STONE_CALENDAR
};
