use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| {
            effects::gain_life(g, item, 1)?;
            effects::draw(g, item, 1)
        },
        "When this creature enters, you gain 1 life and draw a card.",
    )],
    ..oracle::INSPIRING_OVERSEER
};
