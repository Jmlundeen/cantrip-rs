use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{B}{B}"), effects::counter_first_target, "{B}{B}: Counter target green spell.")
        .targets(&[targets::spell_of_either_color::<{ Color::Green as u8 }, { Color::Green as u8 }>()])],
    ..oracle::DEATHGRIP
};
