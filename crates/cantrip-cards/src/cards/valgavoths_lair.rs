use cantrip_api::*;
use cantrip_mechanics::lands::{chosen_color, enters_tapped_choosing_a_color};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    enters: Some(enters_tapped_choosing_a_color),
    abilities: &[mana_ability(TAP, add_any_color(1), "{T}: Add one mana of the chosen color.").colors_from(chosen_color)],
    ..oracle::VALGAVOTHS_LAIR
};
