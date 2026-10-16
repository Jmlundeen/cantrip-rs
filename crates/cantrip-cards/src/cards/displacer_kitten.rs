use cantrip_api::*;
use cantrip_mechanics::{triggers::NONCREATURE_SPELL_YOU_CAST, targets};
use cantrip_oracle as oracle;

/// "Up to one target nonland permanent you control".
fn nonland_permanent_you_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if q.controller(id) == asker.controller) && (targets::NONLAND_PERMANENT.filter)(q, asker, t)
}

/// Exile it, then return that card to the battlefield under its owner's control: a new object (CR 400.7). A token
/// ceases to exist in exile and does not return (CR 111.7).
fn blink(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(permanent) = targets::object(g, item, 0) else { return Ok(()) };
    let token = g.obj(permanent).token;
    let owner = g.obj(permanent).owner;
    let exiled = g.exile(permanent);
    if token || !g.obj(exiled).zone.is(ZoneKind::Exile) {
        return Ok(());
    }
    g.put_onto_battlefield(exiled, owner, false)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[triggered(NONCREATURE_SPELL_YOU_CAST, blink, "Avoidance \u{2014} Whenever you cast a noncreature spell, exile up to one target nonland permanent you control, then return that card to the battlefield under its owner's control.")
        .targets(&[up_to_one(nonland_permanent_you_control)])],
    ..oracle::DISPLACER_KITTEN
};
