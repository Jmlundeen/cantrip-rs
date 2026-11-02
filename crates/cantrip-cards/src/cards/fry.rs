use cantrip_api::*;
use cantrip_mechanics::{effects::damage_first_target, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[targets::creature_or_planeswalker_of_either_color::<{ Color::White as u8 }, { Color::Blue as u8 }>()],
        damage_first_target::<5>,
    ),
    ..oracle::FRY
};
