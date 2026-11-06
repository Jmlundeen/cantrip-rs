use cantrip_api::*;
use cantrip_mechanics::effects;
use cantrip_oracle as oracle;

/// An artifact or enchantment spell on the stack.
fn artifact_or_enchantment_spell(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && (q.chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Enchantment)))
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(artifact_or_enchantment_spell)], effects::counter_first_target),
    ..oracle::ANNUL
};
