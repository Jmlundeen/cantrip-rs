use cantrip_api::*;
use cantrip_mechanics::{fight, targets};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    spell: spell(
        &[
            targets::CREATURE_YOU_CONTROL.labelled("Creature you control to deal damage equal to its power"),
            targets::CREATURE_OR_PLANESWALKER_AN_OPPONENT_CONTROLS.labelled("Creature or planeswalker you don't control to be dealt that damage"),
        ],
        fight::first_target_bites_second,
    ),
    ..oracle::HARD_HITTING_QUESTION
};
