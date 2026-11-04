use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "Target artifact creature card with mana value 3 or less from your graveyard" (CR 115.1, 109.5, 202.3).
fn artifact_creature_card(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_CARD_WITH_MANA_VALUE_3_OR_LESS_IN_YOUR_GRAVEYARD.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).is(CardType::Artifact))
}

/// "Choose one" (CR 700.2a); destroy (CR 701.8) or return the card under your control (CR 110.2a).
static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target nonartifact creature.").targets(&[targets::NONARTIFACT_CREATURE]),
    mode(effects::return_first_target_to_battlefield, "Return target artifact creature card with mana value 3 or less from your graveyard to the battlefield.")
        .targets(&[target(artifact_creature_card)]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::RESISTANCE_IS_FUTILE };
