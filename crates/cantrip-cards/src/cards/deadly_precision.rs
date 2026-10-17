use cantrip_api::*;
use cantrip_mechanics::{effects, targets, PAY_FOUR_OR_SACRIFICE_AN_ARTIFACT_OR_CREATURE};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    // Destroy target creature.
    spell: spell(&[targets::CREATURE], effects::destroy_first_target),
    // As an additional cost to cast this spell, pay {4} or sacrifice an artifact or creature (CR 601.2b).
    additional_choice: &PAY_FOUR_OR_SACRIFICE_AN_ARTIFACT_OR_CREATURE,
    ..oracle::DEADLY_PRECISION
};
