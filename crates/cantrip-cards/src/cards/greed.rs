use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{B}").and(life(2)), |g, item| effects::draw(g, item, 1), "{B}, Pay 2 life: Draw a card.")],
    ..oracle::GREED
};
