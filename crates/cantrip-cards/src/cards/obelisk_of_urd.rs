use cantrip_api::*;
use cantrip_mechanics::chosen_type::{creatures_you_control_of_chosen_type, enters_choosing_a_creature_type};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    statics: &[static_ability(Layer::PtModify, creatures_you_control_of_chosen_type, cantrip_mechanics::statics::plus_pt::<2, 2>, "Creatures you control of the chosen type get +2/+2.")],
    ..oracle::OBELISK_OF_URD
};
