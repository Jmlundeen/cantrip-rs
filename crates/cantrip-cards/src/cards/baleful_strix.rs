use cantrip_api::*;
use cantrip_mechanics::{triggers::this_enters, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Baleful Strix enters, draw a card.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect: effects::draw::<1>,
        ..TriggeredDef::BLANK
    })],
    ..oracle::BALEFUL_STRIX
};
