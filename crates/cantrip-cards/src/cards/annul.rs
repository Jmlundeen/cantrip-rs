use cantrip_api::*;
use cantrip_mechanics::counter_target;
use cantrip_oracle as oracle;

/// An artifact or enchantment spell on the stack.
fn artifact_or_enchantment_spell(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && (q.chars(id).is(CardType::Artifact) || q.chars(id).is(CardType::Enchantment)))
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    counter_target(g, item, 0);
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: artifact_or_enchantment_spell }], effect }),
    ..oracle::ANNUL
};
