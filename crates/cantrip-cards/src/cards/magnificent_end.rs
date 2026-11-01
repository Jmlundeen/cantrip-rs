use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::damage_first_target::<5>),
    own_cost_changes: &[targets::costs_less_if_it_targets_a_tapped_creature::<3>],
    ..oracle::MAGNIFICENT_END
};
