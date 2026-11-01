use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, effects, lands, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<4>),
    own_cost_changes: &[cost_reduction::less_if_you_control_subtype::<2, { lands::sub(Subtype::Vehicle) }>],
    ..oracle::TRUCK_TOSS
};
