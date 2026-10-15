use cantrip_api::*;
use cantrip_mechanics::library::reveal_top_to_hand;
use cantrip_oracle as oracle;

/// Reveal the top card of your library and put it into your hand (a move, not a draw), then lose life equal to its
/// mana value, read as it was revealed.
fn reveal(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some((_, value)) = reveal_top_to_hand(g, item.controller) {
        g.lose_life(item.controller, value as i32);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "At the beginning of your upkeep, reveal the top card of your library and put that card into your hand. You lose life equal to its mana value.",
        listens: EventMask::STEP_BEGAN,
        matches: cantrip_mechanics::triggers::your_upkeep,
        effect: reveal,
        ..TriggeredDef::BLANK
    })],
    ..oracle::DARK_CONFIDANT
};
