use cantrip_api::*;
use cantrip_mechanics::replicate;
use cantrip_oracle as oracle;

/// "Target triggered ability or colorless spell" (CR 115.1): a triggered ability on the stack, or a spell there with no
/// color. An activated ability is not one.
fn target_trigger_or_colorless_spell(q: &dyn Query, _: &Asker, target: Target) -> bool {
    match target {
        Target::Ability(serial) => q.state().stack.iter().any(|i| i.serial == serial && i.kind == StackKind::Triggered),
        Target::Object(id) => q.state().stack.iter().any(|i| i.kind == StackKind::Spell && i.object == id)
            && q.chars(id).colors.is_empty(),
        _ => false,
    }
}

/// "Counter target triggered ability or colorless spell." (CR 701.6a, 701.6b) Nothing if the target is no longer legal
/// (CR 608.2b).
fn counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    match item.target(0) {
        Target::Ability(_) => { cantrip_mechanics::counter_target_ability(g, item, 0); }
        _ => { cantrip_mechanics::counter_target(g, item, 0); }
    }
    Ok(())
}

pub static CARD: CardDef = CardDef {
    spell: spell(&[target(target_trigger_or_colorless_spell)], counter),
    kicker: multikicker(mana("{1}")),
    abilities: &[replicate::trigger("Replicate {1} (When you cast this spell, copy it for each time you paid its replicate cost. You may choose new targets for the copies.)")],
    ..oracle::CONSIGN_TO_MEMORY
};
