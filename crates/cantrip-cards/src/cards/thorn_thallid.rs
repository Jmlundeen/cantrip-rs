use cantrip_api::*;
use cantrip_mechanics::{effects, spore, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        spore::UPKEEP,
        ability(remove_counters(CounterKind::Spore, 3), effects::damage_first_target::<1>,
            "Remove three spore counters from this creature: It deals 1 damage to any target.")
            .targets(&[targets::ANY]),
    ],
    ..oracle::THORN_THALLID
};
