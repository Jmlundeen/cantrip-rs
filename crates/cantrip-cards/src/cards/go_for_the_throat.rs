use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONARTIFACT_CREATURE], effects::destroy_first_target),
    ..oracle::GO_FOR_THE_THROAT
};
