use cantrip_api::*;
use cantrip_mechanics::copy::{create_token_copy_of_it, not_a_token};
use cantrip_mechanics::{effects::loot, triggers::{this_attacks, this_dies}};
use cantrip_oracle as oracle;

/// "Create a token that's a copy of it": no exceptions.
fn create_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_token_copy_of_it(g, item, |_, values| values)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        AbilityDef::Triggered(TriggeredDef {
            text: "Whenever this creature attacks, draw a card, then discard a card.",
            listens: EventMask::ATTACKS,
            matches: this_attacks,
            effect: loot,
            ..TriggeredDef::BLANK
        }),
        AbilityDef::Triggered(TriggeredDef {
            text: "When this creature dies, if it isn't a token, create a token that's a copy of it.",
            listens: EventMask::ZONE_CHANGE,
            matches: this_dies,
            look_back: true,
            condition: Some(not_a_token),
            effect: create_copy,
            ..TriggeredDef::BLANK
        }),
    ],
    ..oracle::SPHINX_OF_FALSE_CONCLUSIONS
};
