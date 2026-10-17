use cantrip_api::*;
use cantrip_mechanics::flashback;
use cantrip_oracle as oracle;

/// "Draw a card. If this spell was cast from a graveyard, draw two cards instead." The zone it was cast from is
/// the spell's own (CR 601.2a). A copy was never cast (CR 707.10), so it should draw one; the engine's `copy_spell` still lets a
/// copy inherit `cast_from` (KI-31, the known-bug scenario), so for now a copy of a flashbacked Plunder draws two.
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
