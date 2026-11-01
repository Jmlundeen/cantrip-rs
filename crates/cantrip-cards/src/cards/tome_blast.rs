use cantrip_api::*;
use cantrip_mechanics::{effects, flashback, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<2>),
    ways: &[flashback::way(cost("{4}{R}"), "Flashback {4}{R}")],
    ..oracle::TOME_BLAST
};
