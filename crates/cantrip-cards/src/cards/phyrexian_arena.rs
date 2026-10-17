use cantrip_api::*;
use cantrip_mechanics::{effects, triggers::YOUR_UPKEEP};
use cantrip_oracle as oracle;

/// "At the beginning of your upkeep, you draw a card and lose 1 life" (CR 503.1a, 121.1, 119.3): each Arena is its own
/// trigger, and the controller (who is the active player) draws and then loses the life.
pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        YOUR_UPKEEP,
        effects::draw_a_card_and_lose_1_life,
        "At the beginning of your upkeep, you draw a card and lose 1 life.",
    )],
    ..oracle::PHYREXIAN_ARENA
};
