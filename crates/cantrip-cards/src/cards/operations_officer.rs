use cantrip_api::*;
use cantrip_mechanics::effects::{draw, plus_one_counter_on_this};
use cantrip_mechanics::triggers::{THIS_ATTACKS, THIS_ENTERS};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| draw(g, item, 1), "When this creature enters, draw a card."),
        triggered(THIS_ATTACKS, plus_one_counter_on_this, "Whenever this creature attacks, put a +1/+1 counter on it."),
    ],
    ..oracle::OPERATIONS_OFFICER
};
