use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::THIS_ENTERS};
use cantrip_oracle as oracle;

/// Reach is a core keyword read from its characteristics.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        THIS_ENTERS,
        |g, item| effects::may_discard_to_draw(g, item, 1),
        "When this creature enters, you may discard a card. If you do, draw a card.",
    )],
    ..oracle::YUYAN_ARCHERS
};
