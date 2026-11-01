use cantrip_api::*;
use cantrip_mechanics::{effects, triggers};
use cantrip_oracle as oracle;

/// "Whenever a creature you control becomes the target of a spell or ability an opponent controls, you may draw a card."
/// (CR 603.3, 121.1): asked of the controller when the trigger resolves, before the spell or ability that targeted resolves.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        triggers::YOUR_CREATURE_TARGETED_BY_AN_OPPONENT,
        effects::may_draw_a_card,
        "Whenever a creature you control becomes the target of a spell or ability an opponent controls, you may draw a card.",
    )],
    ..oracle::SHAPERS_SANCTUARY
};
