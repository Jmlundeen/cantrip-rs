use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Enchant creature or planeswalker you control" (CR 303.4a).
fn creature_or_planeswalker_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE_OR_PLANESWALKER.filter)(q, asker, t) && matches!(t, Target::Object(id) if q.controller(id) == asker.controller)
}

/// "When enchanted permanent dies or is put into exile" (CR 700.4, 603.10a): the permanent this Aura was attached to
/// went from the battlefield to a graveyard or exile. The ability is about it (`StackItem::subject`), as it was.
fn enchanted_permanent_dies_or_is_exiled(q: &dyn Query, ghostform: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && (to.is(ZoneKind::Graveyard) || to.is(ZoneKind::Exile))
                && q.state().obj(ghostform).attached_to == Some(*old) =>
        {
            Fired::about(Target::Object(*old))
        }
        _ => None,
    }
}

/// "Return that card to the battlefield under your control" (CR 603.6c, 730.3c): each card the permanent became, if it is
/// still where it went (CR 400.7), under the ability's controller, together.
fn return_that_card(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(permanent) = item.subject else { return Ok(()) };
    let cards: Vec<(ObjectId, PlayerId, bool)> = g.found_all_after_move(permanent).into_iter().map(|card| (card, item.controller, false)).collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield_as(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when(EventMask::ZONE_CHANGE, enchanted_permanent_dies_or_is_exiled),
        return_that_card,
        "When enchanted permanent dies or is put into exile, return that card to the battlefield under your control.",
    )],
    enchant: Some(target(creature_or_planeswalker_you_control)),
    ..oracle::KAYAS_GHOSTFORM
};
