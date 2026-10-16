use cantrip_api::*;
use cantrip_mechanics::plot::become_plotted;
use cantrip_mechanics::triggers::THIS_ENTERS;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "When this creature enters, exile target spell. It becomes plotted." The spell leaves the stack without being
/// countered (CR 400.3), so a spell that can't be countered is exiled too; it is exiled to its owner's exile zone and
/// becomes plotted there (CR 702.170a). A copy of a spell is exiled and ceases to exist (CR 111.7), so it is not plotted.
fn exile_and_plot(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(spell) = targets::object(g, item, 0) else { return Ok(()) };
    g.remove_spell(spell, CounterTo::Exile);
    if let Some(exiled) = g.obj(spell).became {
        become_plotted(g, exiled);
    }
    Ok(())
}

/// "Spells your opponents cast from graveyards or from exile cost {2} more to cast" (CR 601.2f): a spell on the stack
/// that was cast from one of those zones, or a card in one that would be.
fn opponents_graveyard_and_exile_spells_cost_two_more(q: &dyn Query, source: ObjectId, player: PlayerId, costed: Costed) -> i8 {
    let Costed::Spell(spell) = costed else { return 0 };
    let obj = q.obj(spell);
    let from = if obj.zone.is(ZoneKind::Stack) { obj.cast_from } else { Some(obj.zone.kind()) };
    if player != q.controller(source) && matches!(from, Some(ZoneKind::Graveyard | ZoneKind::Exile)) { 2 } else { 0 }
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(THIS_ENTERS, exile_and_plot, "When this creature enters, exile target spell. It becomes plotted. (Its owner may cast it as a sorcery on a later turn without paying its mana cost.)")
        .targets(&[targets::SPELL])],
    cost_changes: &[opponents_graveyard_and_exile_spells_cost_two_more],
    ..oracle::AVEN_INTERRUPTER
};
