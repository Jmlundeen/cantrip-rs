use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one — • Counter target red spell. • Destroy target red permanent." (CR 608.2, 701.6a).
fn choose_one_counter_target_red_mode_0(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    cantrip_mechanics::counter_target(g, item, 0);
    Ok(())
}

static ABILITY_0_MODES: ModalDef = modes(
        &[
            mode(choose_one_counter_target_red_mode_0, "Counter target red spell.")
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
