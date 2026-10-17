use cantrip_api::*;
use cantrip_mechanics::triggers::LANDFALL;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(LANDFALL, cantrip_mechanics::effects::add_one_mana_of_any_color, "Landfall \u{2014} Whenever a land you control enters, add one mana of any color.")],
    ..oracle::LOTUS_COBRA
};
