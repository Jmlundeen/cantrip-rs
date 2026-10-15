use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// Exile target creature you control, then return it to the battlefield under its owner's control. It returns as a
/// new object (CR 400.7); a token exiled this way ceases to exist and does not return (CR 111.7). Nothing happens if
/// the creature is no longer a legal target (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(creature) = targets::object(g, item, 0) else { return Ok(()) };
    let owner = g.obj(creature).owner;
    let exiled = g.exile(creature);
    if !g.obj(exiled).token && g.obj(exiled).zone.is(ZoneKind::Exile) {
        g.put_onto_battlefield(exiled, owner, false)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[targets::CREATURE_YOU_CONTROL], effect }),
    rebound: true,
    ..oracle::EPHEMERATE
};
