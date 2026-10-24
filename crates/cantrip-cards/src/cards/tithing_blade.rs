use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_ENTERS, YOUR_UPKEEP};
use cantrip_mechanics::{craft, effects};
use cantrip_oracle as oracle;

pub static BACK: CardDef = CardDef {
    abilities: &[triggered(YOUR_UPKEEP, cantrip_mechanics::effects::each_opponent_loses_1_life_you_gain_1, "At the beginning of your upkeep, each opponent loses 1 life and you gain 1 life.")],
    ..oracle::CONSUMING_SEPULCHER
};

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, effects::each_opponent_sacrifices_a_creature, "When this artifact enters, each opponent sacrifices a creature of their choice."),
        craft::ability(mana("{4}{B}"), &craft::CREATURE, "Craft with creature {4}{B}"),
    ],
    back: Some(&BACK),
    ..oracle::TITHING_BLADE
};
