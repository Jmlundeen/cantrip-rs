use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// "Another target artifact" (CR 115.1): any artifact on the battlefield but the Key itself.
fn another_artifact(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::ARTIFACT.filter)(q, asker, t) && t != Target::Object(asker.source)
}

fn untap_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(artifact) = targets::object(g, item, 0) {
        g.untap(artifact);
    }
    Ok(())
}

/// The creature can't be blocked this turn (CR 509.1b, 611.2).
fn cant_be_blocked(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(creature) = targets::object(g, item, 0) {
        g.add_effect(item.object, creature, EffectKind::AddKeywords(Keywords::of(&[Keyword::CantBeBlocked])), Duration::EndOfTurn);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    abilities: &[
        ability(cost("{1}, {T}"), untap_it, "{1}, {T}: Untap another target artifact.")
            .targets(&[TargetSpec { min: 1, max: 1, filter: another_artifact }]),
        ability(cost("{3}, {T}"), cant_be_blocked, "{3}, {T}: Target creature can't be blocked this turn.")
            .targets(&[targets::CREATURE]),
    ],
    ..oracle::MANIFOLD_KEY
};
