use cantrip_api::*;
use cantrip_oracle as oracle;
use cantrip_mechanics::{effects, targets};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::SPELL], effects::counter_first_target_unless_pays::<3>),
    ..oracle::MANA_LEAK
};
