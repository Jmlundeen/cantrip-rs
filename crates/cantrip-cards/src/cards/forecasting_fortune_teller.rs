use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| tokens::create(g, item.controller, &tokens::CLUE, 1).map(|_| ()),
        "When this creature enters, create a Clue token. (It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
    )],
    ..oracle::FORECASTING_FORTUNE_TELLER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
