use cantrip_api::*;
use cantrip_mechanics::battlefront;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ATTACKERS_DECLARED, battlefront::you_attack),
        effects::plus_one_counter_on_each_target,
        "Battlefront — Whenever you attack, put a +1/+1 counter on each of up to X target creatures you control, where X is the number of opponents you attacked this turn.",
    )
    .targets(&[targets::CREATURE_YOU_CONTROL.between(0, TargetSpec::COUNTED)])
    .counted_by(|q, asker| battlefront::opponents_attacked_this_turn(q, asker.controller))],
    ..oracle::MARTOK_HERO_OF_THE_EMPIRE
};
