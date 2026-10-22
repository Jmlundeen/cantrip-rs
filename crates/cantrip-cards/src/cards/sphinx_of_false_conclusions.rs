use cantrip_api::*;
use cantrip_mechanics::copy::{create_token_copy_of_it, not_a_token};
use cantrip_mechanics::{effects::loot, triggers::{THIS_ATTACKS, THIS_DIES}};
use cantrip_oracle as oracle;

/// "Create a token that's a copy of it": no exceptions.
fn create_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    create_token_copy_of_it(g, item, |_, values| values)
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS, loot, "Whenever this creature attacks, draw a card, then discard a card."),
        triggered(THIS_DIES, create_copy, "When this creature dies, if it isn't a token, create a token that's a copy of it.")
            .condition(not_a_token),
    ],
    ..oracle::SPHINX_OF_FALSE_CONCLUSIONS
};
