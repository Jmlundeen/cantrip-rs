use cantrip_api::*;
use cantrip_mechanics::{delayed, history};
use cantrip_oracle as oracle;

/// "If it entered from your graveyard or you cast it from your graveyard" (CR 603.4): `permanent`, a creature, entered from
/// a graveyard of which `you` are the owner (a graveyard is its owner's, CR 404.1), or `cast` - the cast from a graveyard of
/// the spell it entered as, if any - was `you` casting it from your graveyard.
fn from_your_graveyard(q: &dyn Query, you: PlayerId, permanent: ObjectId, from: Zone, cast: Option<&Cast>) -> bool {
    (from.is(ZoneKind::Graveyard) && q.obj(permanent).owner == you)
        || cast.is_some_and(|cast| cast.player == you && q.obj(cast.spell).owner == you)
}

/// "Whenever a creature enters" (CR 603.2): the trigger asks the intervening "if" as it is checked (CR 603.4), from the
/// history just before the entry; a creature that came from the stack entered as the spell that resolved (`ZoneChange::old`).
fn when_creature_enters(q: &dyn Query, asker: &Asker, event: &Event) -> Option<Fired> {
    let Event::ZoneChange { old, new, from, to, reason, .. } = event else { return None };
    if !to.is(ZoneKind::Battlefield) || !q.chars(*new).is(CardType::Creature) {
        return None;
    }
    let cast = if *reason == ZoneReason::Resolve { history::cast_from_graveyard(q.history(), *old) } else { None };
    if from_your_graveyard(q, asker.controller, *new, *from, cast) { Fired::about(Target::Object(*new)) } else { None }
}

/// "Return this card from your graveyard to the battlefield tapped at the beginning of the next end step" (CR 603.7): the
/// delayed ability puts the card, tapped, onto the battlefield.
fn delayed_1(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.put_all_onto_battlefield_as(&[(item.object, item.controller, true)])?;
    Ok(())
}

/// "If it entered from your graveyard or you cast it from your graveyard" (CR 603.4): checked again as the ability resolves,
/// from the history, since the creature has entered by then; if so, the card returns at the beginning of the next end step.
fn if_entered_graveyard_cast_graveyard(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let q: &dyn Query = g;
    let Some(creature) = item.subject.object() else { return Ok(()) };
    let history = q.history();
    let entered_from = history.select(EventQuery::kind(EventMask::ZONE_CHANGE).to(ZoneKind::Battlefield)).rev().find_map(|r| match r.event {
        Event::ZoneChange { new, from, .. } if new == creature => Some(from),
        _ => None,
    });
    let cast = history::cast_from_graveyard_as_it_entered(history, creature);
    if entered_from.is_some_and(|from| from_your_graveyard(q, item.controller, creature, from, cast)) {
        delayed::at_next_end_step(g, item, 0, Target::None);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        triggered(
                when_asked(EventMask::ZONE_CHANGE, when_creature_enters),
                if_entered_graveyard_cast_graveyard,
                "Whenever a creature enters, if it entered from your graveyard or you cast it from your graveyard, return this card from your graveyard to the battlefield tapped at the beginning of the next end step.",
            )
            .functions_in(ZoneKinds::of(&[ZoneKind::Graveyard])),
    ],
    delayed: &[
        delayed_trigger(
            delayed_1,
            "Return this card from your graveyard to the battlefield tapped.",
        ),
    ],
    ..oracle::PRIZED_AMALGAM
};
