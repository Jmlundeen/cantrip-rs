use cantrip_api::*;
use cantrip_oracle as oracle;

/// "This artifact enters with three film counters on it" (CR 122.6, 614.1c).
fn enters_with_film_counters(_: &mut dyn Game, entering: &mut Entering) -> Result<(), Suspend> {
    entering.counters[CounterKind::Film as usize] += 3;
    Ok(())
}

/// "Target activated or triggered ability you control" (CR 115.1, 113.3): an ability on the stack (a mana ability never is,
/// CR 605.3b) whose controller is the one activating. A spell is not one, nor is an ability an opponent controls.
fn ability_you_control(q: &dyn Query, asker: &Asker, target: Target) -> bool {
    matches!(target, Target::Ability(serial)
        if q.state().stack.iter().any(|i| i.serial == serial && i.kind != StackKind::Spell && i.controller == asker.controller))
}

/// "Copy target activated or triggered ability you control. You may choose new targets for the copy." (CR 707.10, 707.10c)
/// Nothing if the ability is no longer there (CR 608.2b).
fn copy_it(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    g.copy_ability(item.target(0), item.controller, true)?;
    Ok(())
}

pub static CARD: CardDef = CardDef {
    enters: Some(enters_with_film_counters),
    abilities: &[ability(
        cost("{2}").and(TAP).and(remove_counters(CounterKind::Film, 1)),
        copy_it,
        "{2}, {T}, Remove a film counter from this artifact: Copy target activated or triggered ability you control. You may choose new targets for the copy.",
    )
    .targets(&[target(ability_you_control)])],
    ..oracle::PETER_PARKERS_CAMERA
};
