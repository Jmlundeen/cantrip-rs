use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

static ABILITY_0_MODES: ModalDef = modes(
        &[
            mode(effects::counter_first_target, "Counter target red spell.")
                .targets(&[targets::spell_of_either_color::<{ Color::Red as u8 }, { Color::Red as u8 }>()]),
            mode(effects::destroy_first_target, "Destroy target red permanent.")
                .targets(&[targets::permanent_of_color::<{ Color::Red as u8 }>()]),
        ],
    )
    .choose(1, 1);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&ABILITY_0_MODES),
    ..oracle::BLUE_ELEMENTAL_BLAST
};
