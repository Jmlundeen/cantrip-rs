use cantrip_api::*;
use cantrip_mechanics::{increment, mana};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[increment::TRIGGER, mana_ability(TAP, add_by(mana::add_green_equal_to_its_power), "{T}: Add an amount of {G} equal to this creature's power.")],
    ..oracle::TOPIARY_LECTURER
};
