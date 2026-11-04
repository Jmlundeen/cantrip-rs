use cantrip_api::*;
use cantrip_mechanics::{effects, targets::filters};
use cantrip_oracle as oracle;

/// Flying and deathtouch are core keywords. "{3}, Sacrifice another creature or artifact: Draw a card" (CR 118.3, 701.21): the
/// sacrifice is a cost, paid as the ability is activated, so no response can stop the draw by removing what was sacrificed.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{3}").and(sacrifice(1, filters::another_creature_or_artifact)),
        |g, item| effects::draw(g, item, 1),
        "{3}, Sacrifice another creature or artifact: Draw a card.",
    )],
    ..oracle::AHRIMAN
};
