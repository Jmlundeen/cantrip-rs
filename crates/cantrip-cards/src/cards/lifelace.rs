use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL_OR_PERMANENT], effects::target_becomes_color::<{ Color::Green as u8 }>),
    ..oracle::LIFELACE
};
