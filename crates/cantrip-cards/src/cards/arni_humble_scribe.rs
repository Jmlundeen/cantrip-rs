use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{another_nontoken_you_control_enters, CREATURE_TYPE};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(when_asked(EventMask::ZONE_CHANGE, another_nontoken_you_control_enters::<CREATURE_TYPE>), effects::untap_self, "Whenever another nontoken creature you control enters, untap Arni."),
        ability(TAP, effects::loot, "{T}: Draw a card, then discard a card."),
    ],
    ..oracle::ARNI_HUMBLE_SCRIBE
};
