use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<3>),
    own_cost_changes: &[targets::less_if_it_targets_color::<2, { Color::Blue as u8 }>],
    ..oracle::MYSTICAL_DISPUTE
};
