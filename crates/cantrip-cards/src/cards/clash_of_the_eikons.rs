use cantrip_api::*;
use cantrip_mechanics::{fight, lands, targets};
use cantrip_oracle as oracle;

/// "Target creature you control fights target creature an opponent controls." (CR 701.14a-b): neither deals damage when either
/// target is no longer legal.
fn fight_targets(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    fight::targets_fight(g, item, 0, 1);
    Ok(())
}

/// "Remove a lore counter from target Saga you control." (CR 122.1, 714.2b): removing a lore counter triggers no chapter ability;
/// the Saga is sacrificed only if it still has as many as its last chapter once state-based actions are checked (CR 714.4).
fn remove_lore_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(saga) = targets::object(g, item, 0) else { return Ok(()) };
    g.remove_counters(saga, CounterKind::Lore, 1);
    Ok(())
}

/// "Put a lore counter on target Saga you control." (CR 122.1, 714.2a): the Saga's next chapter ability triggers if it now has
/// at least as many lore counters as that chapter's number and had fewer before (CR 714.2c).
fn put_lore_counter(g: &mut dyn Game, item: &StackItem) -> Result<(), Suspend> {
    let Some(saga) = targets::object(g, item, 0) else { return Ok(()) };
    g.add_counters(saga, CounterKind::Lore, 1);
    Ok(())
}

/// "Choose one or more" (CR 700.2): each chosen mode has its own targets (CR 700.2c) and is done in printed order (CR 700.2d).
static MODAL: ModalDef = modes(&[
    mode(fight_targets, "Target creature you control fights target creature an opponent controls.")
        .targets(&[targets::CREATURE_YOU_CONTROL, targets::CREATURE_AN_OPPONENT_CONTROLS]),
    mode(remove_lore_counter, "Remove a lore counter from target Saga you control.")
        .targets(&[target(targets::permanent_of_subtype_you_control::<{ lands::sub(Subtype::Saga) }>)]),
    mode(put_lore_counter, "Put a lore counter on target Saga you control.")
        .targets(&[target(targets::permanent_of_subtype_you_control::<{ lands::sub(Subtype::Saga) }>)]),
])
.choose(1, 3);

pub static CARD: CardDef = CardDef { spell: Some(SpellDef::MODAL), modal: Some(&MODAL), ..oracle::CLASH_OF_THE_EIKONS };
