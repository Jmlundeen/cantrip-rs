use cantrip_api::*;
use cantrip_mechanics::triggers::{THIS_DIES, YOU_DRAW_YOUR_SECOND_CARD};
use cantrip_mechanics::{effects, tokens};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
            YOU_DRAW_YOUR_SECOND_CARD,
            effects::plus_one_counter_on_this,
            "Whenever you draw your second card each turn, put a +1/+1 counter on this creature.",
        ),
        triggered(
            THIS_DIES,
            |g, item| tokens::create(g, item.controller, &tokens::CLUE, 1).map(|_| ()),
            "When this creature dies, create a Clue token. (It's an artifact with \"{2}, Sacrifice this token: Draw a card.\")",
        ),
    ],
    ..oracle::KNOWLEDGE_SEEKER
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
