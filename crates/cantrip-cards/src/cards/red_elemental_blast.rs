use cantrip_api::*;
use cantrip_mechanics::{counter_target, effects, targets};
use cantrip_oracle as oracle;

/// "Choose one — • Counter target blue spell. • Destroy target blue permanent." (CR 608.2, 701.6a).
fn choose_one_counter_target_blue_mode_0(
    g: &mut dyn Game,
    item: &StackItem,
) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

/// "Choose one — • Counter target blue spell. • Destroy target blue permanent." (CR 115.1).
fn choose_one_counter_target_blue_mode_1_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    matches!(
        t, Target::Object(id) if cantrip_mechanics::targets::permanent(q, id) && (q
        .chars(id).colors.has(q.words(asker.source).color(Color::Blue)))
    )
}

static ABILITY_0_MODES: ModalDef = modes(
        &[
            mode(choose_one_counter_target_blue_mode_0, "Counter target blue spell.")
                .targets(&[targets::spell_of_either_color::<{ Color::Blue as u8 }, { Color::Blue as u8 }>()]),
            mode(effects::destroy_first_target, "Destroy target blue permanent.")
                .targets(&[target(choose_one_counter_target_blue_mode_1_target_0)]),
        ],
    );

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&ABILITY_0_MODES),
    ..oracle::RED_ELEMENTAL_BLAST
};
