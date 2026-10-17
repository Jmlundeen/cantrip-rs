use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

/// "Draw a card. If this spell was cast from a graveyard, draw two cards instead." The zone it was cast from is
/// the spell's own (CR 601.2a); a copy was never cast (CR 707.10) and draws one.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let count = if g.obj(item.object).cast_from == Some(ZoneKind::Graveyard) { 2 } else { 1 };
    g.draw(item.controller, count);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[], effect),
    ways: &[flashback::way(cost("{3}{U}"), "Flashback {3}{U}")],
    ..oracle::PLUNDER_THE_TROLLSHAWS
};
