use cantrip_api::*;
use cantrip_mechanics::{effects, targets::filters};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        discard(1, filters::any).and(SACRIFICE_THIS),
        |g, item| effects::draw(g, item, 1),
        "Discard a card, Sacrifice this creature: Draw a card.",
    )],
    ..oracle::MASKED_MEOWER
};
