use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Choose one — • Destroy target blue permanent. • Return target Island to its owner's hand." (CR 115.1).
fn choose_one_destroy_target_blue_mode_1_target_0(
    q: &dyn Query,
    asker: &Asker,
    t: Target,
) -> bool {
    targets::permanent_of_any_subtype(q, asker, t, &[Subtype::Island])
}

static ABILITY_0_MODES: ModalDef = modes(
        &[
            mode(effects::destroy_first_target, "Destroy target blue permanent.")
                .targets(&[targets::permanent_of_color::<{ Color::Blue as u8 }>()]),
            mode(
                    effects::return_first_target_to_hand,
                    "Return target Island to its owner's hand.",
                )
                .targets(&[target(choose_one_destroy_target_blue_mode_1_target_0)]),
        ],
    )
    .choose(1, 1);

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef::MODAL),
    modal: Some(&ABILITY_0_MODES),
    ..oracle::ACTIVE_VOLCANO
};
