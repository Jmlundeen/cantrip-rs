use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::spell_of_either_color::<{ Color::Red as u8 }, { Color::Green as u8 }>()], effects::counter_first_target),
    ..oracle::FLASHFREEZE
};
