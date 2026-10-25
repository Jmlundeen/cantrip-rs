use cantrip_api::*;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{3}{U}"), |g, item| cantrip_mechanics::effects::draw(g, item, 1), "{3}{U}: Draw a card.")],
    ..oracle::SPECTRAL_SAILOR
};
