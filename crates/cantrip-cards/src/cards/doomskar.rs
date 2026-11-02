use cantrip_api::*;
use cantrip_mechanics::{effects::destroy_all_creatures, foretell};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[], destroy_all_creatures),
    abilities: &[foretell::ABILITY],
    foretell: Some(mana("{1}{W}{W}")),
    ..oracle::DOOMSKAR
};
