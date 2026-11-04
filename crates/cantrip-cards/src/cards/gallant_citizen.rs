use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When this creature enters, draw a card.")],
    ..oracle::GALLANT_CITIZEN
};
