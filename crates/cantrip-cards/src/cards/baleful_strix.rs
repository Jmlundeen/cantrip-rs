use cantrip_api::*;
use cantrip_mechanics::{triggers::THIS_ENTERS, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, |g, item| effects::draw(g, item, 1), "When Baleful Strix enters, draw a card.")],
    ..oracle::BALEFUL_STRIX
};
