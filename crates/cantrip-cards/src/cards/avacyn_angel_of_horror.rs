use cantrip_api::*;
use cantrip_mechanics::delayed;
use cantrip_oracle as oracle;

/// "Whenever Avacyn or another nontoken creature you control dies" (CR 700.4, 603.10a): a dies ability, looked back on,
/// so it also fires for a creature that died in the same event as this one, and once for each of them. "Nontoken" qualifies
/// only "another ... creature": Avacyn herself triggers it whatever she is, a token copy of her included, though a
/// token that dies leaves nothing to return (CR 111.7). The card it is about is the new object the creature became in
/// the graveyard (CR 400.7).
fn avacyn_or_a_nontoken_creature_dies(q: &dyn Query, me: &Asker, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, from, to, .. }
            if from.is(ZoneKind::Battlefield)
                && to.is(ZoneKind::Graveyard)
                && q.chars(*old).is(CardType::Creature)
                && q.obj(*old).controller == me.controller
                && (*old == me.source || !q.obj(*old).token) =>
        {
            Fired::about(Target::Object(*new))
        }
        _ => None,
    }
}

/// "Return that card to the battlefield under your control at the beginning of the next end step" (CR 603.7c): a delayed
/// ability about that one card in the graveyard, set up as the dies ability resolves.
fn return_it_at_the_next_end_step(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Target::Object(card) = item.subject {
        delayed::at_next_end_step(g, item, 0, Target::Object(card));
    }
    Ok(())
}

/// The card, if it is still the card it was when the ability was set up (a new object once it has left its zone, CR 603.7c),
/// onto the battlefield under this ability's controller, who need not be its owner (CR 110.2a).
fn return_under_your_control(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Target::Object(card) = item.subject else { return Ok(()) };
    if !is_live(g, card) {
        return Ok(());
    }
    g.put_onto_battlefield(card, item.controller, false)?;
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
