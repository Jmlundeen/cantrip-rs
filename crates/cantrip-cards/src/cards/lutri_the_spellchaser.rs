use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "When Lutri enters, if you cast it" (CR 603.4, 603.6a): the spell it entered as was cast, this turn's history says; a
/// permanent that was put onto the battlefield, or a copy of a permanent spell, was not cast.
fn entered_after_being_cast(q: &dyn Query, source: ObjectId, event: &Event) -> Option<Fired> {
    match event {
        Event::ZoneChange { old, new, to, reason: ZoneReason::Resolve, .. }
            if *new == source && to.is(ZoneKind::Battlefield) && q.history().cast_of(*old).is_some() =>
        {
            Fired::PLAIN
        }
        _ => None,
    }
}

/// "Target instant or sorcery spell you control" (CR 115.1): a spell on the stack, not a permanent.
fn your_instant_or_sorcery_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && q.controller(id) == asker.controller
            && {
                let c = q.chars(id);
                c.is(CardType::Instant) || c.is(CardType::Sorcery)
            })
}

/// "Copy target instant or sorcery spell you control. You may choose new targets for the copy."
fn copy(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.copy_spell(spell, item.controller, true)?;
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Lutri, the Spellchaser enters, if you cast it, copy target instant or sorcery spell you control. You may choose new targets for the copy.",
        listens: EventMask::ZONE_CHANGE,
        matches: entered_after_being_cast,
        targets: &[TargetSpec { min: 1, max: 1, filter: your_instant_or_sorcery_spell }],
        effect: copy,
        ..TriggeredDef::BLANK
    })],
    ..oracle::LUTRI_THE_SPELLCHASER
};
