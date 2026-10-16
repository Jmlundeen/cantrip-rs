use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

/// A noncreature spell on the stack.
fn noncreature_spell(q: &dyn Query, _: &Asker, t: Target) -> bool {
    matches!(t, Target::Object(id)
        if q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && !q.chars(id).is(CardType::Creature))
}

fn effect(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if let Some(spell) = targets::object(g, item, 0) {
        g.counter(spell);
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: Some(SpellDef { targets: &[TargetSpec { min: 1, max: 1, filter: noncreature_spell, label: "" }], effect }),
    ..oracle::NEGATE
};
