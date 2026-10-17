use cantrip_api::*;
use cantrip_mechanics::{effects, persist};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, |g, item| effects::gain_life(g, item, 2), "When this creature enters, you gain 2 life."),
        persist::trigger("Persist (When this creature dies, if it had no -1/-1 counters on it, return it to the battlefield under its owner's control with a -1/-1 counter on it.)"),
    ],
    ..oracle::KITCHEN_FINKS
};
