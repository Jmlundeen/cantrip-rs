use cantrip_api::*;
use cantrip_mechanics::{effects, mobilize, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        mobilize::trigger::<1>("Mobilize 1 (Whenever this creature attacks, create a tapped and attacking 1/1 red Warrior creature token. Sacrifice it at the beginning of the next end step.)"),
        triggered(
            triggers::CREATURE_YOU_CONTROL_DIES,
            effects::each_opponent_loses_1_life_you_gain_1,
            "Whenever this creature or another creature you control dies, each opponent loses 1 life and you gain 1 life.",
        ),
    ],
    delayed: &[mobilize::SACRIFICE],
    ..oracle::VENERATED_STORMSINGER
};

pub static EXTRA: &[&CardDef] = &[&mobilize::WARRIOR];
