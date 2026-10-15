use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_mechanics::triggers::{self, this_enters};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    kicker: Some(Kicker { cost: mana(2, 0, 0, 0, 0, 0), multi: true, waterbend: false }),
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Benediction of the Omnissiah \u{2014} When this creature enters, draw a card.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: effects::draw::<1>,
            ..TriggeredDef::BLANK
        }),
        triggers::SQUAD,
    ],
    ..oracle::SICARIAN_INFILTRATOR
};
