use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// "Whenever Avacyn or another nontoken creature you control dies" (CR 700.4, 603.10a): a dies ability, looked back on,
/// so it also fires for a creature that died in the same event as this one, and once for each of them. "Nontoken" qualifies
/// only "another ... creature": Avacyn herself triggers it whatever she is, a token copy of her included, though a
/// token that dies leaves nothing to return (CR 111.7). The ability is about the creature that died, from which "that card" finds
/// the new object it became in the graveyard (CR 400.7) - every card, if it was a merged permanent, which dies once (CR 730.3, 730.3c).
fn avacyn_or_a_nontoken_creature_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == me.controller
                && (*old == me.source || !q.obj(*old).token) =>
        {
            Fired::about(Target::Object(*old))
        }
        _ => None,
    }
}

/// "Return that card to the battlefield under your control at the beginning of the next end step" (CR 603.7c): a delayed
/// ability about the cards the creature became in the graveyard, set up as the dies ability resolves.
fn return_it_at_the_next_end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(creature) = item.subject {
        // Made whatever the creature became: a token has ceased to exist by then (CR 111.7), and the ability finds nothing.
        let cards = g.found_all_after_move(creature);
        delayed::at_next_end_step_about(g, item, 0, &cards);
    }
    Ok(())
}

/// Each card the ability carries, if it is still the card it was when the ability was set up (a new object once it has left its
/// zone, CR 603.7c), onto the battlefield together under this ability's controller, who need not be its owner (CR 110.2a).
fn return_under_your_control(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let cards: Vec<(ObjectId, PlayerId, bool)> = g.event_objects(item).iter().copied().filter(|&c| is_live(g, c)).map(|c| (c, item.controller, false)).collect();
    if !cards.is_empty() {
        g.put_all_onto_battlefield_as(&cards)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(
        when_asked(EventMask::ZONE_CHANGE, avacyn_or_a_nontoken_creature_dies),
        return_it_at_the_next_end_step,
        "Whenever Avacyn or another nontoken creature you control dies, return that card to the battlefield under your control at the beginning of the next end step.",
    )],
    delayed: &[delayed_trigger(
        return_under_your_control,
        "Return that card to the battlefield under your control at the beginning of the next end step.",
    )],
    ..oracle::AVACYN_ANGEL_OF_HORROR
};
