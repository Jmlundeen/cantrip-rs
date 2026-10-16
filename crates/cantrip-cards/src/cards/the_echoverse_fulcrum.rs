use cantrip_api::*;
use cantrip_mechanics::effects::{destroy_all_creatures, loot};
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_oracle as oracle;

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ENTERS, loot, "When The Echoverse Fulcrum enters, draw a card, then discard a card."),
        ability(
            cost("{5}, {T}").and(EXILE_THIS),
            destroy_all_creatures,
            "{5}, {T}, Exile The Echoverse Fulcrum: Destroy all creatures. Activate only as a sorcery.",
        )
        .sorcery_speed(),
    ],
    ..oracle::THE_ECHOVERSE_FULCRUM
};
