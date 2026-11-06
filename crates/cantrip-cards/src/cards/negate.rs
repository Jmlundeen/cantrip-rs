use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::NONCREATURE_SPELL], effects::counter_first_target),
    ..oracle::NEGATE
};
