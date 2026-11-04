use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// "Target Spacecraft card in your graveyard" (CR 115.1, 400.7): a card in the graveyard of the ability's controller whose
/// types include the Spacecraft artifact type, which no text-changing effect can change (CR 612.2 changes a creature type,
/// a basic land type or a color word, not an artifact type).
fn spacecraft_card_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[])
        && matches!(t, Target::Object(id) if q.chars(id).has_subtype(Subtype::Spacecraft))
}

/// "Choose one —" (CR 700.2): one mode, its own target if it has one (CR 700.2c), in printed order (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(effects::return_first_target_to_hand, "Return target Spacecraft card from your graveyard to your hand.")
        .targets(&[target(spacecraft_card_in_your_graveyard)]),
    mode(|g, item| effects::surveil(g, item, 2), "Surveil 2."),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one —\n\u{2022} Return target Spacecraft card from your graveyard to your hand.\n\u{2022} Surveil 2.",
    )],
    ..oracle::EMERGENCY_HOLOGRAPHIC_CREW
};
