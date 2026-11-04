use cantrip_api::*;
use cantrip_mechanics::{effects, rampage, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        rampage::trigger::<2>("Rampage 2"),
        triggered(
            triggers::YOUR_UPKEEP,
            |g, item| effects::sacrifice_this_unless_pay(g, item, cost("{W}{U}{B}")),
            "At the beginning of your upkeep, sacrifice Chromium unless you pay {W}{U}{B}.",
        ),
    ],
    ..oracle::CHROMIUM
};
