use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        effects::gain_life_then_draw::<1, 1>,
        "When this creature enters, you gain 1 life and draw a card.",
    )],
    ..oracle::INSPIRING_OVERSEER
};
