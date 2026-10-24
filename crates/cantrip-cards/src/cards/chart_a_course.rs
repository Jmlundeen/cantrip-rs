use cantrip_api::*;
use cantrip_mechanics::{discard_chosen, history::creatures_attacked_with};
use cantrip_oracle as oracle;

/// "Draw two cards. Then discard a card unless you attacked this turn." You attacked if you declared a creature as an attacker (CR 508.1).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 2);
    if creatures_attacked_with(g, player) == 0 {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::CHART_A_COURSE };
