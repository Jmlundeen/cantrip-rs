use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one — • Counter target blue spell. • Destroy target blue permanent." (CR 608.2, 701.6a).
static ABILITY_0_MODES: ModalDef = modes(
        &[
            mode(effects::counter_first_target, "Counter target blue spell.")
                .targets(&[targets::spell_of_either_color::<{ Color::Blue as u8 }, { Color::Blue as u8 }>()]),
            mode(effects::destroy_first_target, "Destroy target blue permanent.")
                .targets(&[targets::permanent_of_color::<{ Color::Blue as u8 }>()]),
        ],
    );

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&ABILITY_0_MODES),
    ..oracle::RED_ELEMENTAL_BLAST
};
