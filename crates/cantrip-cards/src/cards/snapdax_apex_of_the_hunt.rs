use cantrip_api::*;
use cantrip_mechanics::{effects, mutate, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            mutate::THIS_MUTATES,
            |g, item| effects::damage_target_and_gain_life(g, item, 4, 4),
            "Whenever this creature mutates, it deals 4 damage to target creature or planeswalker an opponent controls and you gain 4 life.",
        )
        .targets(&[targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS]),
    ],
    ways: &[mutate::way(mana("{2}{B/R}{W}{W}"), "Mutate {2}{B/R}{W}{W}")],
    ..oracle::SNAPDAX_APEX_OF_THE_HUNT
};
