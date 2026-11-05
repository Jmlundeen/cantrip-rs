use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(targets::creature_with_mana_value_at_most::<3>)], effects::exile_first_target),
    ..oracle::DEATH_IN_THE_FAMILY
};
