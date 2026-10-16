use cantrip_api::*;
use cantrip_mechanics::{crew::crew, discard_chosen, effects::ask_may, triggers::THIS_ATTACKS_OR_BLOCKS};
use cantrip_oracle as oracle;

const ASK: Question = Question::new("Draw", "draw a card?", "If you do, discard a card", "Draw and discard", "Don't draw");

/// "You may draw a card. If you do, discard a card" (CR 608.2c): "if you do" is whether a card was drawn, not whether
/// the player said yes, so a draw that a replacement took or an empty library made discards nothing.
fn loot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    if !ask_may(g, player, item.object, Purpose::Ask(&ASK))? {
        return Ok(());
    }
    let before = g.history().drawn(player);
    g.draw(player, 1);
    if g.history().drawn(player) > before {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(THIS_ATTACKS_OR_BLOCKS, loot, "Whenever this Vehicle attacks or blocks, you may draw a card. If you do, discard a card."),
        crew(1, "Crew 1 (Tap any number of creatures you control with total power 1 or more: This Vehicle becomes an artifact creature until end of turn.)"),
    ],
    ..oracle::SMUGGLERS_COPTER
};
