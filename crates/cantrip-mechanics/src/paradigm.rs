//! Paradigm (CR 702.192a): "If this is the first time a spell you control with this spell's name has resolved this game, at the
//! beginning of each of your precombat main phases for the rest of the game, create a copy of this object in exile. You may
//! cast the copy without paying its mana cost" and "Exile this spell." Germination Practicum and Improvisation Capstone print it.
//!
//! A card with paradigm sets `paradigm: true` (the core exiles the spell as it resolves) and lists `TRIGGER` among its
//! `abilities`. The delayed trigger is the card's own triggered ability, kept in exile and only for the first spell of its
//! name to resolve. Known gap: the API has no delayed trigger that lasts for the rest of the game, so it ends if that card
//! leaves exile (the ruling says it would go on, and cast a copy from nothing).

use cantrip_api::*;

/// "If this is the first time a spell you control with this spell's name has resolved this game": the first spell with this
/// card's name that its owner cast to resolve, asked of the event history, is this card as it left the stack. A spell that
/// was countered or had no legal target did not resolve, and a second spell of the name resolving later makes no second
/// delayed trigger (CR 702.192a). Only the object the first spell became is the one that triggers.
fn is_the_first_to_resolve(q: &dyn Query, card: ObjectId) -> bool {
    let owner = q.obj(card).owner;
    let first = q.history().all().iter().find_map(|r| match r.event {
        Event::ZoneChange { old, new, from, to, reason: ZoneReason::Resolve }
            if from.is(ZoneKind::Stack)
                && !to.is(ZoneKind::Gone)
                && crate::same_name(q, new, card)
                && q.history().all().iter().any(|c| matches!(&c.event, Event::Cast(cast) if cast.spell == old && cast.player == owner)) =>
        {
            Some((new, to))
        }
        _ => None,
    });
    first.is_some_and(|(new, to)| new == card && to.is(ZoneKind::Exile))
}

/// "At the beginning of each of your precombat main phases" (CR 505.1, 702.192a): its owner's precombat main phase, for
/// the card the first such spell was, whether or not it is still in exile where paradigm put it.
fn your_first_main_phase(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::PrecombatMain }
            if q.state().turn.active == q.obj(source).owner && is_the_first_to_resolve(q, source) =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "You may cast a copy of it from exile without paying its mana cost" (CR 707.12). The card stays in exile for the next
/// first main phase; the copy ceases to exist once it has resolved. A copy that could not be cast, a "no", and a cast
/// given up all leave it as it was.
fn cast_a_copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let card = item.object;
    if !is_live(g, card) || !g.obj(card).zone.is(ZoneKind::Exile) || !g.could_cast_during_resolution(card) {
        return Ok(());
    }
    if g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::CastCopyFree })? == Answer::Yes {
        g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), copy: true, ..CastTerms::default() })?;
    }
    Ok(())
}

pub const TRIGGER: AbilityDef = triggered(when(EventMask::STEP_BEGAN, your_first_main_phase), cast_a_copy, "Paradigm: at the beginning of each of your first main phases, you may cast a copy of this spell from exile without paying its mana cost.")
    .anywhere();
