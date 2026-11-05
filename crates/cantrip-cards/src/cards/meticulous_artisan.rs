use cantrip_api::*;
use cantrip_mechanics::{tokens, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggers::PROWESS,
        triggered(triggers::THIS_ENTERS, tokens::create_a_treasure, "When this creature enters, create a Treasure token."),
    ],
    ..oracle::METICULOUS_ARTISAN
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
