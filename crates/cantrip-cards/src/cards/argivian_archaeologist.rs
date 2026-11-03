use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{W}{W}").and(TAP), effects::return_first_target_to_hand,
        "{W}{W}, {T}: Return target artifact card from your graveyard to your hand.")
        .targets(&[targets::ARTIFACT_CARD_IN_YOUR_GRAVEYARD])],
    ..oracle::ARGIVIAN_ARCHAEOLOGIST
};
