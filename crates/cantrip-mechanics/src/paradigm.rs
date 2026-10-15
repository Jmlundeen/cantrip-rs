//! Paradigm (CR 702.170a): "Then exile this spell. After you first resolve a spell with this name, you may cast a copy
//! of it from exile without paying its mana cost at the beginning of each of your first main phases." Germination
//! Practicum prints it.
//!
//! A card with paradigm sets `paradigm: true` (the core exiles the spell as it resolves) and lists `TRIGGER` among its
//! `abilities`.

use cantrip_api::*;

/// Whether this card was exiled by paradigm: it left the stack for exile as it resolved, asked of the event history. A
/// card exiled any other way has no paradigm to use.
fn exiled_as_it_resolved(q: &dyn Query, card: ObjectId) -> bool {
    q.history().all().iter().any(|r| {
        matches!(r.event, Event::ZoneChange { new, from, to, reason: ZoneReason::Resolve, .. }
            if new == card && from.is(ZoneKind::Stack) && to.is(ZoneKind::Exile))
    })
}

/// "At the beginning of each of your first main phases" (CR 505.1): its owner's precombat main phase, while the card the
/// spell was is still in exile where paradigm put it.
fn your_first_main_phase(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::StepBegan { step: Step::PrecombatMain }
            if q.obj(source).zone.is(ZoneKind::Exile)
                && q.state().turn.active == q.obj(source).owner
                && exiled_as_it_resolved(q, source) =>
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
    if g.ask(item.controller, RequestKind::May { source: card, purpose: Purpose::Cast })? == Answer::Yes {
        g.cast_during_resolution_as(card, CastTerms { instead: Some(ManaCost::ZERO), copy: true, ..CastTerms::default() })?;
    }
    Ok(())
}

pub const TRIGGER: AbilityDef = AbilityDef::Triggered(TriggeredDef {
    text: "Paradigm: at the beginning of each of your first main phases, you may cast a copy of this spell from exile without paying its mana cost.",
    listens: EventMask::STEP_BEGAN,
    matches: your_first_main_phase,
    effect: cast_a_copy,
    anywhere: true,
    ..TriggeredDef::BLANK
});
