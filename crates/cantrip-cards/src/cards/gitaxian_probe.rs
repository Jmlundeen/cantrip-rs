use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Look at target player's hand; draw a card. The look is recorded card by card (`Game::look_at`, `Event::Looked`), so
/// what its caster has seen is known to the history and the view.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(player) = targets::player(g, item, 0) {
        let hand = g.state().zone(Zone::hand(player)).to_vec();
        g.look_at(item.controller, &hand);
    }
    g.draw(item.controller, 1);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], effect),
    ..oracle::GITAXIAN_PROBE
};
