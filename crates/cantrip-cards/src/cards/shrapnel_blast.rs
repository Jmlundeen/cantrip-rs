use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<5>),
    // As an additional cost to cast this spell, sacrifice an artifact (CR 118.8).
    additional_cost: Some(Cost::sacrifice(1, targets::filters::artifact)),
    ..oracle::SHRAPNEL_BLAST
};
