use cantrip_api::*;
use cantrip_mechanics::{effects, harmonize, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::CREATURE], effects::target_creature_gets_and_gains_haste::<3, 0>),
    ways: &[harmonize::way(mana("{4}{R}"), "Harmonize {4}{R}")],
    ..oracle::WILD_RIDE
};
