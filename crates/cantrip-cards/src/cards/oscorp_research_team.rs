use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[ability(cost("{6}{U}"), |g, item| effects::draw(g, item, 2), "{6}{U}: Draw two cards.")],
    ..oracle::OSCORP_RESEARCH_TEAM
};
