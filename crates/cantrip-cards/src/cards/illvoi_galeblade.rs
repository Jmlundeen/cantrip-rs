use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// Flash and flying are core keywords read from its characteristics. "{2}, Sacrifice this creature: Draw a card." (CR 602.2,
/// 701.21): sacrificing it is part of the cost, so the ability is on the stack, and resolves, without it.
pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{2}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{2}, Sacrifice this creature: Draw a card.")],
    ..oracle::ILLVOI_GALEBLADE
};
