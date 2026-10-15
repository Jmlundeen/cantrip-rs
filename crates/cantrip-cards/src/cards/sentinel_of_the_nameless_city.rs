use cantrip_api::*;
use cantrip_mechanics::tokens::create_map;
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

fn map(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_map(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature enters, create a Map token.",
            listens: EventMask::ZONE_CHANGE,
            matches: triggers::this_enters,
            effect: map,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, create a Map token.",
            listens: EventMask::ATTACKS,
            matches: triggers::this_attacks,
            effect: map,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SENTINEL_OF_THE_NAMELESS_CITY
};
