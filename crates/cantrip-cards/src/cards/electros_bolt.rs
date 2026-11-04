use cantrip_api::*;
use cantrip_mechanics::{effects, mayhem, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::damage_first_target::<4>),
    ways: &[mayhem::way(mana("{1}{R}"), "Mayhem {1}{R}")],
    ..oracle::ELECTROS_BOLT
};
