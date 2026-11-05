use cantrip_api::*;
use cantrip_mechanics::{cost_reduction, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    own_cost_changes: &[cost_reduction::less_if_you_cast_another_spell_this_turn::<2>],
    spell: spell(&[], effects::draw_three_then_discard),
    ..oracle::FOCUS_THE_MIND
};
