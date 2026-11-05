use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::ARTIFACT_YOU_CONTROL_ENTERS, effects::plus_one_counter_on_this, "Whenever an artifact you control enters, put a +1/+1 counter on Tidus."),
        triggered(triggers::THIS_ATTACKS, effects::tap_targets, "Whenever Tidus attacks, tap target creature an opponent controls.")
            .targets(&[targets::CREATURE_AN_OPPONENT_CONTROLS]),
    ],
    ..oracle::TIDUS_BLITZBALL_STAR
};
