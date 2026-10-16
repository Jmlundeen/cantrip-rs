use cantrip_api::*;
use cantrip_mechanics::discard_chosen;
use cantrip_oracle as oracle;

/// Whether `player` attacked this turn (CR 508.1): some creature was declared as an attacker in a combat of this turn.
/// Attackers are declared by the active player, so the turn's `Attacks` records under `player`'s turn are theirs; a
/// creature put onto the battlefield attacking was never declared and records none.
fn attacked_this_turn(q: &dyn Query, player: PlayerId) -> bool {
    q.history().this_turn().iter().any(|r| matches!(r.event, Event::Attacks { .. }) && r.active == player)
}

/// "Draw two cards. Then discard a card unless you attacked this turn."
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let player = item.controller;
    g.draw(player, 2);
    if !attacked_this_turn(g, player) {
        discard_chosen(g, player, item.object, 1)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[], effect), ..oracle::CHART_A_COURSE };
