use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOU_DISCARD,
        effects::this_gets::<2, 0>,
        "Whenever you discard a card, Hobgoblin gets +2/+0 until end of turn.",
    )],
    ..oracle::HOBGOBLIN_MANTLED_MARAUDER
};
