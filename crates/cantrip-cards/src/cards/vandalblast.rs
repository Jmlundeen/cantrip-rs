use cantrip_api::*;
use cantrip_mechanics::overload;
use cantrip_oracle as oracle;

/// "Target artifact you don't control": not "an opponent controls" (a teammate's artifact is one you don't control).
fn artifact_you_dont_control(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id) if is_live(q, id) && q.is_on_battlefield(id)
        && q.chars(id).is(CardType::Artifact) && q.controller(id) != asker.controller)
}

/// Destroy target artifact you don't control. Overloaded (CR 702.96a), destroy each artifact you don't control.
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let artifacts = overload::affected(g, item, |chars, controller| chars.is(CardType::Artifact) && controller != item.controller);
    g.destroy_all(&artifacts);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: artifact_you_dont_control }], effect }),
    overload: Some(mana(4, 0, 0, 0, 1, 0)),
    ..oracle::VANDALBLAST
};
