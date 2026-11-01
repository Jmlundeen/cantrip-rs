use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "This spell costs {2} less to cast if it targets an attacking creature." "Return target creature to its owner's
/// hand. Surveil 2."
pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::return_first_target_to_hand_then_surveil::<2>),
    own_cost_changes: &[targets::costs_less_if_it_targets_an_attacking_creature::<2>],
    ..oracle::EPHARAS_DISPERSAL
};
