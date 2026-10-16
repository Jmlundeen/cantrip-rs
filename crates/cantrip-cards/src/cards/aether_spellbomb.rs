use cantrip_api::*;
use cantrip_mechanics::{targets, effects};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{U}").and(SACRIFICE_THIS), effects::return_first_target_to_hand, "{U}, Sacrifice Aether Spellbomb: Return target creature to its owner's hand.")
            .targets(&[targets::CREATURE]),
        ability(cost("{1}").and(SACRIFICE_THIS), |g, item| effects::draw(g, item, 1), "{1}, Sacrifice Aether Spellbomb: Draw a card."),
    ],
    ..oracle::AETHER_SPELLBOMB
};
