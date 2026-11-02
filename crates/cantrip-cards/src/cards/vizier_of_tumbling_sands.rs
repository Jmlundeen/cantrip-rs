use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(TAP, effects::untap_targets, "{T}: Untap another target permanent.").targets(&[targets::OTHER_PERMANENT]),
        triggers::cycling(mana("{1}{U}"), "Cycling {1}{U} ({1}{U}, Discard this card: Draw a card.)"),
        triggered(triggers::YOU_CYCLE_THIS_CARD, effects::untap_targets, "When you cycle this card, untap target permanent.")
            .targets(&[targets::PERMANENT]),
    ],
    ..oracle::VIZIER_OF_TUMBLING_SANDS
};
