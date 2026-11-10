use cantrip_api::*;
use cantrip_mechanics::{regenerate, spore};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        spore::UPKEEP,
        ability(
            remove_counters(CounterKind::Spore, 3),
            regenerate::this_effect,
            "Remove three spore counters from this creature: Regenerate this creature.",
        ),
    ],
    ..oracle::FERAL_THALLID
};
