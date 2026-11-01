use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{effects, targets};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<3>),
    ..oracle::LIGHTNING_BOLT
};
