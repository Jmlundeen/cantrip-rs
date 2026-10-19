use cantrip_api::*;
use cantrip_mechanics::chosen_type::{creatures_you_control_of_chosen_type, enters_choosing_a_creature_type};
use cantrip_mechanics::statics::plus_pt;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_choosing_a_creature_type),
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of any color.")],
    statics: &[static_ability(Layer::PtModify, creatures_you_control_of_chosen_type, plus_pt::<1, 1>, "Creatures you control of the chosen type get +1/+1.")],
    ..oracle::PATCHWORK_BANNER
};
