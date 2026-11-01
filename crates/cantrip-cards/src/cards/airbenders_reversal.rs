use cantrip_api::*;
use cantrip_mechanics::{airbend_targets, effects, targets};
use cantrip_oracle as oracle;

static MODAL: ModalDef = modes(&[
    mode(effects::destroy_first_target, "Destroy target attacking creature.")
        .targets(&[targets::ATTACKING_CREATURE]),
    mode(airbend_targets, "Airbend target creature you control.")
        .targets(&[targets::CREATURE_YOU_CONTROL]),
]);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::AIRBENDERS_REVERSAL };
