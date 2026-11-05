use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// "{1}, Sacrifice a token: Draw a card." The token is sacrificed as a cost (CR 118.3, 701.21a), so it is gone before the
/// ability can be responded to.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        cost("{1}").and(sacrifice(1, targets::token_you_control)),
        |g, item| effects::draw(g, item, 1),
        "{1}, Sacrifice a token: Draw a card.",
    )],
    ..oracle::HARDENED_TACTICIAN
};
