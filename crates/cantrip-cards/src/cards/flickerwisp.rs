use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_mechanics::triggers::this_enters;
use cantrip_oracle as oracle;

/// "Another target permanent": any permanent on the battlefield but the source itself.
fn another_permanent(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if id != asker.source) && (targets::PERMANENT.filter)(q, asker, t)
}

/// Exile another target permanent. Return that card to the battlefield under its owner's control at the beginning of
/// the next end step (CR 603.7): a delayed ability about the card exiled. A token exiled this way ceases to exist, so the card the ability finds is gone (CR 111.7, 400.7).
fn exile(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let exiled = g.exile(permanent);
    if !g.obj(exiled).zone.is(ZoneKind::Exile) {
        return Ok(());
    }
    cantrip_mechanics::delayed::at_next_end_step(g, item, 0, Target::Object(exiled));
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[AbilityDef::Triggered(TriggeredDef {
        text: "When Flickerwisp enters, exile another target permanent. Return that card to the battlefield under its owner's control at the beginning of the next end step.",
        listens: EventMask::ZONE_CHANGE,
        matches: this_enters,
        targets: &[TargetSpec { min: 1, max: 1, filter: another_permanent }],
        effect: exile,
        ..TriggeredDef::BLANK
    })],
    delayed: &[AbilityDef::Triggered(TriggeredDef {
        text: "Return the card exiled with Flickerwisp to the battlefield under its owner's control.",
        effect: cantrip_mechanics::delayed::return_subject_to_battlefield,
        ..TriggeredDef::BLANK
    })],
    ..oracle::FLICKERWISP
};
