use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{4}").and(TAP), |g, item| effects::draw(g, item, 1), "{4}, {T}: Draw a card.")],
    ..oracle::JAYEMDAE_TOME
};
