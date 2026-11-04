use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, triggers::another_creature_or_artifact_you_control_dies),
        |g, item| effects::draw(g, item, 1),
        "The Allagan Eye \u{2014} Whenever one or more other creatures and/or artifacts you control die, draw a card. This ability triggers only once each turn.",
    )
    .triggers_each_turn(1)
    .once_per_batch()],
    ..oracle::GRAHA_TIA
};
