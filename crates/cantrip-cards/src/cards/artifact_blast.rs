use cantrip_api::*;
use cantrip_mechanics::{effects, targets};
use cantrip_oracle as oracle;

/// An artifact spell on the stack (CR 115.1a, 112.1).
fn artifact_spell(q: &dyn Query, asker: &Asker, t: Target) -> bool {
    (targets::SPELL.filter)(q, asker, t)
        && matches!(t, Target::Object(id) if q.chars(id).is(CardType::Artifact))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(artifact_spell)], effects::counter_first_target),
    ..oracle::ARTIFACT_BLAST
};
