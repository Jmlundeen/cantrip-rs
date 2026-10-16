use cantrip_api::*;
use cantrip_mechanics::targets;
use cantrip_oracle as oracle;

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
    abilities: &[triggered(cantrip_mechanics::triggers::THIS_ENTERS_IF_CAST, copy, "When Lutri, the Spellchaser enters, if you cast it, copy target instant or sorcery spell you control. You may choose new targets for the copy.")
        .targets(&[target(your_instant_or_sorcery_spell)])],
    ..oracle::LUTRI_THE_SPELLCHASER
};
