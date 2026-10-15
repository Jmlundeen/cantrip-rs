use cantrip_api::*;
use cantrip_mechanics::tokens::{self, investigate};
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    investigate(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Thraben Inspector enters, investigate.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        effect,
        ..TriggeredDef::BLANK
    })],
    ..oracle::THRABEN_INSPECTOR
};

pub static EXTRA: &[&CardDef] = &[&tokens::CLUE];
