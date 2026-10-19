use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Draw X cards, where X is the number of cards that were put into target player's graveyard from their
/// library this turn." Count this turn’s mill events on resolution.
fn cruel_calculations(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(player) = targets::player(g, item, 0) else { return Ok(()) };
    let x = g
        .history()
        .select(EventQuery::kind(EventMask::ZONE_CHANGE).from(ZoneKind::Library).to(ZoneKind::Graveyard).this_turn())
        .filter(|r| matches!(r.event, Event::ZoneChange { from, to, .. } if from.player == player && to.player == player))
        .count() as u32;
    g.draw(item.controller, x);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[targets::PLAYER], cruel_calculations),
    ..oracle::CRUEL_CALCULATIONS
};
