use cantrip_api::*;
use cantrip_mechanics::{behold, effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<5>),
    additional_cost: Some(sacrifice(1, behold::goblin)),
    ..oracle::GOBLIN_GRENADE
};
