use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::creature_of_either_color::<{ Color::White as u8 }, { Color::Blue as u8 }>()], effects::damage_first_target::<4>),
    ..oracle::RENDING_VOLLEY
};
