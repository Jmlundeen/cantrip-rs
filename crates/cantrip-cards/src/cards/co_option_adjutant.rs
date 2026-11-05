use cantrip_api::*;
use cantrip_mechanics::{effects, statics, triggers};
use cantrip_oracle as oracle;

/// "Artifact creatures you control have [the trigger]." (CR 611.3a, 613.1f).
/// Each creature, including this one, has its own ability controlled by its controller (CR 603.3a).
/// "Whenever this creature deals combat damage to a player, draw a card." (CR 603.2, 121.2).
pub static CARD: CardDef = CardDef {
    delayed: &[triggered(
        triggers::DEALS_COMBAT_DAMAGE_TO_PLAYER,
        |g, item| effects::draw(g, item, 1),
        "Whenever this creature deals combat damage to a player, draw a card.",
    )],
    statics: &[static_ability(
        Layer::Ability,
        statics::your_artifact_creatures,
        statics::grant_delayed_ability,
        "Artifact creatures you control have \"Whenever this creature deals combat damage to a player, draw a card.\"",
    )],
    ..oracle::CO_OPTION_ADJUTANT
};
