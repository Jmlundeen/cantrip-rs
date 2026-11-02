use cantrip_api::*;
use cantrip_mechanics::{effects, impending, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        impending::REMOVE_A_TIME_COUNTER,
        triggered(
            triggers::THIS_ENTERS_OR_ATTACKS,
            effects::damage_first_target::<4>,
            "Whenever this permanent enters or attacks, it deals 4 damage to any target.",
        )
        .targets(&[targets::ANY]),
    ],
    statics: &[impending::NOT_A_CREATURE],
    enters: Some(impending::enters::<4>),
    ways: &[impending::way(mana("{2}{R}{R}"), "Impending 4\u{2014}{2}{R}{R}")],
    ..oracle::OVERLORD_OF_THE_BOILERBILGES
};
