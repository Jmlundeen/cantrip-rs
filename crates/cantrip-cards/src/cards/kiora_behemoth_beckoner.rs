use cantrip_api::*;
use cantrip_mechanics::{effects, targets, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(triggers::CREATURE_WITH_POWER_FOUR_OR_MORE_YOU_CONTROL_ENTERS, |g, item| effects::draw(g, item, 1), "Whenever a creature you control with power 4 or greater enters, draw a card."),
        ability(loyalty(-1), effects::untap_targets, "\u{2212}1: Untap target permanent.").targets(&[targets::PERMANENT]),
    ],
    ..oracle::KIORA_BEHEMOTH_BECKONER
};
