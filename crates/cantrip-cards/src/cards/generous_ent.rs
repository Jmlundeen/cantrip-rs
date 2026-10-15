use cantrip_api::*;
use cantrip_mechanics::typecycling;
use cantrip_mechanics::tokens;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// When this creature enters, create a Food token.
fn make_food(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    tokens::create_food(g, item.controller)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature enters, create a Food token.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_enters,
            effect: make_food,
            ..TriggeredDef::BLANK
        }),
        typecycling::ability(Subtype::Forest, mana(1, 0, 0, 0, 0, 0), "Forestcycling {1} ({1}, Discard this card: Search your library for a Forest card, reveal it, put it into your hand, then shuffle.)"),
    ],
    ..oracle::GENEROUS_ENT
};

pub static EXTRA: &[&CardDef] = &[&tokens::FOOD];
