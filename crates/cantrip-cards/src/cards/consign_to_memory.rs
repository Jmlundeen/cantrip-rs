use cantrip_api::*;
use cantrip_mechanics::replicate;
use cantrip_oracle as oracle;

fn target_trigger_or_colorless_spell(q: &dyn Query, _: &Asker, target: Target) -> bool {
    match target {
        Target::Ability(serial) => q.state().stack.iter().any(|i| i.serial == serial && i.kind == StackKind::Triggered),
        Target::Object(id) => q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && q.chars(id).colors.is_empty(),
        _ => false,
    }
}

fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    if !g.target_legal(item, 0) { return Ok(()) }
    match item.targets[0] {
        Target::Object(id) => g.counter(id),
        Target::Ability(serial) => {
            if let Some(at) = g.state().stack.iter().position(|i| i.serial == serial) {
                let source = g.state().stack[at].object;
                g.counter_item(source, at);
            }
        }
        _ => {}
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(target_trigger_or_colorless_spell)], counter),
    kicker: multikicker(mana("{1}")),
    abilities: &[replicate::trigger("Replicate {1} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)")],
    ..oracle::CONSIGN_TO_MEMORY
};
