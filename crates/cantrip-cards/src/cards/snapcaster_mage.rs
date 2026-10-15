use cantrip_api::*;
use cantrip_mechanics::{grant_flashback, targets, triggers::this_enters};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When this creature enters, target instant or sorcery card in your graveyard gains flashback until end of turn. The flashback cost is equal to its mana cost.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[targets::INSTANT_OR_SORCERY_IN_YOUR_GRAVEYARD],
        effect: grant_flashback,
        ..TriggeredDef::BLANK
    })],
    ..oracle::SNAPCASTER_MAGE
};
