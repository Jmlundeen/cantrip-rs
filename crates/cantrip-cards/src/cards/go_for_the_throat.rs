use cantrip_api::*;
use cantrip_mechanics::{destroy_target, targets};
use cantrip_oracle as oracle;

/// "Destroy target nonartifact creature" (CR 115.1): a creature that is not an artifact.
fn nonartifact_creature(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::CREATURE.filter)(q, asker, t) && matches!(t, Target::Object(id) if !q.chars(id).is(CardType::Artifact))
}

/// "Destroy target nonartifact creature" (CR 701.8). If the target is an artifact when it tries to resolve, it is
/// illegal and the spell does not resolve (CR 608.2b).
fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    destroy_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef { spell: spell(&[target(nonartifact_creature)], effect), ..oracle::GO_FOR_THE_THROAT };
