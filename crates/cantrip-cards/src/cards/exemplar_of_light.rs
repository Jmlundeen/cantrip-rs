use cantrip_api::*;
use cantrip_mechanics::effects::{draw, plus_one_counter_on_this};
use cantrip_mechanics::triggers;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::YOU_GAIN_LIFE, plus_one_counter_on_this, "Whenever you gain life, put a +1/+1 counter on this creature."),
        triggered(triggers::YOU_PUT_PLUS_ONE_COUNTERS_ON_THIS, |g, item| draw(g, item, 1), "Whenever you put one or more +1/+1 counters on this creature, draw a card. This ability triggers only once each turn.")
            .triggers_each_turn(1),
    ],
    ..oracle::EXEMPLAR_OF_LIGHT
};
