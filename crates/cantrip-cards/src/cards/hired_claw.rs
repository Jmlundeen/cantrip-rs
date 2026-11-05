use cantrip_api::*;
use cantrip_mechanics::effects::{self, plus_one_counter_on_this};
use cantrip_mechanics::{history::an_opponent_lost_life, lands, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ATTACKS, triggers::you_attack_with_a_creature_of_subtype::<{ lands::sub(Subtype::Lizard) }>), effects::damage_first_target::<1>, "Whenever you attack with one or more Lizards, this creature deals 1 damage to target opponent.")
            .targets(&[targets::OPPONENT])
            .once_per_batch(),
        ability(cost("{1}{R}"), plus_one_counter_on_this, "{1}{R}: Put a +1/+1 counter on this creature. Activate only if an opponent lost life this turn and only once each turn.")
            .condition(an_opponent_lost_life)
            .once_each_turn(),
    ],
    ..oracle::HIRED_CLAW
};
