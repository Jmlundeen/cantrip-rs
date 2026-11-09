use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::BLOCKING_CREATURE], effects::target_creature_gets::<7, 7>),
    ..oracle::RIGHTEOUSNESS
};
