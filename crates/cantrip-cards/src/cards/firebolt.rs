use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;
use cantrip_mechanics::{effects, targets};

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::ANY], effects::damage_first_target::<2>),
    ways: &[flashback::way(cost("{4}{R}"), "Flashback {4}{R}")],
    ..oracle::FIREBOLT
};
