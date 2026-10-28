use cantrip_api::*;
use cantrip_mechanics::triggers::PLUS_ONE_COUNTERS_PUT_ON_THIS;
use cantrip_mechanics::{effects, increment};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        increment::TRIGGER,
        triggered(PLUS_ONE_COUNTERS_PUT_ON_THIS, |g, item| effects::draw(g, item, 1), "Whenever one or more +1/+1 counters are put on this creature, draw a card."),
    ],
    ..oracle::PENSIVE_PROFESSOR
};
