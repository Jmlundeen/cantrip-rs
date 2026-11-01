use cantrip_api::*;
use cantrip_mechanics::{effects, harmonize, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<2>),
    ways: &[harmonize::way(mana("{5}{R}{R}"), "Harmonize {5}{R}{R}")],
    ..oracle::CHANNELED_DRAGONFIRE
};
