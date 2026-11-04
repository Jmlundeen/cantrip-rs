use cantrip_api::*;
use cantrip_mechanics::{spore, tokens};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[spore::UPKEEP, spore::SAPROLING],
    ..oracle::THALLID
};

pub static EXTRA: &[&CardDef] = &[&tokens::SAPROLING];
