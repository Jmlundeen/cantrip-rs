use cantrip_api::*;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::{effects, exile_target, targets};
use cantrip_oracle as oracle;

/// "Target card with mana value 4 or greater from your graveyard" (CR 115.1, 202.3; X is 0 in a graveyard, CR 202.3e).
fn card_with_mana_value_4_or_greater_in_your_graveyard(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    targets::card_in_your_graveyard(q, asker, t, &[]) && matches!(t, Target::Object(id) if q.mana_value(id) >= 4)
}

/// "Impound \u{2014} Exile target artifact or enchantment": nothing if the target is no longer legal (CR 608.2b).
fn impound(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    exile_target(g, item, 0);
    Ok(())
}

static MODAL: ModalDef = modes(&[
    mode(effects::return_first_target_to_hand, "Repair \u{2014} Return target card with mana value 4 or greater from your graveyard to your hand.")
        .targets(&[target(card_with_mana_value_4_or_greater_in_your_graveyard)]),
    mode(impound, "Impound \u{2014} Exile target artifact or enchantment.").targets(&[targets::ARTIFACT_OR_ENCHANTMENT]),
]);

pub static CARD: CardDef = CardDef {
    abilities: &[modal_triggered(
        THIS_ENTERS,
        &MODAL,
        "When this creature enters, choose one \u{2014} Repair \u{2014} Return target card with mana value 4 or greater from your graveyard to your hand; or Impound \u{2014} Exile target artifact or enchantment.",
    )],
    ..oracle::DAMAGE_CONTROL_CREW
};
