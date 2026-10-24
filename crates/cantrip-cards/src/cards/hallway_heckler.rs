use cantrip_api::*;
use cantrip_mechanics::{effects, prepare};
use cantrip_oracle as oracle;

/// "{T}, Discard a card: Draw a card." The discard is a cost (CR 602.2); the draw is the effect (CR 608.2).
pub static CARD: CardDef = CardDef {
    abilities: &[ability(
        TAP.and(discard_cards(1)),
        |g, item| effects::draw(g, item, 1),
        "{T}, Discard a card: Draw a card.",
    )],
    enters: Some(prepare::enters_prepared),
    prepare: Some(&super::stingerquill_voxmancer::VICIOUS_VERSE),
    ..oracle::HALLWAY_HECKLER
};
