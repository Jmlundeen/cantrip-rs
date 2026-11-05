use cantrip_api::*;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::THIS_ENTERS_OR_DIES;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS_OR_DIES, tokens::create_a_treasure, "When this creature enters or dies, create a Treasure token."),
    ],
    ..oracle::NOGGLE_ROBBER
};

pub static EXTRA: &[&CardDef] = &[&tokens::TREASURE];
